//! 新增功能的独立行为回归。by AI.Coding
use localclip_app::{drag_cache::DragCache, output};
use localclip_core::domain::Payload;
#[test]
fn image_paths_survive_drop_and_modified_exports_are_not_reused() {
    let root = std::env::temp_dir().join(format!("localclip-cache-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let cache = DragCache::new(root.clone());
    let first = cache.prepare(b"original image", "png").unwrap();
    let reused = cache.prepare(b"original image", "png").unwrap();
    assert_eq!(first.path, reused.path);
    assert!(!reused.created);
    std::fs::write(&first.path, b"externally edited").unwrap();
    let next = cache.prepare(b"original image", "png").unwrap();
    assert_ne!(first.path, next.path);
    assert_eq!(std::fs::read(&first.path).unwrap(), b"externally edited");
    assert_eq!(std::fs::read(&next.path).unwrap(), b"original image");
    assert!(cache.bytes().unwrap() > 0);
    std::fs::write(root.join("unrelated.txt"), b"keep").unwrap();
    cache.clear().unwrap();
    assert!(root.join("unrelated.txt").exists());
    assert!(!next.path.exists());
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn output_limit_rejects_large_combined_records_before_transform() {
    let text = "x".repeat(26 * 1024 * 1024);
    assert_eq!(
        output::compose(
            vec![Payload::text(&text), Payload::text(&text)],
            true,
            "newline",
            None,
            None,
            0
        )
        .err(),
        Some("outputTooLarge")
    );
}
