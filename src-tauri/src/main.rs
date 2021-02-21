#![allow(dead_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[macro_use] // for macro
extern crate diesel;

mod discord;
mod model;
mod schema;

use {
    crate::discord::DiscordListener,
    anyhow::{Context as _, Result},
    std::{result::Result as StdResult, sync::Arc},
    tauri::{Webview, WebviewMut},
    tokio::{
        runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime},
        sync::mpsc::channel,
    },
};

struct Context {
    rt: TokioRuntime,
    discord_token: String,
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();

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

    let ctx = Arc::new(Context { rt, discord_token });

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

    let token = ctx.discord_token.clone();
    ctx.rt.spawn(async move {
        DiscordListener::new(tx).start(&token).await.unwrap();
    });

    ctx.rt.spawn(async move {
        while let Some(action) = rx.recv().await {
            tauri::event::emit(&mut webview, "event", Some(model::serialize(action))).unwrap();
        }
    });
}

fn on_client_message(_webview: &mut Webview, _message: &str) -> StdResult<(), String> {
    Ok(())
}
