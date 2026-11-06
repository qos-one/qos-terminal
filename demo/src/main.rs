//! QOS Terminal web demo: the same UI as the desktop Terminal (../src), built for the browser.
//! Server functions are replaced by plain HTTP calls to the demo backend (cloud/demo), which serves simulated data.
#![allow(dead_code, unused_mut)] // shared sources are tuned for the desktop build
#[path = "../../src/api.rs"]
mod api;
#[path = "../../src/calc.rs"]
mod calc;
#[path = "../../src/app.rs"]
mod app;
#[path = "../../src/ui.rs"]
mod ui;

fn main() {
    dioxus::launch(app::App);
}

/// sleep for polling loops in the browser
pub async fn sleep_ms(ms: u64) {
    gloo_timers::future::TimeoutFuture::new(ms.min(u32::MAX as u64) as u32).await;
}
