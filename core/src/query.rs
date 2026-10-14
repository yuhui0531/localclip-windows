use crate::domain::{DAY_MS, Kind, Source};
use std::collections::BTreeSet;
use time::{Date, Month, UtcOffset};

#[derive(Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
}
#[derive(Default)]
pub struct ParsedQuery {
    pub literal: String,
    pub kind: Option<Kind>,
    pub favorites: bool,
    pub source_app_id: Option<String>,
    pub category_ids: BTreeSet<String>,
    pub from: Option<i64>,
    pub until: Option<i64>,
    pub labels: Vec<String>,
    pub problems: Vec<String>,
}

pub fn parse(
    text: &str,
    sources: &[Source],
    categories: &[Category],
    now: i64,
    local_offset: i32,
) -> ParsedQuery {
    let mut result = ParsedQuery {
        literal: text.into(),
        ..Default::default()
    };
    let mut edits = Vec::new();
    let mut category_names = Vec::new();
    let mut offset = 0;
    for token in text.split_inclusive(char::is_whitespace) {
        let part = token.trim_end_matches(char::is_whitespace);
        let start = offset;
        offset += token.len();
        if part.is_empty() {
            continue;
        }
        let end = start + part.len();
        if part.len() > 2
            && part.starts_with('"')
            && part.ends_with('"')
            && key(&part[1..part.len() - 1]).is_some()
        {
            edits.push((start..end, Some(part[1..part.len() - 1].to_string())));
            continue;
        }
        let Some((name, value)) = key(part) else {
            continue;
        };
        let lower = value.to_lowercase();
        match name.as_str() {
            "type" => {
                let (kind, label) = match lower.as_str() {
                    "text" | "文本" => (Kind::Text, "文本"),
                    "url" | "link" | "链接" => (Kind::Url, "链接"),
                    "rich" | "richtext" | "富文本" => (Kind::RichText, "富文本"),
                    "image" | "img" | "图片" => (Kind::Image, "图片"),
                    "file" | "文件" => (Kind::File, "文件"),
                    "mixed" | "混合" => (Kind::Mixed, "混合"),
                    _ => continue,
                };
                result.kind = Some(kind);
                result.labels.push(format!("类型 {label}"));
            }
            "is" => {
                if !matches!(lower.as_str(), "fav" | "favorite" | "favorites" | "收藏") {
                    continue;
                }
                result.favorites = true;
                result.labels.push("收藏".into());
            }
            "app" => {
                let mut matches: Vec<_> = sources
                    .iter()
                    .filter(|s| {
                        s.name.to_lowercase() == lower
                            || s.app_id
                                .as_ref()
                                .is_some_and(|id| id.to_lowercase() == lower)
                    })
                    .collect();
                if matches.is_empty() {
                    matches = sources
                        .iter()
                        .filter(|s| {
                            s.name.to_lowercase().starts_with(&lower)
                                || s.app_id
                                    .as_ref()
                                    .is_some_and(|id| id.to_lowercase().starts_with(&lower))
                        })
                        .collect();
                }
                match matches.as_slice() {
                    [source] => {
                        result.source_app_id = Some(source.app_id.clone().unwrap_or_default());
                        result.labels.push(format!("来源 {}", source.name));
                    }
                    [] => result.problems.push(format!("未找到来源 {value}")),
                    _ => result.problems.push(format!(
                        "来源“{value}”匹配多个：{}",
                        matches
                            .iter()
                            .take(5)
                            .map(|s| s.name.as_str())
                            .collect::<Vec<_>>()
                            .join("、")
                    )),
                }
            }
            "in" => {
                if let Some(category) = categories.iter().find(|c| c.name.to_lowercase() == lower) {
                    result.category_ids.insert(category.id.clone());
                    category_names.push(category.name.as_str());
                } else {
                    result.problems.push(format!("未找到分类 {value}"));
                }
            }
            _ => {
                let after = name == "after";
                let Some((bound, label)) = date(value, now, after, local_offset) else {
                    continue;
                };
                if after {
                    result.from = Some(bound);
                } else {
                    result.until = Some(bound);
                }
                result.labels.push(label);
            }
        }
        edits.push((start..end, None));
    }
    if !category_names.is_empty() {
        result
            .labels
            .push(format!("分类 {}", category_names.join("、")));
    }
    for (mut range, replacement) in edits.into_iter().rev() {
        if let Some(replacement) = replacement {
            result.literal.replace_range(range, &replacement);
            continue;
        }
        if result.literal.as_bytes().get(range.end) == Some(&b' ') {
            range.end += 1;
        } else if range.start > 0 && result.literal.as_bytes()[range.start - 1] == b' ' {
            range.start -= 1;
        }
        result.literal.replace_range(range, "");
    }
    result
}

fn key(token: &str) -> Option<(String, &str)> {
    let (name, value) = token.split_once(':')?;
    let name = name.to_lowercase();
    (matches!(
        name.as_str(),
        "app" | "type" | "is" | "in" | "after" | "before"
    ) && !value.is_empty()
        && !value.starts_with(':'))
    .then_some((name, value))
}

fn date(value: &str, now: i64, after: bool, offset: i32) -> Option<(i64, String)> {
    if let Some(days) = value.strip_suffix('d')
        && (1..=4).contains(&days.len())
        && days.bytes().all(|b| b.is_ascii_digit())
    {
        let days: i64 = days.parse().ok()?;
        return Some((
            now.checked_sub(days * DAY_MS)?,
            if after {
                format!("最近 {days} 天")
            } else {
                format!("{days} 天前")
            },
        ));
    }
    if value.len() != 10
        || value.as_bytes()[4] != b'-'
        || value.as_bytes()[7] != b'-'
        || !value
            .bytes()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return None;
    }
    let day = Date::from_calendar_date(
        value[..4].parse().ok()?,
        Month::try_from(value[5..7].parse::<u8>().ok()?).ok()?,
        value[8..].parse().ok()?,
    )
    .ok()?;
    Some((
        day.midnight()
            .assume_offset(UtcOffset::from_whole_seconds(offset).ok()?)
            .unix_timestamp()
            * 1000,
        format!("{}{}", if after { "从 " } else { "早于 " }, value),
    ))
}
