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
    std::{path::Path, sync::Arc},
    tokio::{
        runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime},
        sync::{
            mpsc::{channel, Sender},
            RwLock,
        },
    },
};

use tauri::{App, Manager, Runtime};

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
    });

    std::mem::forget(Arc::clone(&ctx));

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

    tauri::Builder::default()
        .setup(move |x| {
            setup(Arc::clone(&ctx), x);
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap();

    Ok(())
}

fn setup<R: Runtime>(ctx: Arc<Context>, app: &App<R>) {
    let (tx, mut rx) = channel(10);

    ctx.rt
        .block_on(tx.send(ScreenAction::UpcomingPresentationsUpdate(Arc::clone(&ctx))))
        .ok();

    ctx.rt
        .block_on(async { *ctx.webview_chan.write().await = Some(tx) });

    let win = app.get_window("main").unwrap();

    ctx.rt.spawn(async move {
        while let Some(action) = rx.recv().await {
            win.emit("event", action.serialize().await).unwrap();
        }
    });
}
