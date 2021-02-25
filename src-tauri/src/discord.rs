use std::sync::Arc;

use {
    crate::{
        model::{Page, ScreenAction, Service, User},
        Context,
    },
    anyhow::{Context as _, Result},
    parking_lot::RwLock,
    serenity::{
        async_trait,
        model::{channel::Message, prelude::Ready},
        prelude::{Client, Context as SerenityContext, EventHandler},
    },
};

const PREFIX: &str = "g!live";

enum Command {
    Help,
    Listen,
    StopListening,
    SetNotification(String),
    TimelineClear,
    Pause,
    Resume,
}

struct DiscordListenerInner {
    listening_channel_id: Option<u64>,
    my_id: Option<u64>,
}

pub struct DiscordListener {
    inner: RwLock<DiscordListenerInner>,
    ctx: Arc<Context>,
}

impl DiscordListener {
    pub(crate) fn new(ctx: Arc<Context>) -> Self {
        Self {
            ctx,
            inner: RwLock::new(DiscordListenerInner {
                listening_channel_id: None,
                my_id: None,
            }),
        }
    }

    pub async fn start(self, token: &str) -> Result<()> {
        Client::builder(token)
            .event_handler(self)
            .await
            .context("Failed to create discord client")?
            .start()
            .await
            .context("Failed to start discord client")
    }

    fn parse(&self, msg: &str) -> Option<Command> {
        let mut tokens = msg.split(' ');

        let prefix = tokens.next();
        let sub_command = tokens.next();
        let args = tokens.collect::<Vec<_>>();

        use Command::*;

        match (prefix, sub_command, args.as_slice()) {
            (None, _, _) => None,
            (Some(p), _, _) if p != PREFIX => None,

            (_, Some("listen"), _) => Some(Listen),

            (_, Some("stop_listening"), _) => Some(StopListening),

            (_, Some("set_notification"), args) if args.is_empty() => {
                Some(SetNotification(args.join(" ")))
            }
            (_, Some("set_notification"), _) => Some(Help),

            (_, Some("clear_timeline"), _) => Some(TimelineClear),

            (_, Some("pause"), _) => Some(Pause),
            (_, Some("resume"), _) => Some(Resume),

            (_, _, _) => Some(Help),
        }
    }

    async fn invoke_command(&self, ctx: &SerenityContext, message: &Message, cmd: Command) {
        use Command::*;

        let text_buffer;
        let text = match (cmd, self.ctx.webview_chan.lock().await.as_ref()) {
            (Help, _) => "https://hackmd.io/@U9f9Fv6rTt2UkRA6UriFTA/BJRVQlTZO",

            (Listen, _) => {
                let chan = message.channel_id;
                self.inner.write().listening_channel_id = Some(chan.0);

                text_buffer = format!("now listening at <#{}>", chan.0);
                tracing::info!("{}", &text_buffer);

                text_buffer.as_str()
            }

            (StopListening, _) => {
                if self.inner.read().listening_channel_id.is_some() {
                    self.inner.write().listening_channel_id = None;
                    "stopped"
                } else {
                    "currently not listening any channel"
                }
            }

            (SetNotification(text), Some(sender)) => {
                sender
                    .send(ScreenAction::NotificationUpdate { text })
                    .await
                    .ok();

                "set"
            }

            (TimelineClear, Some(sender)) => {
                sender.send(ScreenAction::TimelineClear).await.ok();
                "cleared"
            }

            // TODO: lock during switching (2sec)
            (Pause, Some(sender)) => {
                sender
                    .send(ScreenAction::SwitchPage(Page::WaitingScreen))
                    .await
                    .ok();
                "switching requested"
            }

            (Resume, Some(sender)) => {
                sender
                    .send(ScreenAction::SwitchPage(Page::LTScreen))
                    .await
                    .ok();
                "switching requested"
            }

            (_, None) => "webview was not ready",
        };

        if let Err(e) = message.channel_id.say(&ctx, text).await {
            tracing::error!("failed to send message!: {:?}\n{}", e, &text);
        }
    }
}

#[async_trait]
impl EventHandler for DiscordListener {
    async fn ready(&self, _: SerenityContext, ready: Ready) {
        tracing::info!("DiscordBot({}) is connected!", ready.user.name);
        self.inner.write().my_id = Some(ready.user.id.0);
    }

    async fn message(&self, ctx: SerenityContext, message: Message) {
        if self.inner.read().my_id.unwrap() == message.author.id.0 {
            return;
        }

        let content = message.content.trim();
        if let Some(cmd) = self.parse(content) {
            self.invoke_command(&ctx, &message, cmd).await;
            return;
        }

        let listening_channel_id = self.inner.read().listening_channel_id;

        if let Some(target_id) = listening_channel_id {
            if target_id != message.channel_id.0 {
                return; // TODO:
            }

            let webview_chan = self.ctx.webview_chan.lock().await;

            if let Some(chan) = webview_chan.as_ref() {
                chan.send(ScreenAction::TimelinePush {
                    user: User {
                        icon: message.author.avatar_url(),
                        ident: None,
                        name: message
                            .author_nick(&ctx)
                            .await
                            .unwrap_or_else(|| message.author.name.clone()),
                    },
                    service: Service::Discord,
                    content: content.to_string(),
                })
                .await
                .ok();
            } else {
                tracing::warn!(
                    "failed to send TimelinePush event because Webview was not initialized"
                );
            }
        }
    }
}
