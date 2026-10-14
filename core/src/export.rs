use crate::{
    domain::{Kind, Payload, web_link},
    inspect,
};

pub struct PreparedFile {
    pub extension: &'static str,
    pub bytes: Vec<u8>,
}
pub fn prepare(
    payload: &Payload,
    language: Option<&str>,
    markup: Option<&str>,
) -> Option<PreparedFile> {
    if payload.items.len() != 1 {
        return None;
    }
    let item = &payload.items[0];
    if payload.kind() == Kind::Image {
        for (format, extension) in [
            ("image/png", "png"),
            ("image/jpeg", "jpg"),
            ("image/x-win-dibv5", "bmp"),
        ] {
            if let Some(bytes) = item.representation(format) {
                return Some(PreparedFile {
                    extension,
                    bytes: if extension == "bmp" {
                        dibv5_to_bmp(bytes)?
                    } else {
                        bytes.to_vec()
                    },
                });
            }
        }
        return None;
    }
    if matches!(payload.kind(), Kind::File | Kind::Mixed) {
        return None;
    }
    let text = item
        .plain_text
        .as_deref()
        .or_else(|| {
            item.representation("text/plain")
                .and_then(|r| std::str::from_utf8(r).ok())
        })
        .or_else(|| {
            item.representation("text/uri")
                .and_then(|r| std::str::from_utf8(r).ok())
        })
        .unwrap_or("");
    if payload.kind() == Kind::Url
        && let Some(url) = web_link(text)
    {
        return Some(PreparedFile {
            extension: "url",
            bytes: format!("[InternetShortcut]\r\nURL={url}\r\n").into_bytes(),
        });
    }
    let types = item
        .representations
        .iter()
        .map(|r| r.format.as_str())
        .collect::<Vec<_>>();
    let format = inspect::export_format(text, language, &types, markup);
    Some(PreparedFile {
        extension: format.extension,
        bytes: if let Some(original) = format.original_type {
            item.representation(original)?.to_vec()
        } else {
            text.as_bytes().into()
        },
    })
}
pub fn dibv5_to_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    fn u32at(bytes: &[u8], offset: usize) -> Option<u32> {
        Some(u32::from_le_bytes(
            bytes.get(offset..offset + 4)?.try_into().ok()?,
        ))
    }
    if dib.len() < 124 || u32at(dib, 0)? != 124 {
        return None;
    }
    let bits = u16::from_le_bytes(dib[14..16].try_into().ok()?);
    let compression = u32at(dib, 16)?;
    if ![1, 4, 8, 16, 24, 32].contains(&bits) || ![0, 3, 6].contains(&compression) {
        return None;
    }
    let colors = u32at(dib, 32)?;
    let colors = if colors == 0 && bits <= 8 {
        1u32 << bits
    } else {
        colors
    };
    let offset = 124usize.checked_add(colors as usize * 4)?;
    if offset > dib.len() {
        return None;
    }
    let size = u32::try_from(dib.len().checked_add(14)?).ok()?;
    let mut bmp = Vec::with_capacity(size as usize);
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&size.to_le_bytes());
    bmp.extend_from_slice(&[0u8; 4]);
    bmp.extend_from_slice(&u32::try_from(offset + 14).ok()?.to_le_bytes());
    bmp.extend_from_slice(dib);
    Some(bmp)
}
pub fn available_name(name: &str, exists: impl Fn(&str) -> bool) -> Option<String> {
    if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
        return None;
    }
    if !exists(name) {
        return Some(name.into());
    }
    let (stem, extension) = name
        .rsplit_once('.')
        .map_or((name, String::new()), |(stem, extension)| {
            (stem, format!(".{extension}"))
        });
    for n in 2..=99 {
        let candidate = format!("{stem} {n}{extension}");
        if !exists(&candidate) {
            return Some(candidate);
        }
    }
    None
}
