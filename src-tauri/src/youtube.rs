use {
    crate::{
        model::{ScreenAction, Service, User},
        Context,
    },
    headless_chrome::{
        protocol::network::{
            events::ResponseReceivedEventParams, methods::GetResponseBodyReturnObject,
        },
        Browser, LaunchOptionsBuilder,
    },
    std::{future::Future, pin::Pin, sync::Arc, task::Poll, thread::sleep, time::Duration},
    tokio::task::block_in_place,
};

#[derive(Debug)]
struct Comment<'a> {
    author_icon: &'a str,
    author_name: &'a str,
    content: &'a str,
}

// a future that never ends.
struct Empty;

impl Future for Empty {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> Poll<()> {
        Poll::Pending
    }
}

pub(crate) struct YoutubeListener {
    ctx: Arc<Context>,
    video_id: String,
}

impl YoutubeListener {
    pub(crate) fn new(ctx: Arc<Context>, video_id: String) -> Self {
        Self { ctx, video_id }
    }

    pub(crate) async fn start(self) {
        // starting browser is heavy task, so we must use block_in_place
        // for not blocking any task on the same thread.
        // also block_in_place must return browser and tab not to run their destructors.
        let (_browser, _tab) = block_in_place(move || {
            let opt = LaunchOptionsBuilder::default()
                .headless(false)
                .build()
                .unwrap();

            let browser = Browser::new(opt).unwrap();
            let tab = browser.new_tab().unwrap();

            tab.enable_log().unwrap();

            tab.navigate_to(&format!(
                "https://www.youtube.com/live_chat?v={}",
                self.video_id
            ))
            .unwrap();

            tab.enable_response_handling(Box::new(move |a, b| self.on_response(a, b)))
                .unwrap();

            (browser, tab)
        });

        // prevent to run destructors
        Empty.await;
    }

    fn on_response(
        &self,
        param: ResponseReceivedEventParams,
        fetch: &dyn Fn() -> Result<GetResponseBodyReturnObject, failure::Error>,
    ) {
        let id = param.request_id;

        if !param
            .response
            .url
            .starts_with("https://www.youtube.com/youtubei/v1/live_chat/get_live_chat")
        {
            return;
        }

        const POLL_COUNT: usize = 10;
        const POLL_INTERVAL: Duration = Duration::from_millis(500);

        // TODO: use tracing's span
        // Wait for chromium to fetch response's body
        // we don't have a way to be notified that chromium completed to fetch,
        // so using polling instead.
        'poll: for _ in 0..POLL_COUNT {
            sleep(POLL_INTERVAL);
            match fetch() {
                Ok(data) => {
                    if data.body.trim().is_empty() {
                        tracing::warn!("id: {}: empty body. retrying", id);
                        continue 'poll;
                    }

                    self.on_comment(&data.body);
                    return;
                }

                Err(e) => {
                    tracing::error!("id: {}: failed to fetch: {}", id, e);
                    return;
                }
            }
        }

        tracing::warn!("couldn't fetch request's body");
    }

    fn on_comment(&self, json_str: &str) {
        self.ctx.rt.block_on(inner(self, json_str));

        // inner function stands for using questionmark operator
        async fn inner(me: &YoutubeListener, json_str: &str) -> Option<()> {
            let raw = serde_json::from_str::<serde_json::Value>(json_str).unwrap();

            let comment_iter = raw
                .get("continuationContents")?
                .get("liveChatContinuation")?
                .get("actions")?
                .as_array()?
                .iter()
                .flat_map(|x| {
                    Some(
                        x.get("addChatItemAction")?
                            .get("item")?
                            .get("liveChatTextMessageRenderer")?,
                    )
                })
                .flat_map(|x| {
                    Some(Comment {
                        author_name: x.get("authorName")?.get("simpleText")?.as_str()?,
                        author_icon: x
                            .get("authorPhoto")?
                            .get("thumbnails")?
                            .as_array()?
                            .last()?
                            .get("url")?
                            .as_str()?,
                        content: x
                            .get("message")?
                            .get("runs")?
                            .as_array()?
                            .first()?
                            .get("text")?
                            .as_str()?,
                    })
                });

            for comment in comment_iter {
                match me.ctx.webview_chan.lock().await.as_ref() {
                    Some(chan) => {
                        chan.send(ScreenAction::TimelinePush {
                            user: User {
                                icon: Some(comment.author_icon.to_string()),
                                ident: None,
                                name: comment.author_name.to_string(),
                            },
                            service: Service::Youtube,
                            content: comment.content.to_string(),
                        })
                        .await
                        .ok();
                    }

                    None => {
                        tracing::warn!(
                            "couldn't send youtube event because WebView was not initialized.",
                        );
                    }
                };
            }

            None
        }
    }
}
