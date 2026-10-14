pub fn decode_utf16(bytes: &[u8]) -> Option<String> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let words: Vec<_> = bytes
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .take_while(|v| *v != 0)
        .collect();
    String::from_utf16(&words).ok()
}

pub fn encode_utf16(text: &str) -> Vec<u8> {
    text.encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect()
}

pub fn decode_html(bytes: &[u8]) -> Option<Vec<u8>> {
    fn offset(bytes: &[u8], key: &[u8]) -> Option<i64> {
        let head = &bytes[..bytes.len().min(65536)];
        let start = head.windows(key.len()).position(|w| w == key)?;
        if start != 0 && head[start - 1] != b'\n' {
            return None;
        }
        let tail = &head[start + key.len()..];
        let end = tail.iter().position(|b| matches!(b, b'\r' | b'\n'))?;
        std::str::from_utf8(&tail[..end]).ok()?.trim().parse().ok()
    }
    let start = usize::try_from(offset(bytes, b"StartFragment:")?).ok()?;
    let end = usize::try_from(offset(bytes, b"EndFragment:")?).ok()?;
    let html_start = offset(bytes, b"StartHTML:")?;
    let html_end = offset(bytes, b"EndHTML:")?;
    if (html_start >= 0 && start < html_start as usize)
        || (html_end >= 0 && (end > html_end as usize || html_end as usize > bytes.len()))
    {
        return None;
    }
    let fragment = bytes.get(start..end)?;
    std::str::from_utf8(fragment).ok()?;
    Some(fragment.to_vec())
}

pub fn encode_html(fragment: &[u8]) -> Option<Vec<u8>> {
    std::str::from_utf8(fragment).ok()?;
    let header = |start, end, fragment_start, fragment_end| {
        format!(
            "Version:1.0\r\nStartHTML:{start:010}\r\nEndHTML:{end:010}\r\nStartFragment:{fragment_start:010}\r\nEndFragment:{fragment_end:010}\r\n"
        )
    };
    let prefix = b"<html><body><!--StartFragment-->";
    let suffix = b"<!--EndFragment--></body></html>";
    let start = header(0, 0, 0, 0).len();
    let fragment_start = start + prefix.len();
    let fragment_end = fragment_start.checked_add(fragment.len())?;
    let end = fragment_end.checked_add(suffix.len())?;
    if end > 9_999_999_999 {
        return None;
    }
    let mut out = header(start, end, fragment_start, fragment_end).into_bytes();
    out.extend_from_slice(prefix);
    out.extend_from_slice(fragment);
    out.extend_from_slice(suffix);
    out.push(0);
    Some(out)
}

pub fn path_to_url(path: &str) -> Option<String> {
    if path.chars().any(char::is_control) || path.starts_with(r"\\?\") || path.starts_with(r"\\.\")
    {
        return None;
    }
    let normal = path.replace('\\', "/");
    let prefix = if let Some(unc) = normal.strip_prefix("//") {
        let (host, rest) = unc.split_once('/')?;
        if host.is_empty()
            || rest.is_empty()
            || !host
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.')
        {
            return None;
        }
        "file:"
    } else if normal
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && normal.as_bytes().get(1..3) == Some(b":/")
    {
        "file:///"
    } else {
        return None;
    };
    let mut url = prefix.to_string();
    for b in normal.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~' | b'/' | b':') {
            url.push(b as char);
        } else {
            use std::fmt::Write;
            write!(&mut url, "%{b:02X}").ok()?;
        }
    }
    Some(url)
}

pub fn url_to_path(url: &str) -> Option<String> {
    let rest = url.strip_prefix("file://")?;
    if rest.contains(['?', '#']) {
        return None;
    }
    let decoded = percent_encoding::percent_decode_str(rest)
        .decode_utf8()
        .ok()?;
    let path = if decoded.starts_with('/') {
        decoded.strip_prefix('/')?.replace('/', "\\")
    } else {
        format!(r"\\{}", decoded.replace('/', "\\"))
    };
    path_to_url(&path)?;
    Some(path)
}

pub fn encode_dropfiles(paths: &[String]) -> Option<Vec<u8>> {
    if paths.is_empty() || paths.len() > 32 {
        return None;
    }
    let mut out = vec![0; 20];
    out[..4].copy_from_slice(&20u32.to_le_bytes());
    out[16..20].copy_from_slice(&1u32.to_le_bytes());
    for path in paths {
        path_to_url(path)?;
        out.extend_from_slice(&encode_utf16(path));
    }
    out.extend_from_slice(&[0, 0]);
    Some(out)
}

pub fn decode_dropfiles(bytes: &[u8]) -> Option<Vec<String>> {
    let offset = u32::from_le_bytes(bytes.get(..4)?.try_into().ok()?) as usize;
    let wide = u32::from_le_bytes(bytes.get(16..20)?.try_into().ok()?);
    if offset < 20 || wide == 0 {
        return None;
    }
    let data = bytes.get(offset..)?;
    if !data.len().is_multiple_of(2) {
        return None;
    }
    let words: Vec<_> = data
        .chunks_exact(2)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    let mut out = Vec::new();
    let mut start = 0;
    for (i, word) in words.iter().enumerate() {
        if *word != 0 {
            continue;
        }
        if i == start {
            return (!out.is_empty()).then_some(out);
        }
        let path = String::from_utf16(&words[start..i]).ok()?;
        path_to_url(&path)?;
        out.push(path);
        if out.len() > 32 {
            return None;
        }
        start = i + 1;
    }
    None
}
