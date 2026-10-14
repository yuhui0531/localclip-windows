use localclip_core::{domain::Payload, export};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use windows::Win32::{
    Foundation::GlobalFree,
    System::{Com::*, Memory::*, Ole::*},
    UI::Shell::{SHCreateDataObject, SHDoDragDrop},
};

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
static NEXT: AtomicU64 = AtomicU64::new(0);

// A synchronous Shell drop keeps temporary exports alive until the receiver finishes.
pub fn export_drag(payload: Payload, language: Option<String>) -> Result<bool, &'static str> {
    std::thread::spawn(move || {
        unsafe { OleInitialize(None) }.map_err(|_| "dragUnavailable")?;
        struct Ole;
        impl Drop for Ole {
            fn drop(&mut self) {
                unsafe { OleUninitialize() };
            }
        }
        let _ole = Ole;
        let mut paths = Vec::new();
        let mut temporary = None;
        if payload.items.iter().all(|i| i.file_path().is_some()) {
            for item in &payload.items {
                let url =
                    std::str::from_utf8(item.representation("text/uri-list").ok_or("unsupported")?)
                        .map_err(|_| "unsupported")?;
                let path = super::formats::url_to_path(url).ok_or("unsupported")?;
                if std::path::Path::new(&path).exists() {
                    paths.push(path);
                }
            }
        } else {
            let file = export::prepare(&payload, language.as_deref(), None).ok_or("unsupported")?;
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|_| "saveFailed")?
                .as_nanos();
            let root = std::env::temp_dir().join(format!(
                "LocalClip-drag-{}-{stamp}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&root).map_err(|_| "saveFailed")?;
            temporary = Some(Temporary(root.clone()));
            let path = root.join(format!("LocalClip.{}", file.extension));
            std::fs::write(&path, file.bytes).map_err(|_| "saveFailed")?;
            paths.push(path.to_str().ok_or("invalidName")?.to_string());
        }
        if paths.is_empty() {
            return Err("filesMissing");
        }
        let bytes = super::formats::encode_dropfiles(&paths).ok_or("unsupported")?;
        let result = unsafe {
            let data: IDataObject = SHCreateDataObject(None, None, None::<&IDataObject>)
                .map_err(|_| "dragUnavailable")?;
            let memory = GlobalAlloc(GMEM_MOVEABLE, bytes.len()).map_err(|_| "dragUnavailable")?;
            let ptr = GlobalLock(memory);
            if ptr.is_null() {
                let _ = GlobalFree(Some(memory));
                return Err("dragUnavailable");
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.cast(), bytes.len());
            let _ = GlobalUnlock(memory);
            let format = FORMATETC {
                cfFormat: 15,
                dwAspect: DVASPECT_CONTENT.0,
                lindex: -1,
                tymed: TYMED_HGLOBAL.0 as u32,
                ..Default::default()
            };
            let medium = STGMEDIUM {
                tymed: TYMED_HGLOBAL.0 as u32,
                u: STGMEDIUM_0 { hGlobal: memory },
                ..Default::default()
            };
            if data.SetData(&format, &medium, true).is_err() {
                let _ = GlobalFree(Some(memory));
                return Err("dragUnavailable");
            }
            SHDoDragDrop(None, &data, None::<&IDropSource>, DROPEFFECT_COPY)
                .map(|effect| effect == DROPEFFECT_COPY)
                .map_err(|_| "dragUnavailable")
        };
        drop(temporary);
        result
    })
    .join()
    .map_err(|_| "dragUnavailable")?
}
