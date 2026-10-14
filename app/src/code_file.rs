//! 代码文件识别与严格解码，正文不进入历史数据库。by AI.Coding
pub const PER_FILE_LIMIT: usize = 4 * 1024 * 1024;
pub const TOTAL_LIMIT: usize = 8 * 1024 * 1024;

pub fn supported(name: &str) -> bool {
    let name = name.to_lowercase();
    ["dockerfile", "makefile", "gnumakefile"].contains(&name.as_str())
        || name.starts_with("dockerfile.")
        || name.rsplit_once('.').is_some_and(|(_, ext)| {
            [
                "json",
                "jsonc",
                "java",
                "py",
                "pyw",
                "js",
                "mjs",
                "cjs",
                "jsx",
                "ts",
                "tsx",
                "yaml",
                "yml",
                "xml",
                "html",
                "htm",
                "css",
                "scss",
                "sass",
                "sh",
                "bash",
                "zsh",
                "swift",
                "c",
                "h",
                "cc",
                "cpp",
                "hpp",
                "cs",
                "go",
                "rs",
                "rb",
                "php",
                "sql",
                "toml",
                "md",
                "dockerfile",
                "kt",
                "kts",
                "m",
                "mm",
                "vue",
                "svelte",
                "lua",
                "pl",
                "r",
                "scala",
                "dart",
                "ini",
                "cfg",
                "conf",
                "gradle",
                "ps1",
                "graphql",
                "gql",
                "proto",
                "tf",
            ]
            .contains(&ext)
        })
}

/// 只接受 UTF-8 或显式 BOM 的 UTF-16/32；拒绝替换解码及二进制控制符。
pub fn decode(bytes: &[u8]) -> Result<String, &'static str> {
    let invalid = "codeFileEncoding";
    let text = if bytes.starts_with(&[0xff, 0xfe, 0, 0]) || bytes.starts_with(&[0, 0, 0xfe, 0xff]) {
        let little = bytes[0] == 0xff;
        let body = &bytes[4..];
        if !body.len().is_multiple_of(4) {
            return Err(invalid);
        }
        body.chunks_exact(4)
            .map(|chunk| {
                let word = chunk.try_into().unwrap();
                char::from_u32(if little {
                    u32::from_le_bytes(word)
                } else {
                    u32::from_be_bytes(word)
                })
                .ok_or(invalid)
            })
            .collect::<Result<String, _>>()?
    } else if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        let little = bytes[0] == 0xff;
        let body = &bytes[2..];
        if !body.len().is_multiple_of(2) {
            return Err(invalid);
        }
        let words = body
            .chunks_exact(2)
            .map(|chunk| {
                let word = chunk.try_into().unwrap();
                if little {
                    u16::from_le_bytes(word)
                } else {
                    u16::from_be_bytes(word)
                }
            })
            .collect::<Vec<_>>();
        String::from_utf16(&words).map_err(|_| invalid)?
    } else {
        std::str::from_utf8(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes))
            .map_err(|_| invalid)?
            .to_owned()
    };
    if text
        .chars()
        .any(|c| c == '\x7f' || c < ' ' && !['\t', '\n', '\x0c', '\r', '\x1b'].contains(&c))
    {
        return Err(invalid);
    }
    Ok(text)
}
