#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::time::Duration;
use tauri::{Webview, WebviewMut};
use tokio::runtime::{Builder as TokioRuntimeBuilder, Runtime as TokioRuntime};

mod cmd;

fn main() {
    let mut rt = TokioRuntimeBuilder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    tauri::AppBuilder::new()
        .setup(move |a, b| setup(&mut rt, a, b))
        .invoke_handler(on_client_message)
        .build()
        .run();
}

fn setup(rt: &mut TokioRuntime, webview: &mut Webview, _source: String) {
    let mut webview: WebviewMut = webview.as_mut();

    rt.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        loop {
            interval.tick().await;
            tauri::event::emit(&mut webview, "test", Some("hogehoge")).unwrap();
            println!("emit!");
        }
    });
}

fn on_client_message(_webview: &mut Webview, _message: &str) -> Result<(), String> {
    Ok(())
}
