use localclip_core::{domain::Payload, export::dibv5_to_bmp};
use std::sync::atomic::{AtomicBool, Ordering};
use windows::{
    Globalization::Language,
    Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize},
    core::HSTRING,
};

pub struct Recognition {
    pub text: String,
    pub missing_languages: Vec<String>,
}
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

fn bitmap(payload: &Payload, cancelled: &AtomicBool) -> Result<SoftwareBitmap, &'static str> {
    for item in &payload.items {
        // 文件引用优先真实内容；图标失败时继续下一项而非识别图标。by AI.Coding
        if item.file_path().is_some() {
            if let Some(path) = super::preview::path(item)
                && let Some(bytes) = super::preview::read(&path)
            {
                let candidate = Payload {
                    items: vec![localclip_core::domain::Item {
                        representations: vec![localclip_core::domain::Representation {
                            format: "image/png".into(),
                            data: bytes,
                        }],
                        ..Default::default()
                    }],
                };
                match bitmap(&candidate, cancelled) {
                    Ok(bitmap) => return Ok(bitmap),
                    Err("unsupported") => {}
                    Err(error) => return Err(error),
                }
            }
            continue;
        }
        for (format, decoder_format) in [
            ("image/png", image::ImageFormat::Png),
            ("image/jpeg", image::ImageFormat::Jpeg),
            ("image/x-win-dibv5", image::ImageFormat::Bmp),
        ] {
            if cancelled.load(Ordering::SeqCst) {
                return Err("cancelled");
            }
            let Some(raw) = item.representation(format) else {
                continue;
            };
            let bmp;
            let bytes = if decoder_format == image::ImageFormat::Bmp {
                bmp = match dibv5_to_bmp(raw) {
                    Some(v) => v,
                    None => continue,
                };
                bmp.as_slice()
            } else {
                raw
            };
            let Ok((width, height)) = image::ImageReader::new(std::io::Cursor::new(bytes))
                .with_guessed_format()
                .ok()
                .and_then(|r| r.into_dimensions().ok())
                .ok_or("unsupported")
            else {
                continue;
            };
            if width as u64 * height as u64 > 40_000_000 {
                return Err("tooManyPixels");
            }
            let stream = InMemoryRandomAccessStream::new().map_err(|_| "ocrUnavailable")?;
            let writer = DataWriter::CreateDataWriter(&stream).map_err(|_| "ocrUnavailable")?;
            writer.WriteBytes(bytes).map_err(|_| "ocrUnavailable")?;
            writer
                .StoreAsync()
                .map_err(|_| "ocrUnavailable")?
                .join()
                .map_err(|_| "ocrUnavailable")?;
            writer.DetachStream().map_err(|_| "ocrUnavailable")?;
            stream.Seek(0).map_err(|_| "ocrUnavailable")?;
            let decoder = match BitmapDecoder::CreateAsync(&stream).and_then(|v| v.join()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            if cancelled.load(Ordering::SeqCst) {
                return Err("cancelled");
            }
            let width = decoder.PixelWidth().map_err(|_| "ocrUnavailable")?;
            let height = decoder.PixelHeight().map_err(|_| "ocrUnavailable")?;
            if width as u64 * height as u64 > 40_000_000 {
                return Err("tooManyPixels");
            }
            let limit = OcrEngine::MaxImageDimension().map_err(|_| "ocrUnavailable")?;
            if width > limit || height > limit {
                return Err("engineDimensionLimit");
            }
            if let Ok(bitmap) = decoder
                .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Ignore)
                .and_then(|v| v.join())
            {
                return Ok(bitmap);
            }
        }
    }
    Err("unsupported")
}

pub fn recognize(
    payload: &Payload,
    languages: &[String],
    cancelled: &AtomicBool,
    hide_contents: bool,
) -> Result<Recognition, &'static str> {
    if hide_contents {
        return Err("contentsHidden");
    }
    if cancelled.load(Ordering::SeqCst) {
        return Err("cancelled");
    }
    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)
            .ok()
            .map_err(|_| "ocrUnavailable")?;
    }
    let _apartment = Apartment;
    let mut missing_languages = Vec::new();
    let mut engines = Vec::new();
    for tag in languages {
        if !["zh-Hans", "zh-Hant", "en-US"].contains(&tag.as_str()) {
            return Err("unsupportedLanguage");
        }
        let language =
            Language::CreateLanguage(&HSTRING::from(tag)).map_err(|_| "ocrUnavailable")?;
        if !OcrEngine::IsLanguageSupported(&language).map_err(|_| "ocrUnavailable")? {
            missing_languages.push(tag.clone());
            continue;
        }
        engines.push(OcrEngine::TryCreateFromLanguage(&language).map_err(|_| "ocrUnavailable")?);
    }
    if engines.is_empty() {
        return Err("languagePackMissing");
    }
    let bitmap = bitmap(payload, cancelled)?;
    for engine in engines {
        let operation = engine
            .RecognizeAsync(&bitmap)
            .map_err(|_| "ocrUnavailable")?;
        while operation.Status().map_err(|_| "ocrUnavailable")?.0 == 0 {
            if cancelled.load(Ordering::SeqCst) {
                let _ = operation.Cancel();
                return Err("cancelled");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let result = operation.GetResults().map_err(|_| "ocrUnavailable")?;
        let lines = result.Lines().map_err(|_| "ocrUnavailable")?;
        let mut text = Vec::new();
        for line in lines {
            text.push(line.Text().map_err(|_| "ocrUnavailable")?.to_string());
        }
        if cancelled.load(Ordering::SeqCst) {
            return Err("cancelled");
        }
        let text = text.join("\n");
        if !text.trim().is_empty() {
            return Ok(Recognition {
                text,
                missing_languages,
            });
        }
    }
    Ok(Recognition {
        text: String::new(),
        missing_languages,
    })
}
