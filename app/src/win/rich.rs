use localclip_core::domain::Item;
use windows::{
    Win32::{
        Foundation::{FreeLibrary, HWND, LPARAM, WPARAM},
        System::LibraryLoader::LoadLibraryW,
        UI::{
            Controls::{EM_REPLACESEL, EM_SETSEL},
            WindowsAndMessaging::*,
        },
    },
    core::w,
};

#[repr(C, packed(4))]
struct Stream {
    cookie: usize,
    error: u32,
    callback: Option<unsafe extern "system" fn(usize, *mut u8, i32, *mut i32) -> u32>,
}
struct Data {
    bytes: Vec<u8>,
    offset: usize,
}
unsafe extern "system" fn read(
    cookie: usize,
    target: *mut u8,
    size: i32,
    written: *mut i32,
) -> u32 {
    unsafe {
        let data = &mut *(cookie as *mut Data);
        let n = (size.max(0) as usize).min(data.bytes.len().saturating_sub(data.offset));
        std::ptr::copy_nonoverlapping(data.bytes.as_ptr().add(data.offset), target, n);
        data.offset += n;
        *written = n as i32;
    }
    0
}
unsafe extern "system" fn write(
    cookie: usize,
    source: *mut u8,
    size: i32,
    written: *mut i32,
) -> u32 {
    unsafe {
        let data = &mut *(cookie as *mut Data);
        if size < 0 || data.bytes.len().saturating_add(size as usize) > 50 * 1_048_576 {
            return 1;
        }
        data.bytes
            .extend_from_slice(std::slice::from_raw_parts(source, size as usize));
        *written = size;
    }
    0
}
// RichEdit resolves independent RTF font/color tables when appending fragments.
pub fn merge(items: &[&Item]) -> Result<Vec<u8>, &'static str> {
    unsafe {
        let module = LoadLibraryW(w!("Msftedit.dll")).map_err(|_| "unsupportedCombination")?;
        let result = (|| {
            let window = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("RICHEDIT50W"),
                w!(""),
                WINDOW_STYLE(ES_MULTILINE as u32),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
            .map_err(|_| "unsupportedCombination")?;
            struct Window(HWND);
            impl Drop for Window {
                fn drop(&mut self) {
                    unsafe {
                        let _ = DestroyWindow(self.0);
                    }
                }
            }
            let window = Window(window);
            for (index, item) in items.iter().enumerate() {
                SendMessageW(
                    window.0,
                    EM_SETSEL,
                    Some(WPARAM(usize::MAX)),
                    Some(LPARAM(-1)),
                );
                if index > 0 {
                    SendMessageW(
                        window.0,
                        EM_REPLACESEL,
                        Some(WPARAM(0)),
                        Some(LPARAM(w!("\r\n").as_ptr() as isize)),
                    );
                }
                if let Some(rtf) = item.representation("text/rtf") {
                    let mut data = Data {
                        bytes: rtf.to_vec(),
                        offset: 0,
                    };
                    let mut stream = Stream {
                        cookie: &mut data as *mut _ as usize,
                        error: 0,
                        callback: Some(read),
                    };
                    SendMessageW(
                        window.0,
                        WM_USER + 73,
                        Some(WPARAM(0x8002)),
                        Some(LPARAM(&mut stream as *mut _ as isize)),
                    );
                    if stream.error != 0 {
                        return Err("unsupportedCombination");
                    }
                } else {
                    let text =
                        super::formats::encode_utf16(item.plain_text.as_deref().unwrap_or(""));
                    SendMessageW(
                        window.0,
                        EM_REPLACESEL,
                        Some(WPARAM(0)),
                        Some(LPARAM(text.as_ptr() as isize)),
                    );
                }
            }
            let mut data = Data {
                bytes: Vec::new(),
                offset: 0,
            };
            let mut stream = Stream {
                cookie: &mut data as *mut _ as usize,
                error: 0,
                callback: Some(write),
            };
            SendMessageW(
                window.0,
                WM_USER + 74,
                Some(WPARAM(2)),
                Some(LPARAM(&mut stream as *mut _ as isize)),
            );
            if stream.error != 0 || data.bytes.is_empty() {
                return Err("unsupportedCombination");
            }
            Ok(data.bytes)
        })();
        let _ = FreeLibrary(module);
        result
    }
}
