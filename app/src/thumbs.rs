//! 内容感知缩略图与有界缓存。by AI.Coding
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
/// 卡片请求缩略图，详情请求原始像素；两者缓存分离且不写入历史数据。
fn thumbnail(payload: &Payload, full: bool) -> Option<Vec<u8>> {
    for item in &payload.items {
        let mut candidates = Vec::new();
        for format in ["image/png", "image/jpeg", "image/x-win-dibv5"] {
            if let Some(bytes) = item.representation(format) {
                if format.ends_with("dibv5") {
                    if let Some(bmp) = dibv5_to_bmp(bytes) {
                        candidates.push(bmp);
                    }
                } else {
                    candidates.push(bytes.to_vec());
                }
            }
        }
        if item.file_path().is_some() {
            if let Some(path) = crate::win::preview::path(item) {
                let bytes = crate::win::preview::read(&path);
                if let Some(png) = bytes
                    .as_deref()
                    .and_then(|b| crate::win::preview::image_preview(b, full))
                {
                    return Some(png);
                }
                if let Some(png) = crate::win::preview::document(&path, bytes.as_deref()) {
                    return Some(png);
                }
            }
            // 文件图标不是内容预览，文件本身失败时不误用其剪贴板图标。
            continue;
        }
        for bytes in candidates {
            let Ok(reader) = ImageReader::new(Cursor::new(&bytes)).with_guessed_format() else {
                continue;
            };
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
            let rendered = if full {
                decoded
            } else {
                decoded.thumbnail(640, 480)
            };
            if rendered.write_to(&mut out, image::ImageFormat::Png).is_ok() {
                return Some(out.into_inner());
            }
        }
    }
    None
}
pub fn response(app: &tauri::AppHandle, uri: &tauri::http::Uri) -> tauri::http::Response<Vec<u8>> {
    let result = (|| {
        let id = thumbnail_id(uri.host()?, uri.path())?;
        let full = uri
            .query()
            .is_some_and(|q| q.split('&').any(|part| part == "view=full"));
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
        state.runtime.store.summary(&id).ok()?;
        let payload = state.runtime.store.payload(&id).ok()?;
        let info = crate::win::preview::info(&payload);
        let key = format!(
            "{}:{}:{}:{}",
            full,
            state.runtime.store.revision().ok()?,
            id,
            info["fileVersion"]
        );
        if let Some(bytes) = state.thumbs.lock().ok()?.get(&key) {
            return Some((*bytes).clone());
        }
        let bytes = thumbnail(&payload, full)?;
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
        state.runtime.store.summary(&id).ok()?;
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

fn thumbnail_id(host: &str, path: &str) -> Option<String> {
    // Wry reverses Windows http://localclip.localhost to localclip://localhost.
    let path = percent_encoding::percent_decode_str(path)
        .decode_utf8()
        .ok()?;
    let id = match host {
        "thumb" => path.strip_prefix('/')?,
        "localhost" | "localclip.localhost" => path.strip_prefix("/thumb/")?,
        _ => return None,
    };
    (!id.is_empty() && id.len() <= 128 && !id.contains(['/', '\\', '\0'])).then(|| id.to_string())
}
#[cfg(test)]
mod tests {
    use super::thumbnail_id;
    #[test]
    fn windows_protocol_rewrite_and_encoded_slash_resolve_the_same_id() {
        assert_eq!(
            thumbnail_id("localhost", "/thumb%2Fgenerated-id"),
            Some("generated-id".into())
        );
        assert_eq!(
            thumbnail_id("localhost", "/thumb/generated-id"),
            Some("generated-id".into())
        );
        assert_eq!(
            thumbnail_id("thumb", "/generated-id"),
            Some("generated-id".into())
        );
        assert_eq!(thumbnail_id("remote.example", "/thumb/generated-id"), None);
        assert_eq!(thumbnail_id("localhost", "/thumb/a%2Fb"), None);
    }
}
