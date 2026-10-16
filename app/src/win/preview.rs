//! 外部文件的有界、只读预览；原始历史与文件均不修改。by AI.Coding
use localclip_core::domain::{Item, Payload};
use std::{
    fs,
    io::{Cursor, Read},
    os::windows::{ffi::OsStrExt, fs::MetadataExt},
    path::Path,
};
use windows::{
    Win32::{Foundation::SIZE, Graphics::Gdi::*, System::Com::*, UI::Shell::*},
    core::PCWSTR,
};

/// 不访问 UNC、网络盘、云端占位文件或重解析点，避免预览触发下载。
pub fn path(item: &Item) -> Option<String> {
    let url = std::str::from_utf8(item.representation("text/uri-list")?).ok()?;
    let path = super::formats::url_to_path(url)?;
    let b = path.as_bytes();
    if b.len() < 3 || !b[0].is_ascii_alphabetic() || &b[1..3] != b":\\" || path[2..].contains(':') {
        return None;
    }
    let drive: Vec<u16> = path[..3].encode_utf16().chain([0]).collect();
    let kind =
        unsafe { windows::Win32::Storage::FileSystem::GetDriveTypeW(PCWSTR(drive.as_ptr())) };
    if ![2, 3].contains(&kind) {
        return None;
    }
    let meta = fs::symlink_metadata(&path).ok()?;
    if meta.file_attributes() & (0x400 | 0x1000 | 0x40000 | 0x400000) != 0 {
        return None;
    }
    Some(path)
}
/// 内容读取受 50 MiB 限制，文件增长也不能突破预算。
pub fn read(path: &str) -> Option<Vec<u8>> {
    let file = fs::File::open(path).ok()?;
    if !file.metadata().ok()?.is_file() || file.metadata().ok()?.len() > crate::output::LIMIT {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(crate::output::LIMIT + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= crate::output::LIMIT).then_some(bytes)
}
/// 文件版本与查询时大小；目录不递归，缺失项目明确标记未知。
pub fn info(payload: &Payload) -> serde_json::Value {
    let mut known = 0u64;
    let mut unknown = 0;
    let mut folders = 0;
    let mut count = 0;
    let mut version = String::new();
    let mut text = None;
    for item in &payload.items {
        if item.file_path().is_none() {
            continue;
        }
        count += 1;
        if let Some(path) = path(item)
            && let Ok(meta) = fs::metadata(&path)
        {
            version.push_str(&format!(
                "{path}:{}:{}:{};",
                meta.len(),
                meta.last_write_time(),
                meta.creation_time()
            ));
            if meta.is_dir() {
                folders += 1;
            } else {
                known = known.saturating_add(meta.len());
            }
            if text.is_none()
                && meta.is_file()
                && meta.len() <= crate::code_file::PER_FILE_LIMIT as u64
            {
                // 完整严格解码后截断预览，不把路径或二进制图标误当正文。
                text = read(&path)
                    .and_then(|b| crate::code_file::decode(&b).ok())
                    .map(|s| s.chars().take(16384).collect::<String>());
            }
        } else {
            unknown += 1;
            version.push_str("missing;");
        }
    }
    if count == 0 {
        for item in &payload.items {
            if let Some(bytes) = ["image/png", "image/jpeg", "image/x-win-dibv5"]
                .into_iter()
                .find_map(|f| item.representation(f))
            {
                known += bytes.len() as u64;
                count += 1;
            }
        }
    }
    serde_json::json!({"contentBytes":known,"contentCount":count,"unknownCount":unknown,"folderCount":folders,"fileVersion":localclip_core::domain::hash(version.as_bytes()),"previewText":text})
}
/// 根据真实编码生成缩略图，不依赖文件扩展名。
pub fn image(bytes: &[u8]) -> Option<Vec<u8>> {
    let reader = image::ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let (w, h) = reader.into_dimensions().ok()?;
    if u64::from(w) * u64::from(h) > 40_000_000 {
        return None;
    }
    let decoded = image::load_from_memory(bytes).ok()?;
    let mut out = Cursor::new(Vec::new());
    decoded
        .thumbnail(640, 480)
        .write_to(&mut out, image::ImageFormat::Png)
        .ok()?;
    Some(out.into_inner())
}
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
/// PDF 首页使用系统渲染；Office/RTF 等由本机 Shell 缩略图处理器提供。
pub fn document(path: &str, bytes: Option<&[u8]>) -> Option<Vec<u8>> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok().ok()? };
    let _apartment = Apartment;
    if let Some(bytes) = bytes
        && bytes.starts_with(b"%PDF-")
        && let Some(png) = pdf(bytes)
    {
        return Some(png);
    }
    unsafe {
        let wide: Vec<u16> = Path::new(path)
            .as_os_str()
            .encode_wide()
            .chain([0])
            .collect();
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None::<&IBindCtx>).ok()?;
        let bitmap = factory
            .GetImage(SIZE { cx: 640, cy: 480 }, SIIGBF_THUMBNAILONLY)
            .ok()?;
        struct Bitmap(HBITMAP);
        impl Drop for Bitmap {
            fn drop(&mut self) {
                unsafe {
                    let _ = DeleteObject(self.0.into());
                }
            }
        }
        let _bitmap = Bitmap(bitmap);
        let mut info = BITMAP::default();
        if GetObjectW(
            bitmap.into(),
            std::mem::size_of::<BITMAP>() as i32,
            Some((&mut info as *mut BITMAP).cast()),
        ) == 0
        {
            return None;
        }
        let w = info.bmWidth;
        let h = info.bmHeight.abs();
        if w <= 0 || h <= 0 || i64::from(w) * i64::from(h) > 4_000_000 {
            return None;
        }
        let mut dib = BITMAPINFO::default();
        dib.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut raw = vec![0u8; (w * h * 4) as usize];
        let dc = GetDC(None);
        let read = GetDIBits(
            dc,
            bitmap,
            0,
            h as u32,
            Some(raw.as_mut_ptr().cast()),
            &mut dib,
            DIB_RGB_COLORS,
        );
        let _ = ReleaseDC(None, dc);
        if read != h {
            return None;
        }
        for pixel in raw.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            pixel[3] = 255;
        }
        let image = image::RgbaImage::from_raw(w as u32, h as u32, raw)?;
        let mut out = Cursor::new(Vec::new());
        image.write_to(&mut out, image::ImageFormat::Png).ok()?;
        Some(out.into_inner())
    }
}
fn pdf(bytes: &[u8]) -> Option<Vec<u8>> {
    use windows::{
        Data::Pdf::{PdfDocument, PdfPageRenderOptions},
        Storage::Streams::{DataReader, DataWriter, InMemoryRandomAccessStream},
    };
    let input = InMemoryRandomAccessStream::new().ok()?;
    let writer = DataWriter::CreateDataWriter(&input).ok()?;
    writer.WriteBytes(bytes).ok()?;
    writer.StoreAsync().ok()?.join().ok()?;
    writer.DetachStream().ok()?;
    input.Seek(0).ok()?;
    let document = PdfDocument::LoadFromStreamAsync(&input).ok()?.join().ok()?;
    if document.PageCount().ok()? == 0 {
        return None;
    }
    let page = document.GetPage(0).ok()?;
    let options = PdfPageRenderOptions::new().ok()?;
    options.SetDestinationWidth(640).ok()?;
    let output = InMemoryRandomAccessStream::new().ok()?;
    page.RenderWithOptionsToStreamAsync(&output, &options)
        .ok()?
        .join()
        .ok()?;
    let size = u32::try_from(output.Size().ok()?).ok()?;
    if size > 8 * 1024 * 1024 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&output.GetInputStreamAt(0).ok()?).ok()?;
    reader.LoadAsync(size).ok()?.join().ok()?;
    let mut png = vec![0; size as usize];
    reader.ReadBytes(&mut png).ok()?;
    Some(png)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_extension_images_decode_and_rewritten_files_change_version() {
        let root = std::env::temp_dir().join(format!("LocalClip-preview-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let file = root.join("generated-hash");
        let mut bytes = Cursor::new(Vec::new());
        image::RgbaImage::new(8, 8)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        fs::write(&file, bytes.get_ref()).unwrap();
        let item = Item {
            representations: vec![localclip_core::domain::Representation {
                format: "text/uri-list".into(),
                data: super::super::formats::path_to_url(file.to_str().unwrap())
                    .unwrap()
                    .into_bytes(),
            }],
            ..Default::default()
        };
        let payload = Payload { items: vec![item] };
        let before = info(&payload);
        assert!(image(&read(&path(&payload.items[0]).unwrap()).unwrap()).is_some());
        fs::write(&file, b"generated text body").unwrap();
        let after = info(&payload);
        assert_ne!(before["fileVersion"], after["fileVersion"]);
        assert_eq!(after["previewText"], "generated text body");
        assert!(image(b"not an image").is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
