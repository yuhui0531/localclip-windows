//! 原生拖出：在持有鼠标输入的 UI STA 线程运行 OLE 拖放。by AI.Coding
use localclip_core::{domain::Payload, export};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use windows::Win32::{
    Foundation::{GlobalFree, HWND},
    System::{Com::*, Memory::*, Ole::*},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, ReleaseCapture, VK_LBUTTON},
        Shell::{SHCreateDataObject, SHDoDragDrop},
    },
};

struct Temporary(PathBuf);
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
static NEXT: AtomicU64 = AtomicU64::new(0);

/// 同步等待接收方完成复制，确保临时导出文件在 drop/cancel 前始终存在。
/// 必须由窗口 UI 线程调用；后台线程没有此次按下事件的输入队列。
pub fn export_drag(
    owner: HWND,
    payload: Payload,
    language: Option<String>,
) -> Result<bool, &'static str> {
    // IPC 调度期间已松开鼠标则取消，不能让拖拽锁住后续失焦行为。
    if unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) } >= 0 {
        return Ok(false);
    }
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
        let data: IDataObject =
            SHCreateDataObject(None, None, None::<&IDataObject>).map_err(|_| "dragUnavailable")?;
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
        // 清除 WebView 的鼠标捕获，让 OLE 接管后续移动及松开。
        let _ = ReleaseCapture();
        SHDoDragDrop(Some(owner), &data, None::<&IDropSource>, DROPEFFECT_COPY)
            .map(|effect| effect == DROPEFFECT_COPY)
            .map_err(|_| "dragUnavailable")
    };
    drop(temporary);
    result
}
