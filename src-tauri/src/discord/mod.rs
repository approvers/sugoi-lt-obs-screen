use {
    anyhow::{Context as _, Result},
    serenity::{
        async_trait,
        model::{channel::Message, prelude::Ready},
        prelude::{Client, Context, EventHandler},
    },
    tokio::sync::RwLock,
};

const PREFIX: &str = "g!live";

enum Command {
    Help,
    Listen,
}

struct DiscordListenerInner {
    listening_channel_id: Option<u64>,
}

pub struct DiscordListener {
    inner: RwLock<DiscordListenerInner>,
}

impl DiscordListener {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(DiscordListenerInner {
                listening_channel_id: None,
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
    }

    async fn message(&self, ctx: Context, message: Message) {
        if self
            .inner
            .read()
            .await
            .listening_channel_id
            .map(|x| x == message.channel_id.0)
            .unwrap_or(false)
        {
            // TODO: notify
        }

        let mut tokens = message.content.split(" ");

        let prefix = tokens.next();
        let sub_command = tokens.next();
        let args = tokens.collect::<Vec<_>>();

        use Command::*;
        let run_cmd = |c| self.invoke_command(&ctx, &message, c);

        match (prefix, sub_command, args) {
            // stabilize or-patterns when
            (None, _, _) => return,
            (Some(p), _, _) if p != PREFIX => return,

            (_, Some("listen"), _) => run_cmd(Listen).await,
            (_, _, _) => run_cmd(Help).await,
        };
    }
}
