use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

pub fn normalize(value: &str) -> String {
    value.nfc().map(|c| c.to_ascii_lowercase()).collect()
}

pub fn literal_rank(body: &[u8], needle: &[u8], source: &str) -> u8 {
    if needle.is_empty() {
        return 4;
    }
    let position = match (std::str::from_utf8(body), std::str::from_utf8(needle)) {
        (Ok(body), Ok(needle)) => body.find(needle),
        _ => body.windows(needle.len()).position(|w| w == needle),
    };
    if let Some(position) = position {
        return if position != 0 {
            2
        } else if body.len() == needle.len() {
            0
        } else {
            1
        };
    }
    if normalize(source)
        .as_bytes()
        .windows(needle.len())
        .any(|w| w == needle)
    {
        3
    } else {
        4
    }
}

pub fn context(original: &str, literal: &str) -> String {
    let graphemes: Vec<&str> = original.graphemes(true).collect();
    let normalized = normalize(original);
    let needle = normalize(literal);
    let Some(position) = (!needle.is_empty())
        .then(|| normalized.find(&needle))
        .flatten()
    else {
        return graphemes.iter().take(180).copied().collect();
    };
    // NFC and ASCII folding preserve grapheme order, not UTF-8 byte offsets.
    let before = normalized[..position].graphemes(true).count();
    let length = normalized[position..position + needle.len()]
        .graphemes(true)
        .count();
    let start = before.saturating_sub(35).min(graphemes.len());
    let end = (before + length + 79).min(graphemes.len());
    let mut result = String::new();
    if start > 0 {
        result.push('…');
    }
    for grapheme in &graphemes[start..end] {
        result.push_str(grapheme);
    }
    if end < graphemes.len() {
        result.push('…');
    }
    result
}

pub fn html_text(html: &str) -> String {
    let mut value = html.to_string();
    for name in ["script", "style", "head"] {
        let mut position = 0;
        loop {
            let lower = value.to_ascii_lowercase();
            let Some(relative) = lower[position..].find(&format!("<{name}")) else {
                break;
            };
            let start = position + relative;
            let boundary = start + name.len() + 1;
            if lower
                .as_bytes()
                .get(boundary)
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
            {
                position = boundary;
                continue;
            }
            let Some(open_end) = lower[boundary..].find('>').map(|n| boundary + n) else {
                break;
            };
            let Some(close_start) = lower[open_end + 1..]
                .find(&format!("</{name}"))
                .map(|n| open_end + 1 + n)
            else {
                position = open_end + 1;
                continue;
            };
            let Some(close_end) = lower[close_start..].find('>').map(|n| close_start + n + 1)
            else {
                break;
            };
            if !lower[close_start + name.len() + 2..close_end - 1]
                .trim()
                .is_empty()
            {
                position = open_end + 1;
                continue;
            }
            value.replace_range(start..close_end, "");
            position = start;
        }
    }
    let mut output = String::new();
    let mut rest = value.as_str();
    while let Some(start) = rest.find('<') {
        output.push_str(&rest[..start]);
        rest = &rest[start..];
        if rest.starts_with("<!--")
            && let Some(end) = rest.find("-->")
        {
            rest = &rest[end + 3..];
            continue;
        }
        let Some(end) = rest.find('>') else {
            break;
        };
        let tag = rest[1..end].to_ascii_lowercase();
        let trimmed = tag.trim_end();
        if trimmed == "br"
            || trimmed
                .strip_suffix('/')
                .is_some_and(|s| s.trim_end() == "br")
            || matches!(
                trimmed,
                "/p" | "/div" | "/li" | "/h1" | "/h2" | "/h3" | "/h4" | "/h5" | "/h6"
            )
        {
            output.push('\n');
        }
        rest = &rest[end + 1..];
    }
    output.push_str(rest);
    for (entity, replacement) in [
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&apos;", "'"),
        ("&nbsp;", " "),
    ] {
        output = output.replace(entity, replacement);
    }
    let mut decoded = String::new();
    let mut rest = output.as_str();
    while let Some(start) = rest.find("&#") {
        decoded.push_str(&rest[..start]);
        rest = &rest[start..];
        if let Some(end) = rest.find(';') {
            let token = &rest[2..end];
            let (digits, radix) = token.strip_prefix('x').map_or((token, 10), |n| (n, 16));
            if !digits.is_empty()
                && digits
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() && (radix == 16 || c.is_ascii_digit()))
                && let Some(scalar) = u32::from_str_radix(digits, radix)
                    .ok()
                    .and_then(char::from_u32)
            {
                decoded.push(scalar);
                rest = &rest[end + 1..];
                continue;
            }
        }
        decoded.push_str("&#");
        rest = &rest[2..];
    }
    decoded.push_str(rest);
    decoded.replace("&amp;", "&")
}
