use {
    crate::{
        model::{ScreenAction, Service, User},
        Context,
    },
    egg_mode::{
        stream::{filter, StreamMessage},
        tweet::Tweet,
        Token,
    },
    std::sync::Arc,
    tokio_stream::StreamExt,
};

const HASHTAGS: &[&str] = &["#限界LT"];

pub(crate) struct TwitterListener {
    ctx: Arc<Context>,
    token: Token,
}

impl TwitterListener {
    pub(crate) fn new(ctx: Arc<Context>, token: Token) -> Self {
        Self { ctx, token }
    }

    pub(crate) async fn start(self) {
        let mut stream = filter().track(HASHTAGS).start(&self.token);

        while let Some(event) = stream.next().await {
            match event {
                Ok(StreamMessage::Tweet(tweet)) => self.on_tweet(tweet).await,
                Ok(_) => {}

                Err(e) => {
                    tracing::warn!("Twitter stream returned an error: {:#?}", e);
                    break;
                }
            }
        }
    }

    async fn on_tweet(&self, tweet: Tweet) {
        match self.ctx.webview_chan.read().await.as_ref() {
            Some(chan) => {
                let user = match tweet.user {
                    Some(u) => User {
                        icon: Some(u.profile_image_url_https),
                        ident: Some(u.screen_name),
                        name: u.name,
                    },

                    None => User {
                        icon: None,
                        ident: None,
                        name: "Unknown user".to_string(),
                    },
                };

                chan.send(ScreenAction::TimelinePush {
                    service: Service::Twitter,
                    content: tweet.text,
                    user,
                })
                .await
                .ok();
            }

            None => {
                tracing::warn!("couldn't send twitter event because Webview was not initialized")
            }
        }
    }
}
