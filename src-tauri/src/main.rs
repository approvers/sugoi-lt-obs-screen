#![allow(dead_code)]
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use anyhow::{Context as _, Result};
use std::result::Result as StdResult;
use std::sync::Arc;
use std::time::Duration;
use tauri::{Webview, WebviewMut};
use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime};

mod cmd;
mod discord;
mod model;

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

    let rt = Arc::new(rt);
    let rt2 = Arc::clone(&rt);

    tauri::AppBuilder::new()
        .setup(move |a, b| setup(Arc::clone(&rt2), a, b))
        .invoke_handler(on_client_message)
        .build()
        .run();

    Ok(())
}

fn setup(rt: Arc<TokioRuntime>, webview: &mut Webview, _source: String) {
    let mut webview: WebviewMut = webview.as_mut();

    rt.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            tauri::event::emit(&mut webview, "test", Some("hogehoge")).unwrap();
        }
    });
}

fn on_client_message(_webview: &mut Webview, _message: &str) -> StdResult<(), String> {
    Ok(())
}
