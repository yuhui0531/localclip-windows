use crate::inspect;
use base64::Engine;
use std::collections::BTreeSet;
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

pub const BYTE_LIMIT: usize = 4 * 1_048_576;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Transform {
    MarkdownCodeBlock,
    Dedent,
    TrimWhitespace,
    StripPrompt,
    StripLineNumbers,
    FormatJson,
    MinifyJson,
    QuoteString,
    UnquoteString,
    Base64Encode,
    Base64Decode,
    UrlEncode,
    UrlDecode,
    CamelCase,
    SnakeCase,
    KebabCase,
    Uppercase,
    Lowercase,
    SortLines,
    UniqueLines,
    JwtPayload,
    TimestampToIso,
    IsoToTimestamp,
    ColorToRgb,
    ColorToHex,
}
pub const TRANSFORMS: [(Transform, &str, &str); 25] = [
    (
        Transform::MarkdownCodeBlock,
        "markdownCodeBlock",
        "Markdown 代码块",
    ),
    (Transform::Dedent, "dedent", "去公共缩进"),
    (Transform::TrimWhitespace, "trimWhitespace", "清理空白"),
    (Transform::StripPrompt, "stripPrompt", "去命令提示符"),
    (Transform::StripLineNumbers, "stripLineNumbers", "去行号"),
    (Transform::FormatJson, "formatJSON", "格式化 JSON"),
    (Transform::MinifyJson, "minifyJSON", "压缩 JSON"),
    (Transform::QuoteString, "quoteString", "转为字符串字面量"),
    (Transform::UnquoteString, "unquoteString", "反转义字符串"),
    (Transform::Base64Encode, "base64Encode", "Base64 编码"),
    (Transform::Base64Decode, "base64Decode", "Base64 解码"),
    (Transform::UrlEncode, "urlEncode", "URL 编码"),
    (Transform::UrlDecode, "urlDecode", "URL 解码"),
    (Transform::CamelCase, "camelCase", "camelCase"),
    (Transform::SnakeCase, "snakeCase", "snake_case"),
    (Transform::KebabCase, "kebabCase", "kebab-case"),
    (Transform::Uppercase, "uppercase", "全大写"),
    (Transform::Lowercase, "lowercase", "全小写"),
    (Transform::SortLines, "sortLines", "行排序"),
    (Transform::UniqueLines, "uniqueLines", "行去重"),
    (Transform::JwtPayload, "jwtPayload", "JWT 载荷 JSON"),
    (
        Transform::TimestampToIso,
        "timestampToISO",
        "时间戳转 ISO 8601",
    ),
    (
        Transform::IsoToTimestamp,
        "isoToTimestamp",
        "ISO 8601 转时间戳",
    ),
    (Transform::ColorToRgb, "colorToRGB", "颜色转 rgb()"),
    (Transform::ColorToHex, "colorToHex", "颜色转 hex"),
];
impl Transform {
    pub fn parse(value: &str) -> Option<Self> {
        TRANSFORMS
            .iter()
            .find(|(_, name, _)| *name == value)
            .map(|(t, _, _)| *t)
    }
    pub fn name(self) -> &'static str {
        TRANSFORMS.iter().find(|(t, _, _)| *t == self).unwrap().1
    }
    pub fn title(self) -> &'static str {
        TRANSFORMS.iter().find(|(t, _, _)| *t == self).unwrap().2
    }
    pub fn applicable(self, text: &str, offset: i32) -> bool {
        if text.is_empty() || text.len() > BYTE_LIMIT {
            return false;
        }
        self.apply(text, None, offset).is_ok()
    }
    pub fn apply(self, text: &str, language: Option<&str>, offset: i32) -> Result<String, String> {
        if text.len() > BYTE_LIMIT {
            return Err("内容过大（超过 4 MiB），无法转换".into());
        }
        let unavailable = || format!("当前内容不适用“{}”", self.title());
        Ok(match self {
            Self::MarkdownCodeBlock => code_block(text, language),
            Self::Dedent => {
                let rows = lines(text);
                let mut indent: Option<&str> = None;
                for (body, _) in &rows {
                    if body.trim_matches([' ', '\t']).is_empty() {
                        continue;
                    }
                    let current = &body[..body.len() - body.trim_start_matches([' ', '\t']).len()];
                    indent = Some(match indent {
                        None => current,
                        Some(previous) => {
                            &previous[..previous
                                .bytes()
                                .zip(current.bytes())
                                .take_while(|(a, b)| a == b)
                                .count()]
                        }
                    });
                }
                let indent = indent.filter(|i| !i.is_empty()).ok_or_else(unavailable)?;
                rows.iter()
                    .map(|(body, end)| {
                        format!("{}{end}", body.strip_prefix(indent).unwrap_or(body))
                    })
                    .collect()
            }
            Self::TrimWhitespace => {
                let mut rows: Vec<_> = lines(text)
                    .into_iter()
                    .map(|(b, e)| (b.trim_end_matches([' ', '\t']), e))
                    .collect();
                let first = rows
                    .iter()
                    .position(|(b, _)| !b.is_empty())
                    .unwrap_or(rows.len());
                rows.drain(..first);
                while rows.last().is_some_and(|(b, _)| b.is_empty()) {
                    rows.pop();
                }
                if let Some(last) = rows.last_mut() {
                    last.1 = "";
                }
                rows.iter().map(|(b, e)| format!("{b}{e}")).collect()
            }
            Self::StripPrompt => {
                let rows = lines(text);
                if !rows.iter().any(|(b, _)| prompt(b).is_some()) {
                    return Err(unavailable());
                }
                rows.iter()
                    .map(|(b, e)| {
                        if let Some(start) = prompt(b) {
                            format!("{}{}{e}", &b[..start], &b[start + 2..])
                        } else {
                            format!("{b}{e}")
                        }
                    })
                    .collect()
            }
            Self::StripLineNumbers => {
                let rows = lines(text);
                let prefixes = numbered_prefixes(&rows).ok_or_else(unavailable)?;
                rows.iter()
                    .zip(prefixes)
                    .map(|((b, e), n)| format!("{}{e}", &b[n..]))
                    .collect()
            }
            Self::FormatJson | Self::MinifyJson => {
                validate_json(text)?;
                reflow_json(text, self == Self::FormatJson)
            }
            Self::QuoteString => quote(text),
            Self::UnquoteString => {
                serde_json::from_str::<String>(text.trim()).map_err(|_| "不是有效的字符串字面量")?
            }
            Self::Base64Encode => base64::engine::general_purpose::STANDARD.encode(text),
            Self::Base64Decode => base64_decode(text)?,
            Self::UrlEncode => {
                let mut out = String::new();
                for b in text.bytes() {
                    if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
                        out.push(b as char);
                    } else {
                        use std::fmt::Write;
                        let _ = write!(out, "%{b:02X}");
                    }
                }
                out
            }
            Self::UrlDecode => {
                let bytes = text.as_bytes();
                let mut found = text.contains('+');
                for (i, b) in bytes.iter().enumerate() {
                    if *b == b'%' {
                        if bytes.get(i + 1).is_none_or(|v| !v.is_ascii_hexdigit())
                            || bytes.get(i + 2).is_none_or(|v| !v.is_ascii_hexdigit())
                        {
                            return Err("URL 解码失败：百分号编码无效".into());
                        }
                        found = true;
                    }
                }
                if !found {
                    return Err("URL 解码失败：百分号编码无效".into());
                }
                percent_encoding::percent_decode_str(&text.replace('+', " "))
                    .decode_utf8()
                    .map_err(|_| "URL 解码失败：百分号编码无效")?
                    .into_owned()
            }
            Self::CamelCase | Self::SnakeCase | Self::KebabCase => lines(text)
                .iter()
                .map(|(b, e)| format!("{}{e}", recase(b, self)))
                .collect(),
            Self::Uppercase => text.chars().flat_map(char::to_uppercase).collect(),
            Self::Lowercase => text.chars().flat_map(char::to_lowercase).collect(),
            Self::SortLines | Self::UniqueLines => {
                let mut rows: Vec<_> = lines(text).iter().map(|(b, _)| *b).collect();
                let trailing = rows.len() > 1 && rows.last() == Some(&"");
                if trailing {
                    rows.pop();
                }
                if rows.len() < 2 {
                    return Err(unavailable());
                }
                if self == Self::SortLines {
                    rows.sort_by(|a, b| natural_compare(a, b));
                } else {
                    let mut seen = BTreeSet::new();
                    rows.retain(|v| seen.insert(v.nfc().collect::<String>()));
                }
                let sep = if text.contains("\r\n") { "\r\n" } else { "\n" };
                rows.join(sep) + if trailing { sep } else { "" }
            }
            Self::JwtPayload => {
                let parts = inspect::jwt_parts(text.trim(), true).ok_or("不是完整的 JWT")?;
                reflow_json(
                    &String::from_utf8(inspect::base64url_data(parts[1]).ok_or("不是完整的 JWT")?)
                        .map_err(|_| "不是完整的 JWT")?,
                    true,
                )
            }
            Self::TimestampToIso => {
                let value = text.trim();
                let ms = inspect::timestamp(value).ok_or("不是 10 位秒或 13 位毫秒时间戳")?;
                inspect::iso(ms, offset, value.len() == 13)
                    .ok_or("不是 10 位秒或 13 位毫秒时间戳")?
            }
            Self::IsoToTimestamp => {
                let value = text.trim();
                let date = time::OffsetDateTime::parse(
                    value,
                    &time::format_description::well_known::Rfc3339,
                )
                .map_err(|_| "不是 ISO 8601 时间")?;
                if value.contains('.') {
                    (date.unix_timestamp_nanos() / 1_000_000).to_string()
                } else {
                    date.unix_timestamp().to_string()
                }
            }
            Self::ColorToRgb | Self::ColorToHex => {
                let color = inspect::color(text).ok_or("不是可识别的颜色值")?;
                if self == Self::ColorToRgb {
                    color.rgb()
                } else {
                    color.hex()
                }
            }
        })
    }
}

pub fn lines(text: &str) -> Vec<(&str, &str)> {
    let mut rows = Vec::new();
    let mut start = 0;
    for (at, g) in text.grapheme_indices(true) {
        if matches!(
            g,
            "\r\n" | "\n" | "\r" | "\u{b}" | "\u{c}" | "\u{85}" | "\u{2028}" | "\u{2029}"
        ) {
            rows.push((&text[start..at], g));
            start = at + g.len();
        }
    }
    rows.push((&text[start..], ""));
    rows
}
fn prompt(line: &str) -> Option<usize> {
    let rest = line.trim_start_matches([' ', '\t']);
    (rest.starts_with("$ ") || rest.starts_with("% ")).then_some(line.len() - rest.len())
}
fn numbered_prefixes(rows: &[(&str, &str)]) -> Option<Vec<usize>> {
    let mut prefixes = Vec::new();
    let (mut count, mut nonblank, mut previous) = (0, 0, None);
    for (body, _) in rows {
        let rest = body.trim_start_matches([' ', '\t']);
        if !rest.is_empty() {
            nonblank += 1;
        }
        let digits = rest.bytes().take_while(u8::is_ascii_digit).count().min(6);
        let mut consumed = 0;
        if digits > 0 {
            let tail = &rest[digits..];
            let bytes = tail.as_bytes();
            let ws = usize::from(matches!(bytes.first(), Some(b' ' | b'\t')));
            let delimiter = if bytes.get(ws) == Some(&b'|') {
                ws + 1 + usize::from(matches!(bytes.get(ws + 1), Some(b' ' | b'\t')))
            } else if bytes.first() == Some(&b':') {
                1 + usize::from(matches!(bytes.get(1), Some(b' ' | b'\t')))
            } else if bytes.first() == Some(&b'.') && matches!(bytes.get(1), Some(b' ' | b'\t')) {
                2
            } else {
                ws
            };
            if delimiter > 0 {
                let n = rest[..digits].parse::<u32>().ok()?;
                if previous.is_some_and(|p| n <= p) {
                    return None;
                }
                previous = Some(n);
                count += 1;
                consumed = body.len() - rest.len() + digits + delimiter;
            }
        }
        prefixes.push(consumed);
    }
    (count >= 2 && count * 5 >= nonblank * 4).then_some(prefixes)
}
pub fn code_block(text: &str, language: Option<&str>) -> String {
    let (mut longest, mut run) = (0, 0);
    for b in text.bytes() {
        if b == b'`' {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    let fence = "`".repeat(3.max(longest + 1));
    let sep = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut language = language.unwrap_or("");
    let beginning = text
        .graphemes(true)
        .take(512)
        .collect::<String>()
        .to_lowercase();
    if language == "xml" && (beginning.contains("<html") || beginning.contains("<!doctype html")) {
        language = "html";
    }
    let ends = lines(text).len() > 1 && lines(text).last().is_some_and(|(s, _)| s.is_empty());
    format!(
        "{fence}{language}{sep}{text}{}{fence}",
        if ends { "" } else { sep }
    )
}
pub fn quote(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c < '\u{20}' => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04X}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
pub fn base64_decode(text: &str) -> Result<String, String> {
    let mut value = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .replace('-', "+")
        .replace('_', "/");
    if value.len() < 4
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"+/=".contains(&b))
    {
        return Err("不是有效的 Base64".into());
    }
    value.push_str(&"=".repeat((4 - value.len() % 4) % 4));
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(value)
        .map_err(|_| "不是有效的 Base64")?;
    let decoded = String::from_utf8(bytes).map_err(|_| "Base64 解码结果不是文本")?;
    if decoded.is_empty()
        || decoded
            .chars()
            .any(|c| c < '\u{20}' && !matches!(c, '\r' | '\n' | '\t'))
    {
        return Err("Base64 解码结果不是文本".into());
    }
    Ok(decoded)
}
pub fn validate_json(text: &str) -> Result<serde_json::Value, String> {
    // Foundation permits trailing commas. Validate a temporary lexical copy; output retains every token.
    let bytes = text.as_bytes();
    let mut clean = Vec::new();
    let (mut quoted, mut escaped) = (false, false);
    for (i, &b) in bytes.iter().enumerate() {
        if quoted {
            clean.push(b);
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
            continue;
        }
        if b == b'"' {
            quoted = true;
        }
        if b == b','
            && bytes[i + 1..]
                .iter()
                .find(|b| !b.is_ascii_whitespace())
                .is_some_and(|b| matches!(b, b']' | b'}'))
        {
            continue;
        }
        clean.push(b);
    }
    serde_json::from_slice(&clean).map_err(|e| {
        if text.trim().is_empty() {
            "不是有效 JSON".into()
        } else {
            format!(
                "不是有效 JSON：第 {} 行第 {} 列",
                e.line(),
                e.column().saturating_sub(1)
            )
        }
    })
}
pub fn reflow_json(text: &str, pretty: bool) -> String {
    let bytes = text.as_bytes();
    let (mut i, mut depth) = (0, 0usize);
    let (mut quoted, mut escaped) = (false, false);
    let mut out = Vec::new();
    fn newline(out: &mut Vec<u8>, pretty: bool, depth: usize) {
        if pretty {
            out.push(b'\n');
            out.extend(std::iter::repeat_n(b' ', depth * 2));
        }
    }
    while i < bytes.len() {
        let b = bytes[i];
        i += 1;
        if quoted {
            out.push(b);
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'"' {
                quoted = false;
            }
            continue;
        }
        match b {
            b'"' => {
                quoted = true;
                out.push(b);
            }
            b' ' | b'\n' | b'\r' | b'\t' => {}
            b'{' | b'[' => {
                out.push(b);
                let mut next = i;
                while next < bytes.len() && bytes[next].is_ascii_whitespace() {
                    next += 1;
                }
                if bytes.get(next) == Some(&(b + 2)) {
                    out.push(b + 2);
                    i = next + 1;
                } else {
                    depth += 1;
                    newline(&mut out, pretty, depth);
                }
            }
            b'}' | b']' => {
                depth = depth.saturating_sub(1);
                newline(&mut out, pretty, depth);
                out.push(b);
            }
            b',' => {
                out.push(b);
                newline(&mut out, pretty, depth);
            }
            b':' => {
                out.push(b);
                if pretty {
                    out.push(b' ');
                }
            }
            _ => out.push(b),
        }
    }
    String::from_utf8(out).expect("reflow only changes ASCII whitespace")
}
fn recase(line: &str, style: Transform) -> String {
    let body = line.trim_start_matches([' ', '\t']);
    let indent = &line[..line.len() - body.len()];
    let chars: Vec<_> = body.chars().collect();
    let mut words = Vec::new();
    let mut current = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if c == '_' || c == '-' || c.is_whitespace() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            continue;
        }
        if c.is_uppercase()
            && current.chars().last().is_some_and(|p| {
                p.is_lowercase()
                    || p.is_numeric()
                    || (p.is_uppercase() && chars.get(i + 1).is_some_and(|n| n.is_lowercase()))
            })
        {
            words.push(std::mem::take(&mut current));
        }
        current.push(c);
    }
    if !current.is_empty() {
        words.push(current);
    }
    if words.is_empty() {
        return line.into();
    }
    let words: Vec<String> = words
        .iter()
        .map(|w| w.chars().flat_map(char::to_lowercase).collect())
        .collect();
    let content = match style {
        Transform::CamelCase => words
            .iter()
            .enumerate()
            .map(|(i, w)| {
                if i == 0 {
                    w.clone()
                } else {
                    let mut gs = w.graphemes(true);
                    format!(
                        "{}{}",
                        gs.next().unwrap_or("").to_uppercase(),
                        gs.collect::<String>()
                    )
                }
            })
            .collect(),
        Transform::SnakeCase => words.join("_"),
        _ => words.join("-"),
    };
    format!("{indent}{content}")
}
fn natural_compare(a: &str, b: &str) -> std::cmp::Ordering {
    let a = a.nfc().collect::<String>().to_lowercase();
    let b = b.nfc().collect::<String>().to_lowercase();
    let (mut left, mut right) = (a.as_str(), b.as_str());
    while !left.is_empty() && !right.is_empty() {
        let ln = left.bytes().take_while(u8::is_ascii_digit).count();
        let rn = right.bytes().take_while(u8::is_ascii_digit).count();
        let order = if ln > 0 && rn > 0 {
            let x = left[..ln].trim_start_matches('0');
            let y = right[..rn].trim_start_matches('0');
            left = &left[ln..];
            right = &right[rn..];
            x.len().cmp(&y.len()).then_with(|| x.cmp(y))
        } else {
            let x = left.chars().next().unwrap();
            let y = right.chars().next().unwrap();
            left = &left[x.len_utf8()..];
            right = &right[y.len_utf8()..];
            x.cmp(&y)
        };
        if order != std::cmp::Ordering::Equal {
            return order;
        }
    }
    left.len().cmp(&right.len())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Join {
    Newline,
    BlankLine,
    Comma,
    Space,
    SqlList,
    JsonArray,
    MarkdownList,
}
impl Join {
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "newline" => Self::Newline,
            "blankLine" => Self::BlankLine,
            "comma" => Self::Comma,
            "space" => Self::Space,
            "sqlList" => Self::SqlList,
            "jsonArray" => Self::JsonArray,
            "markdownList" => Self::MarkdownList,
            _ => return None,
        })
    }
    pub fn join(self, values: &[&str]) -> String {
        let trimmed: Vec<_> = values.iter().map(|v| v.trim()).collect();
        match self {
            Self::Newline => values.join("\n"),
            Self::BlankLine => values.join("\n\n"),
            Self::Comma => trimmed.join(", "),
            Self::Space => trimmed.join(" "),
            Self::SqlList => trimmed
                .iter()
                .map(|v| format!("'{}'", v.replace('\'', "''")))
                .collect::<Vec<_>>()
                .join(", "),
            Self::JsonArray => format!(
                "[{}]",
                trimmed
                    .iter()
                    .map(|v| quote(v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::MarkdownList => trimmed
                .iter()
                .map(|v| {
                    format!(
                        "- {}",
                        lines(v)
                            .iter()
                            .map(|(b, _)| *b)
                            .collect::<Vec<_>>()
                            .join("\n  ")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}
