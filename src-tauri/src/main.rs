#![allow(dead_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// TODO: replace all pub -> pub(crate)

#[macro_use] // for macro
extern crate diesel;

mod discord;
mod model;
mod schema;

use {
    crate::{discord::DiscordListener, model::ScreenAction},
    anyhow::{Context as _, Result},
    diesel::{Connection, SqliteConnection},
    std::{result::Result as StdResult, sync::Arc},
    tauri::{Webview, WebviewMut},
    tokio::{
        runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime},
        sync::{
            mpsc::{channel, Sender},
            Mutex,
        },
    },
};

struct Context {
    rt: TokioRuntime,
    db: Mutex<SqliteConnection>,
    webview_chan: Mutex<Option<Sender<ScreenAction>>>,
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").context("failed to get DATABASE_URL")?;
    let discord_token = std::env::var("DISCORD_TOKEN").context("failed to get DISCORD_TOKEN")?;
    let use_ansi = std::env::var("NO_COLOR").is_err();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(use_ansi)
        .init();

    let rt = TokioRuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to create tokio runtime")?;

    let db = SqliteConnection::establish(&database_url).context("failed to open database")?;

    let ctx = Arc::new(Context {
        rt,
        db: Mutex::new(db),
        webview_chan: Mutex::new(None),
    });

    {
        let my_ctx = Arc::clone(&ctx);
        ctx.rt.spawn(async move {
            DiscordListener::new(my_ctx)
                .start(&discord_token)
                .await
                .context("failed to start discord listener")
                .unwrap();
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
        .block_on(async { *ctx.webview_chan.lock().await = Some(tx) });

    ctx.rt.spawn(async move {
        while let Some(action) = rx.recv().await {
            tauri::event::emit(&mut webview, "event", Some(model::serialize(action))).unwrap();
        }
    });
}

#[allow(clippy::unnecessary_wraps)]
fn on_client_message(_webview: &mut Webview, _message: &str) -> StdResult<(), String> {
    Ok(())
}
