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
