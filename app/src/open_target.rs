//! 从历史原始表示解析显式打开目标，不接受前端任意命令或协议。by AI.Coding
use localclip_core::domain::{Kind, Payload, web_link};

#[derive(Debug, PartialEq, Eq)]
pub struct OpenTarget {
    pub value: String,
    pub file: bool,
}

/// 文件引用优先；网页只接收既有识别规则和明确声明的安全协议。
pub fn targets(payload: &Payload) -> Vec<OpenTarget> {
    let files: Vec<_> = payload
        .items
        .iter()
        .filter_map(|item| item.file_path())
        .map(|value| OpenTarget { value, file: true })
        .collect();
    if !files.is_empty() {
        return files;
    }
    let mut links: Vec<_> = payload
        .items
        .iter()
        .filter_map(|item| item.representation("text/uri"))
        .filter_map(|bytes| std::str::from_utf8(bytes).ok())
        .filter_map(safe_link)
        .collect();
    if links.is_empty()
        && payload.kind() == Kind::Url
        && let Some(link) = payload.plain_text().as_deref().and_then(web_link)
    {
        links.push(link);
    }
    links
        .into_iter()
        .map(|value| OpenTarget { value, file: false })
        .collect()
}

/// 禁止控制字符及 shell 自定义协议；mailto 不走网页域名回退。
fn safe_link(value: &str) -> Option<String> {
    let value = value.trim();
    if value.chars().any(char::is_control) || value.len() > 8192 {
        return None;
    }
    if value
        .get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("mailto:"))
        && value.len() > 7
    {
        return Some(value.into());
    }
    if value.to_ascii_lowercase().starts_with("https://")
        || value.to_ascii_lowercase().starts_with("http://")
    {
        return web_link(value);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use localclip_core::domain::{Item, Representation};
    #[test]
    fn only_declared_safe_links_or_recognized_web_text_open() {
        assert_eq!(
            targets(&Payload::text("example.com"))[0].value,
            "http://example.com"
        );
        assert!(targets(&Payload::text("plain generated text")).is_empty());
        for value in [
            "javascript:alert(1)",
            "powershell:evil",
            "file:///C:/test.exe",
            "https://example.com\0evil",
        ] {
            let payload = Payload {
                items: vec![Item {
                    representations: vec![Representation {
                        format: "text/uri".into(),
                        data: value.as_bytes().into(),
                    }],
                    ..Default::default()
                }],
            };
            assert!(targets(&payload).is_empty());
        }
        assert_eq!(
            safe_link("mailto:test@example.com"),
            Some("mailto:test@example.com".into())
        );
    }
    #[test]
    fn multiple_files_remain_separate_for_explicit_selection() {
        let items = ["file:///C:/first.txt", "file:///C:/second.txt"]
            .into_iter()
            .map(|path| Item {
                representations: vec![Representation {
                    format: "text/uri-list".into(),
                    data: path.as_bytes().into(),
                }],
                ..Default::default()
            })
            .collect();
        let result = targets(&Payload { items });
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|t| t.file));
        assert_eq!(result[1].value, "/C:/second.txt");
    }
}
