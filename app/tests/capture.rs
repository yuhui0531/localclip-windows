#![cfg(windows)]

use localclip_app::win::clipboard::{self, MessageWindow};
use localclip_core::domain::Payload;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc,
    time::Duration,
};
use windows::Win32::{
    Foundation::HANDLE,
    System::{DataExchange::*, Memory::*},
};

#[test]
fn sensitive_markers_never_reach_generated_store_files() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../target/native-tests")
        .join(format!("privacy-{}", std::process::id()));
    assert!(!root.exists());
    let mut child = Command::new(env!("CARGO_BIN_EXE_localclip-demo"))
        .args(["capture", "--data-dir"])
        .arg(&root)
        .args(["--seconds", "30"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if tx.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(10)).unwrap(),
        "listener-ready"
    );
    let window = MessageWindow::new().unwrap();
    let sentinel = "LocalClip-generated-private-sentinel-四种标记";
    for marker in [
        "ExcludeClipboardContentFromMonitorProcessing",
        "Clipboard Viewer Ignore",
        "CanIncludeInClipboardHistory",
        "CanUploadToCloudClipboard",
    ] {
        clipboard::write(window.0, &Payload::text(sentinel)).unwrap();
        unsafe {
            OpenClipboard(Some(window.0)).unwrap();
            let memory = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, 4).unwrap();
            let pointer = GlobalLock(memory);
            assert!(!pointer.is_null());
            std::ptr::write_bytes(pointer, 0, 4);
            let _ = GlobalUnlock(memory);
            SetClipboardData(
                clipboard::registered(marker).unwrap(),
                Some(HANDLE(memory.0)),
            )
            .unwrap();
            CloseClipboard().unwrap();
        }
        assert_eq!(
            rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            "capture-status=sensitive"
        );
        let utf16: Vec<u8> = sentinel.encode_utf16().flat_map(u16::to_le_bytes).collect();
        for file in std::fs::read_dir(&root).unwrap() {
            let bytes = std::fs::read(file.unwrap().path()).unwrap();
            assert!(
                !bytes
                    .windows(sentinel.len())
                    .any(|v| v == sentinel.as_bytes())
            );
            assert!(!bytes.windows(utf16.len()).any(|v| v == utf16));
        }
    }
    child.kill().unwrap();
    child.wait().unwrap();
    reader.join().unwrap();
    unsafe {
        OpenClipboard(Some(window.0)).unwrap();
        EmptyClipboard().unwrap();
        CloseClipboard().unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
    println!("privacy markers=4 file-byte-scans=4 leaks=0");
}
