use std::sync::Arc;

use {
    crate::{
        model::{ScreenAction, Service, User},
        Context,
    },
    anyhow::{Context as _, Result},
    serenity::{
        async_trait,
        model::{channel::Message, prelude::Ready},
        prelude::{Client, Context as SerenityContext, EventHandler},
    },
    tokio::sync::RwLock,
};

const PREFIX: &str = "g!live";

enum Command {
    Help,
    Listen,
    StopListening,
    SetNotification(String),
    TimelineClear,
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

    async fn invoke_command(&self, ctx: &SerenityContext, message: &Message, cmd: Command) {
        use Command::*;

        let text_buffer;
        let text = match (cmd, self.ctx.webview_chan.lock().await.as_ref()) {
            (Help, _) => "https://hackmd.io/@U9f9Fv6rTt2UkRA6UriFTA/BJRVQlTZO",

            (Listen, _) => {
                let chan = message.channel_id;
                self.inner.write().await.listening_channel_id = Some(chan.0);

                text_buffer = format!("now listening at <#{}>", chan.0);
                tracing::info!("{}", &text_buffer);

                text_buffer.as_str()
            }

            (StopListening, _) => {
                if self.inner.read().await.listening_channel_id.is_some() {
                    self.inner.write().await.listening_channel_id = None;
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

            (SetNotification(_), None) | (TimelineClear, None) => "webview was not ready",
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
        self.inner.write().await.my_id = Some(ready.user.id.0);
    }

    async fn message(&self, ctx: SerenityContext, message: Message) {
        if self.inner.read().await.my_id.unwrap() == message.author.id.0 {
            return;
        }

        let content = message.content.trim();

        match (
            self.inner.read().await.listening_channel_id,
            self.ctx.webview_chan.lock().await.as_ref(),
        ) {
            (Some(target_id), Some(chan)) if message.channel_id == target_id => {
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
            }

            (Some(target_id), None) if message.channel_id == target_id => {
                tracing::warn!(
                    "failed to send TimelinePush event because Webview was not initialized"
                );
            }
            _ => {}
        };

        let mut tokens = content.split(" ");

        let prefix = tokens.next();
        let sub_command = tokens.next();
        let args = tokens.collect::<Vec<_>>();

        use Command::*;
        let run_cmd = |c| self.invoke_command(&ctx, &message, c);

        match (prefix, sub_command, args.as_slice()) {
            (None, _, _) => return,
            (Some(p), _, _) if p != PREFIX => return,

            (_, Some("listen"), _) => run_cmd(Listen).await,

            (_, Some("stop_listening"), _) => run_cmd(StopListening).await,

            (_, Some("set_notification"), args) if args.len() > 0 => {
                run_cmd(SetNotification(args.join(" "))).await
            }
            (_, Some("set_notification"), _) => run_cmd(Help).await,

            (_, Some("clear_timeline"), _) => run_cmd(TimelineClear).await,

            (_, _, _) => run_cmd(Help).await,
        };
    }
}
