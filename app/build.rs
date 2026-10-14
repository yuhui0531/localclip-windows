fn main() {
    #[cfg(feature = "desktop")]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "search",
                "clip_detail",
                "perform",
                "transform_menu",
                "save_as",
                "inspect",
                "queue_start",
                "queue_stop",
                "queue_status",
                "set_pinned",
                "ocr_copy",
                "ocr_cancel",
                "set_favorite",
                "set_title",
                "delete",
                "undo_delete",
                "categories",
                "upsert_category",
                "delete_category",
                "assign_categories",
                "settings",
                "update_settings",
                "preview_change",
                "apply_change",
                "usage",
                "capture_status",
                "pause",
                "resume",
            ]),
        ))
        .expect("Tauri manifest build failed");
    }
}
