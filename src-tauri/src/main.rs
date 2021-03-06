#![allow(dead_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// TODO: replace all pub -> pub(crate)

mod client;
mod model;
mod presentations;

use {
    crate::{model::ScreenAction, presentations::Presentations},
    anyhow::{Context as _, Result},
    std::{result::Result as StdResult, sync::Arc},
    tauri::{Webview, WebviewMut},
    tokio::{
        runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime},
        sync::{
            mpsc::{channel, Sender},
            RwLock,
        },
    },
};

struct Context {
    rt: TokioRuntime,
    webview_chan: RwLock<Option<Sender<ScreenAction>>>,
    presentations: RwLock<Presentations>,
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

    let presentations = Presentations::new();
    // rt
    //     .block_on(Presentations::load_from_file(Path::new(
    //         "./presentations.yaml",
    //     )))
    //     .context("failed to load presentations.yaml")?;

    let ctx = Arc::new(Context {
        rt,
        webview_chan: RwLock::new(None),
        presentations: RwLock::new(presentations),
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
        use egg_mode::{KeyPair, Token};

        let twitter_token = Token::Access {
            consumer: KeyPair {
                key: env_var("TWITTER_CONSUMER_KEY").into(),
                secret: env_var("TWITTER_CONSUMER_SECRET").into(),
            },
            access: KeyPair {
                key: env_var("TWITTER_ACCESS_KEY").into(),
                secret: env_var("TWITTER_ACCESS_SECRET").into(),
            },
        };

        let twitter_ctx = Arc::clone(&ctx);

        ctx.rt.spawn(async move {
            TwitterListener::new(twitter_ctx, twitter_token)
                .start()
                .await;
        });
    }

    #[cfg(feature = "youtube")]
    {
        use crate::client::youtube::YoutubeListener;

        let my_ctx = Arc::clone(&ctx);
        ctx.rt.spawn(async move {
            // TODO: replace video id
            YoutubeListener::new(my_ctx, "5VoIGGMYrDg".to_string())
                .start()
                .await;
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
