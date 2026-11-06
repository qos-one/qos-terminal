mod api;
mod calc;
mod app;
mod ui;

#[cfg(feature = "desktop")]
use dioxus::prelude::*;

use app::App;

/// macOS: set the Dock icon at runtime and copy icon.icns into the .app bundle dx builds (Info.plist references it but dx serve does not copy it)
#[cfg(all(feature = "desktop", target_os = "macos"))]
fn install_mac_icon() {
    use objc2::{AnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;
    const ICNS: &[u8] = include_bytes!("../assets/icon/qos.icns");
    const PNG: &[u8] = include_bytes!("../assets/icon/icon_512.png");
    if let Ok(exe) = std::env::current_exe() {
        if let Some(res) = exe.parent().and_then(|p| p.parent()).map(|c| c.join("Resources")) {
            if res.is_dir() {
                let target = res.join("icon.icns");
                if std::fs::read(&target).map(|cur| cur != ICNS).unwrap_or(true) {
                    let _ = std::fs::write(&target, ICNS);
                }
            }
        }
    }
    if let Some(mtm) = MainThreadMarker::new() {
        let app = NSApplication::sharedApplication(mtm);
        let data = NSData::with_bytes(PNG);
        #[allow(unused_unsafe)]
        unsafe {
            if let Some(img) = NSImage::initWithData(NSImage::alloc(), &data) {
                app.setApplicationIconImage(Some(&img));
            }
        }
    }
}

fn main() {
    #[cfg(all(feature = "desktop", target_os = "macos"))]
    install_mac_icon();
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
        let window = WindowBuilder::new()
            .with_title("QOS Terminal")
            .with_inner_size(LogicalSize::new(1680.0, 1040.0))
            .with_min_inner_size(LogicalSize::new(1180.0, 760.0));
        // window/Dock icon (QX from the logo) — RGBA 64x64 embedded in the binary
        let icon = dioxus::desktop::tao::window::Icon::from_rgba(include_bytes!("../assets/icon/window_icon_64.rgba").to_vec(), 64, 64).ok();
        let mut cfg = Config::new().with_window(window).with_menu(None);
        if let Some(i) = icon { cfg = cfg.with_icon(i); }
        dioxus::LaunchBuilder::new()
            .with_cfg(desktop!(cfg))
            .launch(App);
    }
    #[cfg(not(feature = "desktop"))]
    dioxus::launch(App);
}

/// sleep for polling loops (the web demo in ../demo has its own sleep_ms)
#[cfg(any(feature = "desktop", feature = "mobile"))]
pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}
