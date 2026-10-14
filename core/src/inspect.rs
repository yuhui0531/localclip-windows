use crate::{domain, transform};
use base64::Engine;
use serde_json::{Value, json};
use time::{OffsetDateTime, UtcOffset};
use unicode_segmentation::UnicodeSegmentation;

pub fn timestamp(value: &str) -> Option<i64> {
    if !matches!(value.len(), 10 | 13) || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n = value.parse::<i64>().ok()?;
    let ms = if value.len() == 10 { n * 1000 } else { n };
    (946_684_800_000..=4_102_444_800_000)
        .contains(&ms)
        .then_some(ms)
}
fn date(ms: i64, offset: i32) -> Option<OffsetDateTime> {
    Some(
        OffsetDateTime::from_unix_timestamp_nanos(ms as i128 * 1_000_000)
            .ok()?
            .to_offset(UtcOffset::from_whole_seconds(offset).ok()?),
    )
}
fn local_time(ms: i64, offset: i32) -> Option<String> {
    let d = date(ms, offset)?;
    Some(format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        d.year(),
        d.month() as u8,
        d.day(),
        d.hour(),
        d.minute(),
        d.second()
    ))
}
pub fn iso(ms: i64, offset: i32, fraction: bool) -> Option<String> {
    let d = date(ms, offset)?;
    let suffix = if offset == 0 {
        "Z".into()
    } else {
        format!(
            "{}{:02}:{:02}",
            if offset < 0 { "-" } else { "+" },
            offset.abs() / 3600,
            (offset.abs() % 3600) / 60
        )
    };
    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{suffix}",
        d.year(),
        d.month() as u8,
        d.day(),
        d.hour(),
        d.minute(),
        d.second(),
        if fraction {
            format!(".{:03}", d.millisecond())
        } else {
            String::new()
        }
    ))
}
pub fn base64url_data(value: &str) -> Option<Vec<u8>> {
    let mut value = value.replace('-', "+").replace('_', "/");
    value.push_str(&"=".repeat((4 - value.len() % 4) % 4));
    base64::engine::general_purpose::STANDARD.decode(value).ok()
}
pub fn jwt_parts(value: &str, complete: bool) -> Option<Vec<&str>> {
    if !value.starts_with("eyJ") {
        return None;
    }
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() < 2 || (complete && parts.len() != 3) {
        return None;
    }
    let head: Value = serde_json::from_slice(&base64url_data(parts[0])?).ok()?;
    if !head.as_object()?.contains_key("alg") {
        return None;
    }
    if complete {
        let body: Value = serde_json::from_slice(&base64url_data(parts[1])?).ok()?;
        body.as_object()?;
    }
    Some(parts)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: f64,
}
impl Color {
    pub fn hex(self) -> String {
        format!(
            "#{:02X}{:02X}{:02X}{}",
            self.red,
            self.green,
            self.blue,
            if self.alpha < 1.0 {
                format!("{:02X}", (self.alpha * 255.0).round() as u8)
            } else {
                String::new()
            }
        )
    }
    pub fn rgb(self) -> String {
        let body = format!("{}, {}, {}", self.red, self.green, self.blue);
        if self.alpha < 1.0 {
            format!("rgba({body}, {})", significant(self.alpha))
        } else {
            format!("rgb({body})")
        }
    }
    pub fn hsl(self) -> String {
        let (r, g, b) = (
            self.red as f64 / 255.0,
            self.green as f64 / 255.0,
            self.blue as f64 / 255.0,
        );
        let high = r.max(g).max(b);
        let low = r.min(g).min(b);
        let light = (high + low) / 2.0;
        let delta = high - low;
        let (mut hue, mut saturation) = (0.0, 0.0);
        if delta > 0.0 {
            saturation = delta / (1.0 - (2.0 * light - 1.0).abs());
            hue = if high == r {
                60.0 * (((g - b) / delta) % 6.0)
            } else if high == g {
                60.0 * ((b - r) / delta + 2.0)
            } else {
                60.0 * ((r - g) / delta + 4.0)
            };
            if hue < 0.0 {
                hue += 360.0;
            }
        }
        let body = format!(
            "{}, {}%, {}%",
            hue.round() as i32,
            (saturation * 100.0).round() as i32,
            (light * 100.0).round() as i32
        );
        if self.alpha < 1.0 {
            format!("hsla({body}, {})", significant(self.alpha))
        } else {
            format!("hsl({body})")
        }
    }
}
fn significant(value: f64) -> String {
    if value == 0.0 {
        return "0".into();
    }
    let digits = (2 - value.abs().log10().floor() as i32).max(0) as usize;
    format!("{value:.digits$}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}
pub fn color(text: &str) -> Option<Color> {
    let value = text.trim();
    if let Some(hex) = value.strip_prefix('#') {
        if !matches!(hex.len(), 3 | 6 | 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
            return None;
        }
        let hex = if hex.len() == 3 {
            hex.chars().flat_map(|c| [c, c]).collect::<String>()
        } else {
            hex.into()
        };
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
            .collect::<Option<Vec<_>>>()?;
        return Some(Color {
            red: bytes[0],
            green: bytes[1],
            blue: bytes[2],
            alpha: bytes.get(3).map_or(1.0, |b| *b as f64 / 255.0),
        });
    }
    let lower = value.to_ascii_lowercase();
    let (name, body) = lower.split_once('(')?;
    let body = body.strip_suffix(')')?;
    if !matches!(name, "rgb" | "rgba" | "hsl" | "hsla") {
        return None;
    }
    let p: Vec<_> = body.split(',').map(str::trim).collect();
    if !matches!(p.len(), 3 | 4) {
        return None;
    }
    fn number(v: &str, max_digits: usize, decimal: bool) -> Option<f64> {
        let mut split = v.split('.');
        let before = split.next()?;
        let after = split.next();
        if before.len() > max_digits
            || !before.bytes().all(|b| b.is_ascii_digit())
            || split.next().is_some()
        {
            return None;
        }
        if let Some(a) = after {
            if !decimal || a.is_empty() || !a.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
        } else if before.is_empty() {
            return None;
        }
        v.parse().ok()
    }
    let alpha = if p.len() == 4 {
        number(p[3], usize::MAX, true)?
    } else {
        1.0
    };
    if !(0.0..=1.0).contains(&alpha) {
        return None;
    }
    if name.starts_with("rgb") {
        let r = number(p[0], 3, false)?;
        let g = number(p[1], 3, false)?;
        let b = number(p[2], 3, false)?;
        if [r, g, b].iter().any(|v| *v > 255.0) {
            return None;
        }
        return Some(Color {
            red: r as u8,
            green: g as u8,
            blue: b as u8,
            alpha,
        });
    }
    let h = number(p[0], 3, true)?;
    let s = number(p[1].strip_suffix('%')?, 3, true)? / 100.0;
    let l = number(p[2].strip_suffix('%')?, 3, true)? / 100.0;
    if h > 360.0 || s > 1.0 || l > 1.0 {
        return None;
    }
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h / 60.0) as u32 % 6 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Some(Color {
        red: ((r + m) * 255.0).round() as u8,
        green: ((g + m) * 255.0).round() as u8,
        blue: ((b + m) * 255.0).round() as u8,
        alpha,
    })
}
enum Detection {
    Jwt,
    Uuid(Option<u8>),
    Timestamp(i64),
    Color(Color),
    Base64,
}
fn detect(text: &str, truncated: bool) -> Option<Detection> {
    let value = text.trim();
    if value.is_empty() || value.len() > 16384 {
        return None;
    }
    let single = !value.chars().any(char::is_whitespace);
    if single && jwt_parts(value, !truncated).is_some() {
        return Some(Detection::Jwt);
    }
    if truncated {
        return None;
    }
    if single
        && value.len() == 36
        && value.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
    {
        let v = (value.as_bytes()[14] as char).to_digit(16)? as u8;
        return Some(Detection::Uuid((1..=8).contains(&v).then_some(v)));
    }
    if single && let Some(ms) = timestamp(value) {
        return Some(Detection::Timestamp(ms));
    }
    if let Some(color) = color(value) {
        return Some(Detection::Color(color));
    }
    if single
        && value.len() >= 16
        && !value.bytes().all(|b| b.is_ascii_hexdigit())
        && transform::base64_decode(value).is_ok()
    {
        return Some(Detection::Base64);
    }
    None
}
pub fn badge(text: &str, truncated: bool, now: i64, offset: i32) -> Option<String> {
    Some(match detect(text, truncated)? {
        Detection::Jwt => "JWT".into(),
        Detection::Uuid(v) => v.map_or("UUID".into(), |v| format!("UUID v{v}")),
        Detection::Color(c) => c.hex(),
        Detection::Base64 => "Base64".into(),
        Detection::Timestamp(ms) => {
            let rendered = local_time(ms, offset)?;
            if date(ms, offset)?.year() == date(now, offset)?.year() {
                rendered[5..16].into()
            } else {
                rendered[..16].into()
            }
        }
    })
}
fn row(label: &str, value: impl Into<String>, block: bool) -> Value {
    json!({"label":label,"value":value.into(),"block":block})
}
fn action(title: &str, transform: &str) -> Value {
    json!({"title":title,"transform":transform})
}
fn inspection(kind: &str, rows: Vec<Value>, actions: Vec<Value>) -> Value {
    json!({"kind":kind,"rows":rows,"actions":actions})
}
fn relative(ms: i64, now: i64, offset: i32) -> String {
    let delta = ms as i128 - now as i128;
    let secs = delta.unsigned_abs() / 1000;
    // Calendar years/months are complete intervals, matching the reference date formatter.
    let early = date(ms.min(now), offset);
    let late = date(ms.max(now), offset);
    let months = early
        .zip(late)
        .map(|(a, b)| {
            ((b.year() - a.year()) * 12 + b.month() as i32
                - a.month() as i32
                - i32::from((b.day(), b.time()) < (a.day(), a.time())))
            .max(0)
        })
        .unwrap_or(0);
    let (n, unit) = if months >= 12 {
        ((months / 12) as u128, "年")
    } else if months >= 1 {
        (months as u128, "个月")
    } else if secs >= 604800 {
        (secs / 604800, "周")
    } else if secs >= 86400 {
        (secs / 86400, "天")
    } else if secs >= 3600 {
        (secs / 3600, "小时")
    } else if secs >= 60 {
        (secs / 60, "分钟")
    } else {
        (secs, "秒")
    };
    format!("{n}{unit}{}", if delta < 0 { "前" } else { "后" })
}
pub fn full(text: &str, language: Option<&str>, now: i64, offset: i32) -> Option<Value> {
    let value = text.trim();
    if value.is_empty() {
        return None;
    }
    match detect(value, false) {
        Some(Detection::Jwt) => {
            let parts = jwt_parts(value, true)?;
            let header = String::from_utf8(base64url_data(parts[0])?).ok()?;
            let payload = String::from_utf8(base64url_data(parts[1])?).ok()?;
            let claims: Value = serde_json::from_str(&payload).ok()?;
            let mut rows = vec![
                row("Header", transform::reflow_json(&header, true), true),
                row("Payload", transform::reflow_json(&payload, true), true),
            ];
            for (key, label) in [("iat", "签发"), ("nbf", "生效"), ("exp", "过期")] {
                if let Some(sec) = claims[key].as_f64() {
                    let ms = (sec * 1000.0) as i64;
                    rows.push(row(label, local_time(ms, offset)?, false));
                    if key == "exp" {
                        let time = relative(ms, now, offset);
                        rows.push(row(
                            "状态",
                            if ms < now {
                                format!("已过期（{time}）")
                            } else {
                                format!("{time}过期")
                            },
                            false,
                        ));
                    }
                }
            }
            let mut result =
                inspection("JWT", rows, vec![action("粘贴 Payload JSON", "jwtPayload")]);
            result["note"] = json!("未验证签名");
            return Some(result);
        }
        Some(Detection::Uuid(v)) => {
            return Some(inspection(
                "UUID",
                vec![row(
                    "版本",
                    v.map_or("未知".into(), |v| format!("v{v}")),
                    false,
                )],
                vec![],
            ));
        }
        Some(Detection::Timestamp(ms)) => {
            return Some(inspection(
                "时间戳",
                vec![
                    row("单位", if value.len() == 13 { "毫秒" } else { "秒" }, false),
                    row("本地时间", local_time(ms, offset)?, false),
                    row("UTC", format!("{} UTC", local_time(ms, 0)?), false),
                    row("ISO 8601", iso(ms, offset, false)?, false),
                    row("相对", relative(ms, now, offset), false),
                ],
                vec![action("转为 ISO 8601 后粘贴", "timestampToISO")],
            ));
        }
        Some(Detection::Color(c)) => {
            return Some(inspection(
                "颜色",
                vec![
                    row("HEX", c.hex(), false),
                    row("RGB", c.rgb(), false),
                    row("HSL", c.hsl(), false),
                ],
                vec![
                    action("转为 rgb() 后粘贴", "colorToRGB"),
                    action("转为 hex 后粘贴", "colorToHex"),
                ],
            ));
        }
        Some(Detection::Base64) => {
            return Some(inspection(
                "Base64",
                vec![row(
                    "解码",
                    transform::base64_decode(value)
                        .ok()?
                        .graphemes(true)
                        .take(2000)
                        .collect::<String>(),
                    true,
                )],
                vec![action("解码后粘贴", "base64Decode")],
            ));
        }
        None => {}
    }
    if value.starts_with(['{', '[']) {
        match transform::validate_json(value) {
            Ok(v) => {
                let shape = if let Some(o) = v.as_object() {
                    format!("对象，{} 个键", o.len())
                } else if let Some(a) = v.as_array() {
                    format!("数组，{} 个元素", a.len())
                } else {
                    "值".into()
                };
                return Some(inspection(
                    "JSON",
                    vec![row("合法", "是", false), row("顶层", shape, false)],
                    vec![action("格式化后粘贴", "formatJSON")],
                ));
            }
            Err(e) if language == Some("json") => {
                return Some(inspection(
                    "JSON",
                    vec![row(
                        "合法",
                        format!("否：{}", e.replace("不是有效 JSON：", "")),
                        false,
                    )],
                    vec![],
                ));
            }
            _ => {}
        }
    }
    if let Some(url) = domain::web_link(value) {
        let (scheme, tail) = url.split_once("://")?;
        let boundary = tail.find(['/', '?', '#']).unwrap_or(tail.len());
        let authority = &tail[..boundary];
        let hostport = authority.rsplit('@').next()?;
        let host = if hostport.starts_with('[') {
            &hostport[..=hostport.find(']')?]
        } else {
            hostport.split(':').next()?
        };
        let mut rows = vec![row("协议", scheme, false), row("主机", host, false)];
        let rest = &tail[boundary..];
        let path = rest.split(['?', '#']).next()?;
        let decode = |v: &str| {
            percent_encoding::percent_decode_str(v)
                .decode_utf8_lossy()
                .into_owned()
        };
        if !path.is_empty() {
            rows.push(row("路径", decode(path), false));
        }
        if let Some((_, query)) = rest.split_once('?') {
            for item in query.split('#').next()?.split('&').take(30) {
                let (k, v) = item.split_once('=').unwrap_or((item, ""));
                rows.push(row(&format!("?{}", decode(k)), decode(v), false));
            }
        }
        return Some(inspection("链接", rows, vec![]));
    }
    let kind = language
        .and_then(language_name)
        .map_or("文本".into(), |n| format!("代码 · {n}"));
    Some(inspection(
        &kind,
        vec![
            row("行数", transform::lines(text).len().to_string(), false),
            row("字符", text.graphemes(true).count().to_string(), false),
        ],
        vec![action("Markdown 代码块后粘贴", "markdownCodeBlock")],
    ))
}

const LANGUAGES: [(&str, &str, &str); 20] = [
    ("json", "json", "JSON"),
    ("java", "java", "Java"),
    ("python", "py", "Python"),
    ("javascript", "js", "JavaScript"),
    ("typescript", "ts", "TypeScript"),
    ("yaml", "yaml", "YAML"),
    ("xml", "xml", "XML"),
    ("css", "css", "CSS"),
    ("bash", "sh", "Shell"),
    ("swift", "swift", "Swift"),
    ("sql", "sql", "SQL"),
    ("cpp", "cpp", "C++"),
    ("csharp", "cs", "C#"),
    ("go", "go", "Go"),
    ("ruby", "rb", "Ruby"),
    ("php", "php", "PHP"),
    ("rust", "rs", "Rust"),
    ("ini", "ini", "INI"),
    ("markdown", "md", "Markdown"),
    ("", "", ""),
];
fn language_name(value: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .find(|(name, _, _)| !name.is_empty() && *name == value)
        .map(|(_, _, display)| *display)
}
pub struct ExportFormat {
    pub extension: &'static str,
    pub original_type: Option<&'static str>,
}
pub fn export_format(
    text: &str,
    language: Option<&str>,
    types: &[&str],
    markup: Option<&str>,
) -> ExportFormat {
    let rich = types.contains(&"text/rtf") || types.contains(&"text/html");
    let markup = markup
        .unwrap_or("")
        .graphemes(true)
        .take(65536)
        .collect::<String>()
        .to_lowercase();
    if let Some(language) = language
        && let Some((_, ext, _)) = LANGUAGES
            .iter()
            .find(|(l, _, _)| !l.is_empty() && *l == language)
        && (!rich
            || [
                "<pre",
                "<code",
                "monospace",
                "menlo",
                "monaco",
                "courier",
                "sfmono",
                "sf mono",
                "consolas",
                "jetbrains mono",
                "fira code",
            ]
            .iter()
            .any(|s| markup.contains(s)))
    {
        let start = text
            .graphemes(true)
            .take(512)
            .collect::<String>()
            .to_lowercase();
        return ExportFormat {
            extension: if language == "xml"
                && (start.contains("<html") || start.contains("<!doctype html"))
            {
                "html"
            } else {
                ext
            },
            original_type: None,
        };
    }
    if looks_like_markdown(text) {
        return ExportFormat {
            extension: "md",
            original_type: None,
        };
    }
    for (t, ext) in [("text/rtf", "rtf"), ("text/html", "html")] {
        if types.contains(&t) {
            return ExportFormat {
                extension: ext,
                original_type: Some(t),
            };
        }
    }
    ExportFormat {
        extension: "txt",
        original_type: None,
    }
}
fn looks_like_markdown(text: &str) -> bool {
    let text = text.graphemes(true).take(16384).collect::<String>();
    let rows = transform::lines(&text);
    let rows: Vec<_> = rows
        .iter()
        .filter(|(s, _)| !s.is_empty())
        .take(400)
        .map(|(s, _)| *s)
        .collect();
    let after_space = |s: &str| {
        s.strip_prefix(' ')
            .is_some_and(|s| s.chars().next().is_some_and(|c| !c.is_whitespace()))
    };
    let heading = rows.iter().any(|s| {
        let n = s.bytes().take_while(|b| *b == b'#').count();
        (1..=6).contains(&n) && after_space(&s[n..])
    });
    let fence = rows
        .iter()
        .any(|s| s.starts_with("```") || s.starts_with("~~~"));
    let lists = rows
        .iter()
        .filter(|s| {
            let s = s.trim_start();
            if s.starts_with(['-', '*', '+']) {
                after_space(&s[1..])
            } else {
                let n = s.bytes().take_while(u8::is_ascii_digit).count();
                n > 0 && s[n..].strip_prefix('.').is_some_and(after_space)
            }
        })
        .count()
        >= 2;
    let links = rows.iter().any(|s| {
        let Some(open) = s.find('[') else {
            return false;
        };
        let Some(mid) = s[open + 1..].find("](").map(|i| open + 1 + i) else {
            return false;
        };
        let Some(end) = s[mid + 2..].find(')').map(|i| mid + 2 + i) else {
            return false;
        };
        mid > open + 1 && end > mid + 2 && !s[mid + 2..end].chars().any(char::is_whitespace)
    });
    [heading, fence, lists, links]
        .iter()
        .filter(|v| **v)
        .count()
        >= 2
}
pub fn sanitized_file_name(value: &str) -> Option<String> {
    let mut name = String::new();
    for g in value.graphemes(true) {
        if g.chars().any(|c| {
            c.is_control()
                || matches!(
                    c,
                    '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\u{2028}' | '\u{2029}'
                )
        }) {
            name.push('-');
        } else {
            name.push_str(g);
        }
    }
    let name = name.trim().trim_start_matches('.');
    let mut out = String::new();
    for g in name.graphemes(true) {
        if out.len() + g.len() > 200 {
            break;
        }
        out.push_str(g);
    }
    out = out.trim().trim_end_matches(['.', ' ']).to_string();
    if out.is_empty() {
        return None;
    }
    let stem = out.split('.').next()?.trim_end().to_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ["COM", "LPT"].iter().any(|p| {
            stem.strip_prefix(p).is_some_and(|suffix| {
                matches!(
                    suffix,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        });
    if reserved {
        if out.len() == 200 {
            let last = out.grapheme_indices(true).next_back()?.0;
            out.truncate(last);
            out = out.trim_end_matches(['.', ' ']).to_string();
        }
        out.insert(0, '_');
    }
    Some(out)
}
pub fn export_base_name(title: Option<&str>, copied_at: i64, offset: i32) -> Option<String> {
    if let Some(name) = title.and_then(sanitized_file_name) {
        return Some(name);
    }
    let d = date(copied_at, offset)?;
    Some(format!(
        "LocalClip-{:04}{:02}{:02}-{:02}{:02}{:02}",
        d.year(),
        d.month() as u8,
        d.day(),
        d.hour(),
        d.minute(),
        d.second()
    ))
}
pub fn file_name(base: &str, index: Option<usize>, extension: &str) -> Option<String> {
    if !LANGUAGES
        .iter()
        .any(|(_, e, _)| !e.is_empty() && *e == extension)
        && !["txt", "md", "rtf", "html", "url", "png", "jpg", "bmp"].contains(&extension)
    {
        return None;
    }
    Some(format!(
        "{}{}.{extension}",
        sanitized_file_name(base)?,
        index.map_or(String::new(), |i| format!("-{i}"))
    ))
}
