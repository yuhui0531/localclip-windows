#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--uninstall-history-confirmed") {
        // This flag is used only by the MSI's explicitly checked removal option.
        let Some(local) = std::env::var_os("LOCALAPPDATA") else {
            std::process::exit(1);
        };
        let history = std::path::PathBuf::from(local).join("LocalClip/History");
        let result = if history.exists() {
            std::fs::remove_dir_all(history)
        } else {
            Ok(())
        };
        std::process::exit(if result.is_ok() { 0 } else { 1 });
    }
    if std::env::args().nth(1).as_deref() == Some("--clipboard-read") {
        let _ = localclip_app::win::clipboard::worker_main();
        return;
    }
    if let Err(error) = localclip_app::desktop::run() {
        use windows::{
            Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW},
            core::w,
        };
        let message = if error.to_string().contains("incompatibleSchema") {
            w!("数据版本不兼容。LocalClip 没有修改或覆盖历史，请使用兼容版本。")
        } else {
            w!(
                "LocalClip 无法启动。请确认数据目录可写、磁盘空间充足，并已安装 Microsoft Edge WebView2 运行时。历史数据不会被覆盖。"
            )
        };
        unsafe {
            let _ = MessageBoxW(None, message, w!("LocalClip"), MB_OK | MB_ICONERROR);
        }
        std::process::exit(1);
    }
}
#[cfg(not(windows))]
fn main() {
    eprintln!("LocalClip requires Windows.");
}
