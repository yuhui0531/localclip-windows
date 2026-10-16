//! 合并与转换输出预算，避免读取多条记录造成无界内存增长。by AI.Coding
use localclip_core::{
    domain::{Payload, Representation},
    transform::{Join, Transform},
};

pub const LIMIT: u64 = 50 * 1024 * 1024;

/// 输入和派生输出均执行 50 MiB 上限。
pub fn compose(
    records: Vec<Payload>,
    plain: bool,
    join: &str,
    transform: Option<&str>,
    language: Option<&str>,
    offset: i32,
) -> Result<Payload, &'static str> {
    if records.is_empty() {
        return Err("notFound");
    }
    if records
        .iter()
        .map(Payload::byte_count)
        .try_fold(0u64, |sum, n| sum.checked_add(n))
        .is_none_or(|n| n > LIMIT)
    {
        return Err("outputTooLarge");
    }
    let style = Join::parse(join).ok_or("invalidRequest")?;
    if let Some(transform) = transform {
        let values = records
            .iter()
            .map(|p| p.plain_text().ok_or("unsupported"))
            .collect::<Result<Vec<_>, _>>()?;
        let value = style.join(&values.iter().map(String::as_str).collect::<Vec<_>>());
        return Transform::parse(transform)
            .ok_or("invalidRequest")?
            .apply(&value, language, offset)
            .map_err(|_| "transformUnavailable")
            .and_then(|v| {
                if v.len() as u64 > LIMIT {
                    Err("outputTooLarge")
                } else {
                    Ok(Payload::text(&v))
                }
            });
    }
    if records.len() == 1 && !plain && join == "newline" {
        return Ok(records.into_iter().next().unwrap());
    }
    if records
        .iter()
        .flat_map(|p| &p.items)
        .all(|i| i.file_path().is_some())
        && !plain
        && join == "newline"
    {
        let payload = Payload {
            items: records.into_iter().flat_map(|p| p.items).collect(),
        };
        payload.validate()?;
        return Ok(payload);
    }
    let values = records
        .iter()
        .map(|p| p.plain_text().ok_or("unsupported"))
        .collect::<Result<Vec<_>, _>>()?;
    let text = style.join(&values.iter().map(String::as_str).collect::<Vec<_>>());
    if text.len() as u64 > LIMIT {
        return Err("outputTooLarge");
    }
    let mut payload = Payload::text(&text);
    if !plain && join == "newline" {
        let items = records.iter().flat_map(|p| &p.items).collect::<Vec<_>>();
        if items.iter().any(|i| {
            i.representations
                .iter()
                .any(|r| r.format.starts_with("image/"))
                || i.file_path().is_some()
        }) {
            return Err("unsupportedCombination");
        }
        if items
            .iter()
            .any(|i| i.representation("text/html").is_some())
        {
            let html = items
                .iter()
                .map(|i| {
                    i.representation("text/html")
                        .and_then(|v| std::str::from_utf8(v).ok())
                        .map(String::from)
                        .unwrap_or_else(|| escape_html(i.plain_text.as_deref().unwrap_or("")))
                })
                .collect::<Vec<_>>()
                .join("<br>");
            payload.items[0].representations.push(Representation {
                format: "text/html".into(),
                data: html.into_bytes(),
            });
        }
        #[cfg(windows)]
        if items.iter().any(|i| i.representation("text/rtf").is_some()) {
            payload.items[0].representations.push(Representation {
                format: "text/rtf".into(),
                data: crate::win::rich::merge(&items)?,
            });
        }
    }
    payload.validate()?;
    Ok(payload)
}
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\n', "<br>")
}
