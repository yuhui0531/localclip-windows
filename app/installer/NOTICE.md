The MSI template is adapted from Tauri's `crates/tauri-bundler/src/bundle/windows/msi/main.wxs` (MIT OR Apache-2.0). Changes add opt-in current-user history deletion and leave first launch unchecked. Source: https://github.com/tauri-apps/tauri/blob/dev/crates/tauri-bundler/src/bundle/windows/msi/main.wxs

The deletion action executes only for REMOVE=ALL, an explicit REMOVELOCALCLIPHISTORY=1, and no major-upgrade removal. Silent uninstall defaults to preservation. LocalClip-UpdateBackups is outside the deleted directory.
