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
    use localclip_app::win::listener::{Listener, now};
    use localclip_core::store::{HistoryStore, Ingest, Options, Snapshot};
    use std::sync::Arc;
    let args: Vec<_> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some("capture") {
        return Err("usage: capture --data-dir <isolated-directory> --seconds <duration>");
    }
    let arg = |key: &str| {
        args.iter()
            .position(|a| a == key)
            .and_then(|i| args.get(i + 1))
    };
    let root = std::path::PathBuf::from(arg("--data-dir").ok_or("dataDirectoryRequired")?);
    std::fs::create_dir_all(&root).map_err(|_| "accessUnavailable")?;
    let seconds: u64 = arg("--seconds").and_then(|v| v.parse().ok()).unwrap_or(60);
    let disk_root = root.clone();
    let store = Arc::new(
        HistoryStore::open(
            root.join("history.sqlite"),
            Options {
                now: Arc::new(now),
                available_space: Arc::new(move || {
                    use std::os::windows::ffi::OsStrExt;
                    use windows::{Win32::Storage::FileSystem::GetDiskFreeSpaceExW, core::PCWSTR};
                    let path: Vec<u16> = disk_root.as_os_str().encode_wide().chain([0]).collect();
                    let mut bytes = 0;
                    unsafe {
                        GetDiskFreeSpaceExW(PCWSTR(path.as_ptr()), Some(&mut bytes), None, None)
                            .map_err(|_| localclip_core::store::StoreError::Unavailable)?;
                    }
                    Ok(bytes)
                }),
                ..Default::default()
            },
        )
        .map_err(|_| "storageUnavailable")?,
    );
    let policy = store.clone();
    let writer = store.clone();
    let listener = Listener::start(
        Arc::new(move || policy.settings().map_err(|_| "storageUnavailable")),
        Arc::new(move |capture, at, revision| match capture {
            Ok(capture) => {
                let result = writer.ingest(Snapshot {
                    capture_id: format!("{}-{}-{}", std::process::id(), capture.sequence, at),
                    captured_at: at,
                    payload: capture.payload,
                    source: capture.source,
                    policy_revision: revision,
                    sensitive: false,
                });
                let status = match result {
                    Ok(Ingest::Inserted(_)) => "inserted",
                    Ok(Ingest::Deduplicated(_)) => "deduplicated",
                    Ok(Ingest::Skipped(reason)) => reason,
                    Err(_) => "storageUnavailable",
                };
                println!("capture-status={status}");
            }
            Err(reason) => println!("capture-status={reason}"),
        }),
    )?;
    println!("listener-ready");
    std::thread::sleep(std::time::Duration::from_secs(seconds));
    drop(listener);
    println!("listener-stopped");
    Ok(())
}
