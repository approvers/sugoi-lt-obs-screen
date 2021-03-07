use {
    crate::{
        model::{Page, ScreenAction, Service, User},
        Context,
    },
    anyhow::{Context as _, Result},
    async_trait::async_trait,
    lazy_static::lazy_static,
    parking_lot::RwLock,
    regex::Regex,
    serenity::{
        model::{channel::Message, id::UserId, prelude::Ready, user::User as SerenityUser},
        prelude::{Client, Context as SerenityContext, EventHandler},
    },
    std::{future::Future, pin::Pin, sync::Arc},
    tokio::sync::mpsc::Sender,
};

#[cfg(feature = "obs")]
use crate::obs::ObsAction;

const PREFIX: &str = "g!live";

fn extract_user_id_from_mention(mention_text: &str) -> Option<u64> {
    lazy_static! {
        static ref MENTION_REGEX: Regex = Regex::new(r#"<@!(?P<id>\d+)>"#).unwrap();
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
    assert_eq!(extract_user_id_from_mention("<@!123>"), Some(123));
    assert_eq!(extract_user_id_from_mention("<@!012345>"), Some(12345));
    assert_eq!(extract_user_id_from_mention("hogehoge"), None);
}

macro_rules! block {
    ($b:block) => {
        loop {
            break $b;
        }
    };
}

/// Option<impl Future<Output = U>> -> Option<U>
pub(crate) trait OptionFutExt<O> {
    fn map_await<'a>(self) -> Pin<Box<dyn Future<Output = Option<O>> + Send + 'a>>
    where
        Self: 'a;
}

impl<I, O> OptionFutExt<O> for Option<I>
where
    I: Future<Output = O> + Send,
{
    fn map_await<'a>(self) -> Pin<Box<dyn Future<Output = Option<O>> + Send + 'a>>
    where
        Self: 'a,
    {
        Box::pin(async {
            match self {
                Some(t) => Some(t.await),
                None => None,
            }
        })
    }
}

enum Command<'a> {
    Help(Option<&'a str>), // additional error message if available
    Listen,
    StopListening,
    SetNotification(String),
    TimelineClear,
    Pause,
    Resume,
    Presentation(PresentationCommand<'a>),
    Tweet {
        with_youtube_footer: bool,
        with_discord_footer: bool,
        with_twitter_footer: bool,
        msg: String,
        simulation: bool,
    },
}

enum PresentationCommand<'a> {
    Push {
        user_mention: &'a str,
        title: String,
    },
    Reorder {
        map: Vec<usize>,
    },
    Remove {
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
        let mut tokens = msg.split(' ').filter(|x| !x.trim().is_empty());

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

                if parsed_map.is_err() {
                    return Some(Help(Some(
                        "presentations reorder command's arguments must be valid usize",
                    )));
                }

                Some(Presentation(Reorder {
                    map: parsed_map.unwrap(),
                }))
            }

            (_, Some("presentations"), ["remove", index, ..]) => match index.parse() {
                Ok(index) => Some(Presentation(Remove { index })),
                Err(_) => Some(Help(Some(
                    "presentations delete command's argument must be valid usize",
                ))),
            },

            (_, Some("presentations"), ["remove", ..]) => Some(Help(Some(
                "presentations delete command requires at least 1 arguments",
            ))),

            (_, Some("presentations"), ["update", index, new_title, ..]) => match index.parse() {
                Ok(index) => Some(Presentation(Update { index, new_title })),
                Err(_) => Some(Help(Some(
                    "presentations update command's first argument must be valid usize",
                ))),
            },

            (_, Some("presentations"), ["update", ..]) => Some(Help(Some(
                "presentations update command requires at least 2 arguments",
            ))),

            (_, cmd @ (Some("tweet") | Some("tweet-simulation")), [flags, body @ ..])
                if flags.starts_with("-") && !body.is_empty() =>
            {
                let mut with_youtube_footer = false;
                let mut with_discord_footer = false;
                let mut with_twitter_footer = false;

                // skip -
                for c in flags.chars().skip(1) {
                    match c.to_ascii_lowercase() {
                        't' => with_twitter_footer = true,
                        'y' => with_youtube_footer = true,
                        'd' => with_discord_footer = true,
                        _ => return Some(Help(Some("unknown footer flag. supported flags are d: Discord, y: Youtube, t: Twitter")))
                    }
                }

                let msg = body.join(" ");

                if !(msg.starts_with("```") && msg.ends_with("```")) {
                    return Some(Help(Some(
                        "Tweet body must be covered with codeblock to avoid mention issues.",
                    )));
                }

                let msg_len = msg.chars().count();
                let msg = msg
                    .chars()
                    .skip("```".len())
                    .take(msg_len - ("```".len() * 2))
                    .collect::<String>()
                    .trim()
                    .to_string();

                Some(Tweet {
                    with_youtube_footer,
                    with_discord_footer,
                    with_twitter_footer,
                    msg,
                    simulation: cmd == Some("tweet-simulation"),
                })
            }

            (_, cmd @ (Some("tweet") | Some("tweet-simulation")), [body @ ..])
                if !body.is_empty() =>
            {
                Some(Tweet {
                    with_youtube_footer: false,
                    with_discord_footer: false,
                    with_twitter_footer: false,
                    msg: body.join(" "),
                    simulation: cmd == Some("tweet-simulation"),
                })
            }

            (_, Some("tweet"), _) => Some(Help(Some("tweet command requires argument"))),

            (_, _, _) => Some(Help(Some("unknown subcommand"))),
        }
    }

    async fn update_presentations(&self, sender: &Sender<ScreenAction>) {
        let ctx = Arc::clone(&self.ctx);

        sender
            .send(ScreenAction::UpcomingPresentationsUpdate(ctx))
            .await
            .ok();
    }

    async fn invoke_command(&self, ctx: &SerenityContext, message: &Message, cmd: Command<'_>) {
        use Command::*;
        use PresentationCommand::*;

        let text_buffer;
        let text = match (cmd, self.ctx.webview_chan.read().await.as_ref()) {
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

                #[cfg(feature = "obs")]
                match self.ctx.obs_chan.read().await.as_ref() {
                    Some(obs_chan) => {
                        obs_chan.send(ObsAction::Mute).await.ok();
                    }

                    None => {
                        tracing::warn!(
                            "failed to mute stream because obs_channel was not initialized"
                        );
                    }
                }

                "switching requested"
            }

            (Resume, Some(sender)) => {
                sender
                    .send(ScreenAction::SwitchPage(Page::LTScreen))
                    .await
                    .ok();

                #[cfg(feature = "obs")]
                match self.ctx.obs_chan.read().await.as_ref() {
                    Some(obs_chan) => {
                        obs_chan.send(ObsAction::UnMute).await.ok();
                    }

                    None => {
                        tracing::warn!(
                            "failed to unmute stream because obs_channel was not initialized"
                        );
                    }
                }

                "switching requested"
            }

            (Presentation(List), _) => {
                let mut list = self.ctx.presentations.read().await.list();

                // make list codeblock
                list.insert_str(0, "```\n");
                list.push_str("\n```");

                text_buffer = list;
                text_buffer.as_str()
            }

            (Presentation(Pop), Some(sender)) => block! {{
                let popped = self.ctx.presentations.write().await.pop().await;

                if popped.is_none() {
                    break "no other entries in queue";
                }

                let popped = popped.unwrap();

                self.update_presentations(sender).await;

                sender
                    .send(ScreenAction::PresentationUpdate {
                        presenter: popped.presenter,
                        title: popped.title,
                    })
                    .await
                    .ok();


                // TODO: introduce command
                "popped(removed an entry at 0 index and updated ongoing presentation)\nDO NOT FORGET TO TWEET!"
            }},

            #[allow(unused_variables)]
            (Presentation(Reorder { map }), _) => "unimplemented",

            (Presentation(Remove { index }), Some(sender)) => {
                let deleted = self.ctx.presentations.write().await.remove(index).await;

                if deleted {
                    self.update_presentations(sender).await;
                    "removed"
                } else {
                    "not found such entry"
                }
            }

            (Presentation(Update { index, new_title }), Some(sender)) => {
                let mut lock = self.ctx.presentations.write().await;
                let present = lock.get_mut(index);

                match present {
                    Some(p) => {
                        p.title = new_title.to_string();
                        self.update_presentations(sender).await;
                        "overwrote"
                    }

                    None => "not found such entry",
                }
            }

            (
                Presentation(Push {
                    user_mention,
                    title,
                }),
                Some(sender),
            ) => block! {{
                let uid = match extract_user_id_from_mention(user_mention) {
                    Some(id) => id,
                    None => break "1st argument must be user mention"
                };

                let user = match UserId(uid).to_user(&ctx).await {
                    Ok(u) => u,
                    Err(e) => {
                        tracing::error!("failed to fetch user info. id: {}, err: {}", uid, e);
                        break "couldn't get user info. please check user mention is correct. read log for more info."
                    }
                };

                let name = message
                    .guild_id
                    .map(|gid| user.nick_in(&ctx.http, gid))
                    .map_await()
                    .await
                    .and_then(|x| x);

                let name = name.as_ref().unwrap_or(&user.name);

                self.ctx
                    .presentations
                    .write()
                    .await
                    .push(crate::presentations::Presentation {
                        presenter: User {
                            icon: user.avatar_url(),
                            ident: None,
                            name: name.into(),
                        },
                        title,
                    }).await;

                self.update_presentations(sender).await;

                "pushed"
            }},

            (
                Tweet {
                    with_youtube_footer,
                    with_discord_footer,
                    with_twitter_footer,
                    msg,
                    simulation,
                },
                _,
            ) => block! {{
                let mut message = msg.to_string();

                let has_footer = with_youtube_footer || with_discord_footer || with_twitter_footer;

                if has_footer {
                    message.push_str("\n");
                }

                if with_youtube_footer {
                    message.push('\n');

                    message.push_str(&format!(
                        include_str!("tweet_template/footer/youtube.fmt.txt"),
                        YOUTUBE_URL = self.ctx.sns_info.youtube_stream_url
                    ));
                }

                if with_discord_footer {
                    message.push('\n');

                    message.push_str(&format!(
                        include_str!("tweet_template/footer/discord.fmt.txt"),
                        DISCORD_INVITATION_URL = self.ctx.sns_info.discord_invitation_url
                    ));
                }

                if with_twitter_footer {
                    message.push('\n');
                    message.push_str(include_str!("tweet_template/footer/twitter.txt"));
                }

                let tweet_len: u32 = message
                    .chars()
                    .map(|x| if x.is_ascii() { 1 } else { 2 })
                    .sum();

                if tweet_len > 280 {
                    text_buffer = format!("Tweet length is longer than 280({}). Shorten the message or the footer.", tweet_len);
                    break text_buffer.as_str();
                }

                if !simulation {
                    let link = match self.tweet(&message).await {
                        Ok(Some(link)) => link,
                        Ok(None) => "unavailable".to_string(),

                        Err(e) => {
                            tracing::error!("failed to tweet: {:?}", e);
                            break "failed to tweet. read log for more details.";
                        }
                    };

                    message = format!("Tweeted.\nlink: {}\nbody:\n```\n{}\n```", link, message);
                } else {
                    message = format!("Tweet simulation.\nbody:\n```\n{}\n```", message);
                };

                text_buffer = message;
                text_buffer.as_str()
            }},

            (_, None) => "webview was not ready",
        };

        if let Err(e) = message.channel_id.say(&ctx, text).await {
            tracing::error!("failed to send message!: {:?}\n{}", e, &text);
        }
    }

    #[rustfmt::skip]
    async fn can_invoke_command(&self, ctx: &SerenityContext, user: &SerenityUser) -> Result<bool> {
        const LT_SERVER_GUILD_ID: u64 = 813469320680177715;
        const LT_SERVER_ORGANIZER_ROLE_ID: u64 = 813469405077831710;
        const LT_SERVER_OPERATOR_ROLE_ID: u64 = 813469837711900742;

        Ok(
            user
                .has_role(&ctx, LT_SERVER_GUILD_ID, LT_SERVER_ORGANIZER_ROLE_ID)
                .await? ||
            user
                .has_role(&ctx, LT_SERVER_GUILD_ID, LT_SERVER_OPERATOR_ROLE_ID)
                .await?
        )
    }

    // TODO: tweet function should not be here
    /// returns tweet link if available
    async fn tweet(&self, msg: &str) -> Result<Option<String>> {
        #[cfg(feature = "twitter")]
        {
            let result = egg_mode::tweet::DraftTweet::new(msg.to_string())
                .send(&self.ctx.twitter_credentials)
                .await
                .context("failed to tweet")?;

            Ok(Some(format!("https://twitter.com/_/status/{}/", result.id)))
        }

        #[cfg(not(feature = "twitter"))]
        {
            tracing::warn!("Tweet simulation:\n{}", msg);
            Ok(None)
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
            let should_run_cmd = match self.can_invoke_command(&ctx, &message.author).await {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        "failed to check whether user {} can invoke a command: {}",
                        message.author.name,
                        e
                    );
                    false
                }
            };

            if should_run_cmd {
                self.invoke_command(&ctx, &message, cmd).await;
                return;
            }
        }

        let listening_channel_id = self.inner.read().listening_channel_id.clone();

        if let Some(target_id) = listening_channel_id {
            if target_id != message.channel_id.0 {
                return; // TODO:
            }

            match self.ctx.webview_chan.read().await.as_ref() {
                Some(chan) => {
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

                None => tracing::warn!(
                    "failed to send TimelinePush event because Webview was not initialized"
                ),
            }
        }
    }
}
