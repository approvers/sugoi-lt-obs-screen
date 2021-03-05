use std::sync::Arc;

use {
    crate::{
        model::{Page, ScreenAction, Service, User},
        Context,
    },
    anyhow::{Context as _, Result},
    lazy_static::lazy_static,
    parking_lot::RwLock,
    regex::Regex,
    serenity::{
        async_trait,
        model::{channel::Message, prelude::Ready},
        prelude::{Client, Context as SerenityContext, EventHandler},
    },
};

const PREFIX: &str = "g!live";

fn extract_user_id_from_mention(mention_text: &str) -> Option<u64> {
    lazy_static! {
        static ref MENTION_REGEX: Regex = Regex::new(r#"<@(?P<id>\d+)>"#).unwrap();
    }

    let id_str = MENTION_REGEX
        .captures(mention_text)?
        .name("id")
        .unwrap()
        .as_str();

    Some(id_str.parse().ok()?)
}

#[test]
fn test_extract_user_id() {
    assert_eq!(extract_user_id_from_mention("<@123>"), Some(123));
    assert_eq!(extract_user_id_from_mention("<@012345>"), Some(12345));
    assert_eq!(extract_user_id_from_mention("hogehoge"), None);
}

enum Command<'a> {
    Help(Option<&'static str>), // additional error message if available
    Listen,
    StopListening,
    SetNotification(String),
    TimelineClear,
    Pause,
    Resume,
    Presentation(PresentationCommand<'a>),
}

enum PresentationCommand<'a> {
    Push {
        user_mention: &'a str,
        title: String,
    },
    Reorder {
        map: Vec<usize>,
    },
    Delete {
        index: usize,
    },
    Update {
        index: usize,
        new_title: &'a str,
    },
    List,
    Pop,
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

    fn parse<'a>(&self, msg: &'a str) -> Option<Command<'a>> {
        let mut tokens = msg.split(' ');

        let prefix = tokens.next();
        let sub_command = tokens.next();
        let args = tokens.collect::<Vec<_>>();

        use Command::*;
        use PresentationCommand::*;

        match (prefix, sub_command, args.as_slice()) {
            (None, _, _) => None,
            (Some(p), _, _) if p != PREFIX => None,

            (_, Some("pause"), _) => Some(Pause),
            (_, Some("resume"), _) => Some(Resume),
            (_, Some("listen"), _) => Some(Listen),
            (_, Some("stop_listening"), _) => Some(StopListening),
            (_, Some("clear_timeline"), _) => Some(TimelineClear),

            (_, Some("set_notification"), []) => {
                Some(Help(Some("set_notification requires argument")))
            }

            (_, Some("set_notification"), args) => Some(SetNotification(args.join(" "))),

            (_, Some("presentations"), ["pop", ..]) => Some(Presentation(Pop)),
            (_, Some("presentations"), ["list", ..]) => Some(Presentation(List)),

            (_, Some("presentations"), ["push", user_mention, title @ ..]) if !title.is_empty() => {
                Some(Presentation(Push {
                    user_mention,
                    title: title.join(" "),
                }))
            }

            (_, Some("presentations"), ["push", ..]) => Some(Help(Some(
                "presentations push command requires >= 2 arguments",
            ))),

            (_, Some("presentations"), ["reorder", map @ ..]) => {
                let parsed_map = map.iter().map(|x| x.parse()).collect::<Result<_, _>>();

                if let Err(_) = parsed_map {
                    return Some(Help(Some(
                        "presentations reorder command's arguments must be valid usize",
                    )));
                }

                Some(Presentation(Reorder {
                    map: parsed_map.unwrap(),
                }))
            }

            (_, Some("presentations"), ["delete", index, ..]) => match index.parse() {
                Ok(index) => Some(Presentation(Delete { index })),
                Err(_) => Some(Help(Some(
                    "presentations delete command's argument must be valid usize",
                ))),
            },

            (_, Some("presentations"), ["update", index, new_title, ..]) => match index.parse() {
                Ok(index) => Some(Presentation(Update { index, new_title })),
                Err(_) => Some(Help(Some(
                    "presentations update command's first argument must be valid usize",
                ))),
            },

            (_, _, _) => Some(Help(Some("unknown subcommand"))),
        }
    }

    async fn invoke_command(&self, ctx: &SerenityContext, message: &Message, cmd: Command<'_>) {
        use Command::*;
        use PresentationCommand::*;

        let text_buffer;
        let text = match (cmd, self.ctx.webview_chan.lock().await.as_ref()) {
            (Help(None), _) => "https://hackmd.io/@U9f9Fv6rTt2UkRA6UriFTA/BJRVQlTZO",

            (Help(Some(hint)), _) => {
                text_buffer = format!(
                    "{}\nhttps://hackmd.io/@U9f9Fv6rTt2UkRA6UriFTA/BJRVQlTZO",
                    hint
                );

                text_buffer.as_str()
            }

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

            (Presentation(List), _) => unimplemented!(),

            (Presentation(Pop), _) => unimplemented!(),

            #[allow(unused_variables)]
            (Presentation(Reorder { map }), _) => unimplemented!(),

            #[allow(unused_variables)]
            (Presentation(Delete { index }), _) => unimplemented!(),

            #[allow(unused_variables)]
            (Presentation(Update { index, new_title }), _) => unimplemented!(),

            #[allow(unused_variables)]
            (
                Presentation(Push {
                    user_mention,
                    title,
                }),
                _,
            ) => unimplemented!(),

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
