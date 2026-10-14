//! 原生拖出：文本使用 Unicode，导出使用内存虚拟文件，真实文件保留路径。by AI.Coding
use localclip_core::{domain::Payload, export};
use std::mem::ManuallyDrop;
use windows::Win32::{
    Foundation::{GlobalFree, HWND},
    Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
    System::{Com::*, DataExchange::RegisterClipboardFormatW, Memory::*, Ole::*},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, ReleaseCapture, VK_LBUTTON},
        Shell::*,
    },
};

/// 在 UI STA 线程同步拖放；数据对象持有内容，接收方无需追赶临时文件的删除时机。
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
    let data = drag_data(&payload, language.as_deref())?;
    unsafe {
        // 清除 WebView 的鼠标捕获，让 OLE 接管后续移动及松开。
        let _ = ReleaseCapture();
        SHDoDragDrop(Some(owner), &data, None::<&IDropSource>, DROPEFFECT_COPY)
            .map(|effect| effect == DROPEFFECT_COPY)
            .map_err(|_| "dragUnavailable")
    }
}

/// 同一内容按目标协商为文本或虚拟文件；只有真实文件才发布 CF_HDROP。
fn drag_data(payload: &Payload, language: Option<&str>) -> Result<IDataObject, &'static str> {
    let data: IDataObject = unsafe { SHCreateDataObject(None, None, None::<&IDataObject>) }
        .map_err(|_| "dragUnavailable")?;
    if !payload.items.is_empty() && payload.items.iter().all(|i| i.file_path().is_some()) {
        let mut paths = Vec::new();
        for item in &payload.items {
            let url =
                std::str::from_utf8(item.representation("text/uri-list").ok_or("unsupported")?)
                    .map_err(|_| "unsupported")?;
            let path = super::formats::url_to_path(url).ok_or("unsupported")?;
            if std::path::Path::new(&path).exists() {
                paths.push(path);
            }
        }
        if paths.is_empty() {
            return Err("filesMissing");
        }
        let bytes = super::formats::encode_dropfiles(&paths).ok_or("unsupported")?;
        set_bytes(&data, 15, &bytes)?;
    } else {
        let file = export::prepare(payload, language, None).ok_or("unsupported")?;
        let item = &payload.items[0];
        if let Some(text) = item.plain_text.as_deref().or_else(|| {
            item.representation("text/plain")
                .or_else(|| item.representation("text/uri"))
                .and_then(|bytes| std::str::from_utf8(bytes).ok())
        }) {
            // 文本编辑器取得正文，而不是把导出文件路径当作待打开的文档。
            let bytes: Vec<u8> = text
                .encode_utf16()
                .chain([0])
                .flat_map(u16::to_le_bytes)
                .collect();
            set_bytes(&data, 13, &bytes)?;
        }
        let name: Vec<u16> = format!("LocalClip.{}", file.extension)
            .encode_utf16()
            .collect();
        let mut filename = [0; 260];
        filename[..name.len()].copy_from_slice(&name);
        let descriptor = FILEGROUPDESCRIPTORW {
            cItems: 1,
            fgd: [FILEDESCRIPTORW {
                dwFlags: (FD_ATTRIBUTES | FD_FILESIZE).0 as u32,
                dwFileAttributes: FILE_ATTRIBUTE_NORMAL.0,
                nFileSizeHigh: ((file.bytes.len() as u64) >> 32) as u32,
                nFileSizeLow: file.bytes.len() as u32,
                cFileName: filename,
                ..Default::default()
            }],
        };
        // packed 描述符不含填充字节；文件内容由 COM 流持有，完全不落临时磁盘。
        let bytes = unsafe {
            std::slice::from_raw_parts(
                (&descriptor as *const FILEGROUPDESCRIPTORW).cast::<u8>(),
                std::mem::size_of::<FILEGROUPDESCRIPTORW>(),
            )
        };
        let descriptor_format = unsafe { RegisterClipboardFormatW(CFSTR_FILEDESCRIPTORW) } as u16;
        let contents_format = unsafe { RegisterClipboardFormatW(CFSTR_FILECONTENTS) } as u16;
        if descriptor_format == 0 || contents_format == 0 {
            return Err("dragUnavailable");
        }
        set_bytes(&data, descriptor_format, bytes)?;
        let stream = unsafe { SHCreateMemStream(Some(&file.bytes)) }.ok_or("dragUnavailable")?;
        let mut medium = STGMEDIUM {
            tymed: TYMED_ISTREAM.0 as u32,
            u: STGMEDIUM_0 {
                pstm: ManuallyDrop::new(Some(stream)),
            },
            ..Default::default()
        };
        let format = FORMATETC {
            cfFormat: contents_format,
            dwAspect: DVASPECT_CONTENT.0,
            lindex: 0,
            tymed: TYMED_ISTREAM.0 as u32,
            ..Default::default()
        };
        unsafe {
            if data.SetData(&format, &medium, true).is_err() {
                ReleaseStgMedium(&mut medium);
                return Err("dragUnavailable");
            }
        }
    }
    Ok(data)
}

/// SetData 成功转交所有权；失败则释放当前分配，避免拖放错误路径泄漏。
fn set_bytes(data: &IDataObject, cf_format: u16, bytes: &[u8]) -> Result<(), &'static str> {
    unsafe {
        let memory = GlobalAlloc(GMEM_MOVEABLE, bytes.len()).map_err(|_| "dragUnavailable")?;
        let ptr = GlobalLock(memory);
        if ptr.is_null() {
            let _ = GlobalFree(Some(memory));
            return Err("dragUnavailable");
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.cast(), bytes.len());
        let _ = GlobalUnlock(memory);
        let format = FORMATETC {
            cfFormat: cf_format,
            dwAspect: DVASPECT_CONTENT.0,
            lindex: -1,
            tymed: TYMED_HGLOBAL.0 as u32,
            ..Default::default()
        };
        let mut medium = STGMEDIUM {
            tymed: TYMED_HGLOBAL.0 as u32,
            u: STGMEDIUM_0 { hGlobal: memory },
            ..Default::default()
        };
        if data.SetData(&format, &medium, true).is_err() {
            ReleaseStgMedium(&mut medium);
            return Err("dragUnavailable");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use localclip_core::domain::{Item, Representation};

    /// 每个测试线程自行初始化 OLE，并在所有数据对象释放后退出。
    struct Apartment;
    impl Apartment {
        fn new() -> Self {
            unsafe { OleInitialize(None).unwrap() };
            Self
        }
    }
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { OleUninitialize() };
        }
    }
    struct Medium(STGMEDIUM);
    impl Drop for Medium {
        fn drop(&mut self) {
            unsafe { ReleaseStgMedium(&mut self.0) };
        }
    }

    /// 接收方只按标准格式读取，不依赖生产函数中的编码步骤。
    fn format(id: u16, index: i32, medium: TYMED) -> FORMATETC {
        FORMATETC {
            cfFormat: id,
            dwAspect: DVASPECT_CONTENT.0,
            lindex: index,
            tymed: medium.0 as u32,
            ..Default::default()
        }
    }
    fn bytes(data: &IDataObject, id: u16) -> Vec<u8> {
        unsafe {
            let medium = Medium(data.GetData(&format(id, -1, TYMED_HGLOBAL)).unwrap());
            let memory = medium.0.u.hGlobal;
            let ptr = GlobalLock(memory);
            assert!(!ptr.is_null());
            let result = std::slice::from_raw_parts(ptr.cast::<u8>(), GlobalSize(memory)).to_vec();
            let _ = GlobalUnlock(memory);
            result
        }
    }
    fn contents(data: &IDataObject, expected: &[u8], extension: &str) {
        unsafe {
            let descriptor_id = RegisterClipboardFormatW(CFSTR_FILEDESCRIPTORW) as u16;
            let descriptor = bytes(data, descriptor_id);
            assert_eq!(u32::from_le_bytes(descriptor[..4].try_into().unwrap()), 1);
            let group =
                std::ptr::read_unaligned(descriptor.as_ptr().cast::<FILEGROUPDESCRIPTORW>());
            let file = group.fgd[0];
            let name = file.cFileName;
            let count = name.iter().position(|v| *v == 0).unwrap();
            assert_eq!(
                String::from_utf16(&name[..count]).unwrap(),
                format!("LocalClip.{extension}")
            );
            assert_eq!(
                ((file.nFileSizeHigh as u64) << 32) | file.nFileSizeLow as u64,
                expected.len() as u64
            );
            let id = RegisterClipboardFormatW(CFSTR_FILECONTENTS) as u16;
            // 延迟读取和再次请求必须仍可取得完整字节，空文件也须提供合法流。
            for _ in 0..2 {
                let medium = Medium(data.GetData(&format(id, 0, TYMED_ISTREAM)).unwrap());
                let stream = medium.0.u.pstm.as_ref().unwrap();
                stream.Seek(0, STREAM_SEEK_SET, None).unwrap();
                let mut received = vec![0; expected.len() + 1];
                let mut read = 0;
                stream
                    .Read(
                        received.as_mut_ptr().cast(),
                        received.len() as u32,
                        Some(&mut read),
                    )
                    .ok()
                    .unwrap();
                assert_eq!(&received[..read as usize], expected);
            }
        }
    }

    #[test]
    fn text_drop_offers_body_and_virtual_file_without_a_temporary_path() {
        let _ole = Apartment::new();
        let text = "generated 文本 🦀\r\nsecond line";
        // 原 payload 离开作用域后，目标持有的数据对象仍然拥有正文。
        let data = {
            let payload = Payload::text(text);
            drag_data(&payload, None).unwrap()
        };
        assert!(unsafe { data.QueryGetData(&format(15, -1, TYMED_HGLOBAL)) }.is_err());
        let raw = bytes(&data, 13);
        let utf16: Vec<u16> = raw
            .chunks_exact(2)
            .map(|v| u16::from_le_bytes([v[0], v[1]]))
            .take_while(|v| *v != 0)
            .collect();
        assert_eq!(String::from_utf16(&utf16).unwrap(), text);
        contents(&data, text.as_bytes(), "txt");
    }

    #[test]
    fn exports_keep_empty_code_image_and_link_bytes() {
        let _ole = Apartment::new();
        contents(&drag_data(&Payload::text(""), None).unwrap(), b"", "txt");
        let code = "fn main() { println!(\"generated\"); }";
        contents(
            &drag_data(&Payload::text(code), Some("rust")).unwrap(),
            code.as_bytes(),
            "rs",
        );
        let image = Payload {
            items: vec![Item {
                representations: vec![Representation {
                    format: "image/png".into(),
                    data: vec![137, 80, 78, 71],
                }],
                ..Default::default()
            }],
        };
        let data = drag_data(&image, None).unwrap();
        assert!(unsafe { data.QueryGetData(&format(13, -1, TYMED_HGLOBAL)) }.is_err());
        contents(&data, &[137, 80, 78, 71], "png");
        let link = Payload::text("https://example.com");
        contents(
            &drag_data(&link, None).unwrap(),
            b"[InternetShortcut]\r\nURL=https://example.com\r\n",
            "url",
        );
    }

    #[test]
    fn real_files_still_offer_original_paths() {
        let _ole = Apartment::new();
        let root =
            std::env::temp_dir().join(format!("LocalClip-drag-protocol-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("generated.txt");
        std::fs::write(&path, b"generated").unwrap();
        let url = super::super::formats::path_to_url(path.to_str().unwrap()).unwrap();
        let payload = Payload {
            items: vec![Item {
                representations: vec![Representation {
                    format: "text/uri-list".into(),
                    data: url.into_bytes(),
                }],
                ..Default::default()
            }],
        };
        let data = drag_data(&payload, None).unwrap();
        assert!(unsafe { data.QueryGetData(&format(15, -1, TYMED_HGLOBAL)) }.is_ok());
        assert!(unsafe { data.QueryGetData(&format(13, -1, TYMED_HGLOBAL)) }.is_err());
        assert!(path.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
