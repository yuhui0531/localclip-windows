#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--clipboard-read") {
        let _ = localclip_app::win::clipboard::worker_main();
        return;
    }
    if localclip_app::desktop::run().is_err() {
        std::process::exit(1);
    }
}
#[cfg(not(windows))]
fn main() {
    eprintln!("LocalClip requires Windows.");
}
