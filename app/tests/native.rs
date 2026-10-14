#![cfg(windows)]

use localclip_app::win::source;

#[test]
fn current_process_identity_is_stable_and_path_is_normalized() {
    let a = source::process(std::process::id()).unwrap();
    let b = source::process(std::process::id()).unwrap();
    assert_eq!(a.created, b.created);
    assert_ne!(a.created, 0);
    assert_eq!(a.path, a.path.to_lowercase());
    assert!(!a.name.is_empty());
    assert!(source::process(u32::MAX).is_none());
}

#[test]
fn conditional_clipboard_write_preserves_newer_copy() {
    use localclip_app::win::clipboard::{self, MessageWindow};
    use localclip_core::domain::Payload;
    use windows::Win32::System::DataExchange::*;
    let window = MessageWindow::new().unwrap();
    let (first, _) = clipboard::write(window.0, &Payload::text("generated first")).unwrap();
    let (second, _) = clipboard::write(window.0, &Payload::text("generated newer copy")).unwrap();
    assert_ne!(first.sequence, second.sequence);
    assert!(matches!(
        clipboard::write_checked(
            window.0,
            &Payload::text("stale conversion"),
            Some(first.sequence)
        ),
        Err("clipboardChanged")
    ));
    assert_eq!(unsafe { GetClipboardSequenceNumber() }, second.sequence);
    assert!(
        clipboard::write_checked(
            window.0,
            &Payload::text("generated replacement"),
            Some(second.sequence)
        )
        .is_ok()
    );
    unsafe {
        OpenClipboard(Some(window.0)).unwrap();
        EmptyClipboard().unwrap();
        CloseClipboard().unwrap();
    }
}
