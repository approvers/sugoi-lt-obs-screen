use {
    crate::model::{ScreenAction, Service, User},
    anyhow::{Context as _, Result},
    serenity::{
        async_trait,
        model::{channel::Message, prelude::Ready},
        prelude::{Client, Context, EventHandler},
    },
    tokio::sync::{mpsc::Sender, RwLock},
};

const PREFIX: &str = "g!live";

enum Command {
    Help,
    Listen,
    SetNotification(String),
    TimelineClear,
}

struct DiscordListenerInner {
    listening_channel_id: Option<u64>,
    my_id: Option<u64>,
}

pub struct DiscordListener {
    inner: RwLock<DiscordListenerInner>,
    sender: Sender<ScreenAction>,
}

impl DiscordListener {
    pub fn new(sender: Sender<ScreenAction>) -> Self {
        Self {
            sender,
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

    async fn invoke_command(&self, ctx: &Context, message: &Message, cmd: Command) {
        use Command::*;

        let text = match cmd {
            Help => "https://hackmd.io/@U9f9Fv6rTt2UkRA6UriFTA/BJRVQlTZO".to_string(),

            Listen => {
                let chan = message.channel_id;
                self.inner.write().await.listening_channel_id = Some(chan.0);

                let msg = format!("now listening at <#{}>", chan.0);
                tracing::info!("{}", &msg);

                msg
            }

            SetNotification(text) => {
                self.sender
                    .send(ScreenAction::NotificationUpdate { text })
                    .await
                    .ok();

                "set".to_string()
            }

            TimelineClear => {
                self.sender.send(ScreenAction::TimelineClear).await.ok();
                "cleared".to_string()
            }
        };

        if let Err(e) = message.channel_id.say(&ctx, &text).await {
            tracing::error!("failed to send message!: {:?}\n{}", e, &text);
        }
    }
}

#[async_trait]
impl EventHandler for DiscordListener {
    async fn ready(&self, _: Context, ready: Ready) {
        tracing::info!("DiscordBot({}) is connected!", ready.user.name);
        self.inner.write().await.my_id = Some(ready.user.id.0);
    }

    async fn message(&self, ctx: Context, message: Message) {
        if self.inner.read().await.my_id.unwrap() == message.author.id.0 {
            return;
        }

        let content = message.content.trim();

        if self
            .inner
            .read()
            .await
            .listening_channel_id
            .map(|x| x == message.channel_id.0)
            .unwrap_or(false)
        {
            self.sender
                .send(ScreenAction::TimelinePush {
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

        let mut tokens = content.split(" ");

        let prefix = tokens.next();
        let sub_command = tokens.next();
        let args = tokens.collect::<Vec<_>>();

        use Command::*;
        let run_cmd = |c| self.invoke_command(&ctx, &message, c);

        match (prefix, sub_command, args.as_slice()) {
            // stabilize or-patterns when
            (None, _, _) => return,
            (Some(p), _, _) if p != PREFIX => return,

            (_, Some("listen"), _) => run_cmd(Listen).await,

            (_, Some("set_notification"), args) if args.len() > 0 => {
                run_cmd(SetNotification(args.join(" "))).await
            }
            (_, Some("set_notification"), _) => run_cmd(Help).await,

            (_, Some("clear_timeline"), _) => run_cmd(TimelineClear).await,

            (_, _, _) => run_cmd(Help).await,
        };
    }
}
