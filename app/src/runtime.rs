//! Windows 采集与前台事件生命周期。by AI.Coding
use crate::win::{
    clipboard::Result,
    foreground::Tracker,
    listener::{Listener, now},
    ocr, paste,
    source::Target,
};
use localclip_core::{
    domain::Payload,
    store::{HistoryStore, Ingest, Options, Snapshot, StoreError},
};
use std::{
    path::Path,
    sync::{Arc, Mutex, atomic::AtomicBool},
};

pub struct Runtime {
    pub store: Arc<HistoryStore>,
    pub drag_cache: Mutex<crate::drag_cache::DragCache>,
    root: std::path::PathBuf,
    listener: Mutex<Option<Listener>>,
    tracker: Mutex<Option<Tracker>>,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
impl Runtime {
    /// 将前台变化独立传递给面板，避免把窗口事件当作采集状态。
    pub fn open(
        root: &Path,
        notify: Arc<dyn Fn(&'static str) + Send + Sync>,
        foreground_changed: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self> {
        std::fs::create_dir_all(root).map_err(|_| "storageUnavailable")?;
        let disk_root = root.to_owned();
        let store = Arc::new(
            HistoryStore::open(
                root.join("history.sqlite"),
                Options {
                    now: Arc::new(now),
                    available_space: Arc::new(move || {
                        use std::os::windows::ffi::OsStrExt;
                        use windows::{
                            Win32::Storage::FileSystem::GetDiskFreeSpaceExW, core::PCWSTR,
                        };
                        let path: Vec<u16> =
                            disk_root.as_os_str().encode_wide().chain([0]).collect();
                        let mut bytes = 0;
                        unsafe {
                            GetDiskFreeSpaceExW(
                                PCWSTR(path.as_ptr()),
                                Some(&mut bytes),
                                None,
                                None,
                            )
                            .map_err(|_| StoreError::Unavailable)?;
                        }
                        Ok(bytes)
                    }),
                    ..Default::default()
                },
            )
            .map_err(|e| {
                if e == StoreError::IncompatibleSchema {
                    "incompatibleSchema"
                } else {
                    "storageUnavailable"
                }
            })?,
        );
        store.expire().map_err(|_| "storageUnavailable")?;
        let maintenance = store.clone();
        let maintenance_notify = notify.clone();
        let policy = store.clone();
        let writer = store.clone();
        let listener = Listener::start(
            Arc::new(move || policy.settings().map_err(|_| "storageUnavailable")),
            Arc::new(move |capture, at, revision| {
                let status = match capture {
                    Ok(capture) => match writer.ingest(Snapshot {
                        capture_id: format!("{}-{}-{}", std::process::id(), capture.sequence, at),
                        captured_at: at,
                        payload: capture.payload,
                        source: capture.source,
                        policy_revision: revision,
                        sensitive: false,
                    }) {
                        Ok(Ingest::Inserted(_)) => "inserted",
                        Ok(Ingest::Deduplicated(_)) => "deduplicated",
                        Ok(Ingest::Skipped(reason)) => reason,
                        Err(_) => "storageUnavailable",
                    },
                    Err(reason) => reason,
                };
                notify(status);
            }),
            Arc::new(move || match maintenance.expire() {
                Ok(result) if result.removed > 0 => maintenance_notify("expired"),
                Ok(_) => maintenance_notify("ready"),
                Err(_) => maintenance_notify("storageUnavailable"),
            }),
        )?;
        let tracker = Tracker::start(foreground_changed)?;
        Ok(Self {
            root: root.to_owned(),
            drag_cache: Mutex::new(crate::drag_cache::DragCache::new(root.join("DragImages"))),
            store,
            listener: Mutex::new(Some(listener)),
            tracker: Mutex::new(Some(tracker)),
        })
    }
    pub fn output(
        &self,
        payload: Payload,
        target: Option<Target>,
        direct: bool,
        modifiers_released: bool,
        hide: impl FnOnce(),
    ) -> Result<paste::Outcome> {
        let listener = self.listener.lock().map_err(|_| "accessUnavailable")?;
        paste::output_with_code_files(
            listener.as_ref().ok_or("accessUnavailable")?,
            payload,
            target,
            direct,
            modifiers_released,
            hide,
            || {
                self.store.settings().is_ok_and(|s| {
                    s.preferences.smart_code_file_paste && !s.preferences.hide_contents
                })
            },
        )
    }
    pub fn ocr(
        &self,
        id: &str,
        languages: &[String],
        cancelled: &AtomicBool,
    ) -> Result<(usize, Vec<String>)> {
        let sequence =
            unsafe { windows::Win32::System::DataExchange::GetClipboardSequenceNumber() };
        let payload = self.store.payload(id).map_err(|_| "notFound")?;
        let settings = self.store.settings().map_err(|_| "storageUnavailable")?;
        let result = ocr::recognize(
            &payload,
            languages,
            cancelled,
            settings.preferences.hide_contents,
        )?;
        if cancelled.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("cancelled");
        }
        if self
            .store
            .settings()
            .map_err(|_| "storageUnavailable")?
            .preferences
            .hide_contents
        {
            return Err("contentsHidden");
        }
        if result.text.is_empty() {
            return Ok((0, result.missing_languages));
        }
        let lines = result.text.lines().count();
        self.store.summary(id).map_err(|_| "notFound")?;
        self.listener
            .lock()
            .map_err(|_| "accessUnavailable")?
            .as_ref()
            .ok_or("accessUnavailable")?
            .copy_snapshot(Payload::text(&result.text), sequence)?;
        Ok((lines, result.missing_languages))
    }
    pub fn free_disk_bytes(&self) -> Result<u64> {
        use std::os::windows::ffi::OsStrExt;
        use windows::{Win32::Storage::FileSystem::GetDiskFreeSpaceExW, core::PCWSTR};
        let path: Vec<u16> = self.root.as_os_str().encode_wide().chain([0]).collect();
        let mut bytes = 0;
        unsafe {
            GetDiskFreeSpaceExW(PCWSTR(path.as_ptr()), Some(&mut bytes), None, None)
                .map_err(|_| "storageUnavailable")?;
        }
        Ok(bytes)
    }
    pub fn shutdown(&self) {
        if let Ok(mut listener) = self.listener.lock() {
            drop(listener.take());
        }
        if let Ok(mut tracker) = self.tracker.lock() {
            drop(tracker.take());
        }
    }
}
