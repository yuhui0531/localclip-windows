use crate::desktop::AppState;
use image::ImageReader;
use localclip_core::{domain::Payload, export::dibv5_to_bmp};
use std::{
    collections::VecDeque,
    io::Cursor,
    sync::{Arc, Mutex},
};
use tauri::Manager;

#[derive(Default)]
pub struct Cache {
    entries: VecDeque<(String, Arc<Vec<u8>>)>,
    bytes: usize,
}
impl Cache {
    pub fn clear(&mut self) {
        self.entries.clear();
        self.bytes = 0;
    }
    fn get(&mut self, key: &str) -> Option<Arc<Vec<u8>>> {
        let index = self.entries.iter().position(|(k, _)| k == key)?;
        let value = self.entries.remove(index)?;
        let bytes = value.1.clone();
        self.entries.push_back(value);
        Some(bytes)
    }
    fn put(&mut self, key: String, bytes: Vec<u8>) -> Arc<Vec<u8>> {
        while self.bytes + bytes.len() > 16 * 1_048_576 {
            let Some((_, old)) = self.entries.pop_front() else {
                break;
            };
            self.bytes -= old.len();
        }
        let bytes = Arc::new(bytes);
        if bytes.len() <= 16 * 1_048_576 {
            self.bytes += bytes.len();
            self.entries.push_back((key, bytes.clone()));
        }
        bytes
    }
}
fn thumbnail(payload: &Payload) -> Option<Vec<u8>> {
    for item in &payload.items {
        let mut candidates = Vec::new();
        for format in ["image/png", "image/jpeg", "image/x-win-dibv5"] {
            if let Some(bytes) = item.representation(format) {
                candidates.push(if format.ends_with("dibv5") {
                    dibv5_to_bmp(bytes)?
                } else {
                    bytes.to_vec()
                });
            }
        }
        if item.file_path().is_some() {
            let path = crate::win::formats::url_to_path(
                std::str::from_utf8(item.representation("text/uri-list")?).ok()?,
            )?;
            if localclip_core::domain::is_image_file(&path)
                && std::fs::metadata(&path).ok()?.len() <= 50 * 1_048_576
                && let Ok(bytes) = std::fs::read(path)
            {
                candidates.push(bytes);
            }
        }
        for bytes in candidates {
            let reader = ImageReader::new(Cursor::new(&bytes))
                .with_guessed_format()
                .ok()?;
            let Ok((w, h)) = reader.into_dimensions() else {
                continue;
            };
            if u64::from(w) * u64::from(h) > 40_000_000 {
                continue;
            }
            let Ok(decoded) = image::load_from_memory(&bytes) else {
                continue;
            };
            let mut out = Cursor::new(Vec::new());
            if decoded
                .thumbnail(640, 480)
                .write_to(&mut out, image::ImageFormat::Png)
                .is_ok()
            {
                return Some(out.into_inner());
            }
        }
    }
    None
}
pub fn response(app: &tauri::AppHandle, uri: &tauri::http::Uri) -> tauri::http::Response<Vec<u8>> {
    let result = (|| {
        if uri.host() != Some("thumb") && uri.host() != Some("localclip.localhost") {
            return None;
        }
        let id = uri
            .path()
            .strip_prefix("/thumb/")
            .or_else(|| uri.path().strip_prefix('/'))?;
        if id.is_empty() || id.contains('/') || id.len() > 128 {
            return None;
        }
        let state = app.state::<AppState>();
        if state
            .runtime
            .store
            .settings()
            .ok()?
            .preferences
            .hide_contents
        {
            return None;
        }
        state.runtime.store.summary(id).ok()?;
        let key = format!("{}:{}", state.runtime.store.revision().ok()?, id);
        if let Some(bytes) = state.thumbs.lock().ok()?.get(&key) {
            return Some((*bytes).clone());
        }
        let payload = state.runtime.store.payload(id).ok()?;
        let bytes = thumbnail(&payload)?;
        // Do not retain a result that completed after the panel was hidden.
        if app
            .get_webview_window("main")
            .and_then(|w| w.is_visible().ok())
            .unwrap_or(false)
        {
            state.thumbs.lock().ok()?.put(key, bytes.clone());
        }
        Some(bytes)
    })();
    tauri::http::Response::builder()
        .status(if result.is_some() { 200 } else { 404 })
        .header("Content-Type", "image/png")
        .header("Cache-Control", "no-store")
        .header("Access-Control-Allow-Origin", "http://tauri.localhost")
        .body(result.unwrap_or_default())
        .unwrap()
}
pub type SharedCache = Mutex<Cache>;
