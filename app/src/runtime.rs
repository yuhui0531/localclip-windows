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
    listener: Mutex<Option<Listener>>,
    tracker: Mutex<Option<Tracker>>,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
impl Runtime {
    pub fn open(root: &Path, notify: Arc<dyn Fn(&'static str) + Send + Sync>) -> Result<Self> {
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
            .map_err(|_| "storageUnavailable")?,
        );
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
        )?;
        let tracker = Tracker::start()?;
        Ok(Self {
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
        paste::output(
            listener.as_ref().ok_or("accessUnavailable")?,
            payload,
            target,
            direct,
            modifiers_released,
            hide,
        )
    }
    pub fn ocr(
        &self,
        id: &str,
        languages: &[String],
        cancelled: &AtomicBool,
    ) -> Result<(usize, Vec<String>)> {
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
        if result.text.is_empty() {
            return Ok((0, result.missing_languages));
        }
        let lines = result.text.lines().count();
        self.output(Payload::text(&result.text), None, false, true, || {})?;
        Ok((lines, result.missing_languages))
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
