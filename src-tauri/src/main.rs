#![feature(or_patterns)]
#![allow(dead_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// TODO: replace all pub -> pub(crate)

mod client;
mod model;
mod presentations;

#[cfg(feature = "obs")]
mod obs;

use {
    crate::{model::ScreenAction, presentations::Presentations},
    anyhow::{Context as _, Result},
    std::{path::Path, result::Result as StdResult, sync::Arc},
    tauri::{Webview, WebviewMut},
    tokio::{
        runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime},
        sync::{
            mpsc::{channel, Sender},
            RwLock,
        },
    },
};

#[cfg(feature = "obs")]
use crate::obs::ObsAction;

struct SnsInfo {
    youtube_stream_url: String,
    discord_invitation_url: String,
}

struct Context {
    rt: TokioRuntime,
    sns_info: SnsInfo,
    webview_chan: RwLock<Option<Sender<ScreenAction>>>,
    presentations: RwLock<Presentations>,

    #[cfg(feature = "obs")]
    obs_chan: RwLock<Option<Sender<ObsAction>>>,

    #[cfg(feature = "twitter")]
    twitter_credentials: egg_mode::Token,
}

fn env_var(name: &str) -> String {
    match std::env::var(name) {
        Ok(v) => v,
        Err(_) => panic!("environment variable {} not set", name),
    }
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let use_ansi = std::env::var("NO_COLOR").is_err();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(use_ansi)
        .init();

    let rt = TokioRuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;

    let presentations = rt
        .block_on(Presentations::load_from_file(Path::new(
            "./presentations.yaml",
        )))
        .context("failed to load presentations.yaml")?;

    let youtube_stream_url = env_var("YOUTUBE_STREAM_URL");
    let discord_invitation_url = env_var("DISCORD_INVITATION_URL");

    #[cfg(feature = "twitter")]
    let twitter_token = egg_mode::Token::Access {
        consumer: egg_mode::KeyPair {
            key: env_var("TWITTER_CONSUMER_KEY").into(),
            secret: env_var("TWITTER_CONSUMER_SECRET").into(),
        },
        access: egg_mode::KeyPair {
            key: env_var("TWITTER_ACCESS_KEY").into(),
            secret: env_var("TWITTER_ACCESS_SECRET").into(),
        },
    };

    let ctx = Arc::new(Context {
        rt,
        webview_chan: RwLock::new(None),
        presentations: RwLock::new(presentations),

        sns_info: SnsInfo {
            youtube_stream_url,
            discord_invitation_url,
        },

        #[cfg(feature = "obs")]
        obs_chan: RwLock::new(None),

        #[cfg(feature = "twitter")]
        twitter_credentials: twitter_token,
    });

    #[cfg(feature = "discord")]
    {
        use crate::client::discord::DiscordListener;
        let discord_token = env_var("DISCORD_TOKEN");

        let my_ctx = Arc::clone(&ctx);

        ctx.rt.spawn(async move {
            DiscordListener::new(my_ctx)
                .start(&discord_token)
                .await
                .context("failed to start discord listener")
                .unwrap();
        });
    }

    #[cfg(feature = "twitter")]
    {
        use crate::client::twitter::TwitterListener;

        let twitter_ctx = Arc::clone(&ctx);
        let cred = ctx.twitter_credentials.clone();

        ctx.rt.spawn(async move {
            TwitterListener::new(twitter_ctx, cred).start().await;
        });
    }

    #[cfg(feature = "youtube")]
    {
        use crate::client::youtube::YoutubeListener;

        let youtube_ctx = Arc::clone(&ctx);
        ctx.rt.spawn(async move {
            // TODO: replace video id
            YoutubeListener::new(youtube_ctx, "5VoIGGMYrDg".to_string())
                .start()
                .await;
        });
    }

    #[cfg(feature = "obs")]
    {
        use crate::obs::ObsClient;

        let addr = env_var("OBS_ADDRESS");
        let pass = env_var("OBS_PASS");
        let port = env_var("OBS_PORT")
            .parse()
            .context("failed to decode OBS_PORT")?;

        let (tx, rx) = channel(10);

        ctx.rt
            .block_on(async { *ctx.obs_chan.write().await = Some(tx) });

        ctx.rt.spawn(async move {
            ObsClient::connect(&addr, port, &pass)
                .await
                .context("failed to initialize ObsClient")
                .unwrap()
                .start(rx)
                .await
                .context("error occur while running ObsClient")
                .unwrap()
        });
    }

    tauri::AppBuilder::new()
        .setup(move |a, b| setup(Arc::clone(&ctx), a, b))
        .invoke_handler(on_client_message)
        .build()
        .run();

    Ok(())
}

fn setup(ctx: Arc<Context>, webview: &mut Webview, _source: String) {
    let mut webview: WebviewMut = webview.as_mut();

    let (tx, mut rx) = channel(10);

    ctx.rt
        .block_on(tx.send(ScreenAction::UpcomingPresentationsUpdate(Arc::clone(&ctx))))
        .ok();

    ctx.rt
        .block_on(async { *ctx.webview_chan.write().await = Some(tx) });

    ctx.rt.spawn(async move {
        while let Some(action) = rx.recv().await {
            tauri::event::emit(&mut webview, "event", Some(action.serialize().await)).unwrap();
        }
    });
}

#[allow(clippy::unnecessary_wraps)]
fn on_client_message(_webview: &mut Webview, _message: &str) -> StdResult<(), String> {
    Ok(())
}
