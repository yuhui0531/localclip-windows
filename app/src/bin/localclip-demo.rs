#[cfg(windows)]
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--clipboard-read") {
        let _ = localclip_app::win::clipboard::worker_main();
        return;
    }
    if let Err(reason) = run() {
        eprintln!("status={reason}");
        std::process::exit(1);
    }
}
#[cfg(not(windows))]
fn main() {
    eprintln!("LocalClip system integration requires Windows.");
}

#[cfg(windows)]
fn run() -> Result<(), &'static str> {
    use localclip_app::{runtime::Runtime, win::source::Target};
    use std::sync::{Arc, atomic::AtomicBool};
    let args: Vec<_> = std::env::args().collect();
    let command = args
        .get(1)
        .map(String::as_str)
        .ok_or("usage: capture|paste <id>|ocr <id> --data-dir <isolated-directory>")?;
    if !["capture", "paste", "ocr"].contains(&command) {
        return Err("unsupportedCommand");
    }
    let arg = |key: &str| {
        args.iter()
            .position(|v| v == key)
            .and_then(|i| args.get(i + 1))
    };
    let root = std::path::PathBuf::from(arg("--data-dir").ok_or("dataDirectoryRequired")?);
    let target =
        unsafe { Target::capture(windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow()) };
    let runtime = Runtime::open(
        &root,
        Arc::new(|status| println!("capture-status={status}")),
    )?;
    let result = match command {
        "capture" => {
            println!("listener-ready");
            let seconds = arg("--seconds").and_then(|v| v.parse().ok()).unwrap_or(60);
            std::thread::sleep(std::time::Duration::from_secs(seconds));
            Ok(())
        }
        "paste" => {
            let id = args.get(2).ok_or("recordRequired")?;
            let payload = runtime.store.payload(id).map_err(|_| "notFound")?;
            let output = runtime.output(payload, target, true, true, || {})?;
            if matches!(output.status, "commandSent" | "copied") {
                runtime
                    .store
                    .touch(vec![id.clone()])
                    .map_err(|_| "storageUnavailable")?;
            }
            println!(
                "output-status={} missing-count={}",
                output.status, output.missing_files
            );
            Ok(())
        }
        "ocr" => {
            let id = args.get(2).ok_or("recordRequired")?;
            let languages = arg("--languages")
                .map(String::as_str)
                .unwrap_or("zh-Hans,zh-Hant,en-US")
                .split(',')
                .map(String::from)
                .collect::<Vec<_>>();
            let (lines, missing) = runtime.ocr(id, &languages, &AtomicBool::new(false))?;
            println!("ocr-lines={lines} missing-language-count={}", missing.len());
            Ok(())
        }
        _ => unreachable!(),
    };
    runtime.shutdown();
    println!("listener-stopped");
    result
}
