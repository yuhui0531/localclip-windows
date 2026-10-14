pub use crate::web::web_link;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const MIB: u64 = 1_048_576;
pub const DAY_MS: i64 = 86_400_000;
pub const FORMATS: [&str; 8] = [
    "text/plain",
    "text/uri",
    "text/html",
    "text/rtf",
    "image/png",
    "image/jpeg",
    "image/x-win-dibv5",
    "text/uri-list",
];

#[derive(Clone, PartialEq, Eq)]
pub struct Representation {
    pub format: String,
    pub data: Vec<u8>,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Item {
    pub representations: Vec<Representation>,
    pub declared_types: Vec<String>,
    pub plain_text: Option<String>,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct Payload {
    pub items: Vec<Item>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Text,
    Url,
    RichText,
    Image,
    File,
    Mixed,
}
impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Url => "url",
            Self::RichText => "richText",
            Self::Image => "image",
            Self::File => "file",
            Self::Mixed => "mixed",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "text" => Self::Text,
            "url" => Self::Url,
            "richText" => Self::RichText,
            "image" => Self::Image,
            "file" => Self::File,
            "mixed" => Self::Mixed,
            _ => return None,
        })
    }
}

pub fn hash(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

impl Item {
    pub fn representation(&self, format: &str) -> Option<&[u8]> {
        self.representations
            .iter()
            .find(|r| r.format == format)
            .map(|r| r.data.as_slice())
    }
    pub fn file_path(&self) -> Option<String> {
        let text = std::str::from_utf8(self.representation("text/uri-list")?).ok()?;
        let rest = text.strip_prefix("file://")?;
        let start = rest.find('/')?;
        // The adapter produces local/UNC file URLs; reject foreign URI schemes and controls.
        let host = &rest[..start];
        let path = percent_encoding::percent_decode_str(rest[start..].split(['?', '#']).next()?)
            .decode_utf8()
            .ok()?;
        if path.contains('\0') || !path.starts_with('/') {
            return None;
        }
        if host.is_empty() || host.eq_ignore_ascii_case("localhost") {
            Some(path.into_owned())
        } else if host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.'))
        {
            Some(format!("//{host}{path}"))
        } else {
            None
        }
    }
}

impl Payload {
    pub fn text(value: &str) -> Self {
        Self {
            items: vec![Item {
                representations: vec![Representation {
                    format: "text/plain".into(),
                    data: value.as_bytes().into(),
                }],
                declared_types: vec!["text/plain".into()],
                plain_text: Some(value.into()),
            }],
        }
    }
    pub fn byte_count(&self) -> u64 {
        self.items
            .iter()
            .flat_map(|i| &i.representations)
            .map(|r| r.data.len() as u64)
            .sum()
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.items.len() > 32 || self.byte_count() > 50 * MIB {
            return Err("tooLarge");
        }
        if self.items.is_empty() || self.items.iter().any(|i| i.representations.is_empty()) {
            return Err("unsupported");
        }
        for item in &self.items {
            let mut seen = BTreeSet::new();
            if item
                .representations
                .iter()
                .any(|r| !FORMATS.contains(&r.format.as_str()) || !seen.insert(&r.format))
            {
                return Err("unsupported");
            }
            if item.representation("text/uri-list").is_some() && item.file_path().is_none() {
                return Err("unsupported");
            }
            if item
                .representation("text/uri")
                .and_then(|r| std::str::from_utf8(r).ok())
                .is_some_and(|s| s.to_ascii_lowercase().starts_with("file:"))
                && item.file_path().is_none()
            {
                return Err("unsupported");
            }
        }
        if self
            .items
            .iter()
            .any(|i| i.representation("text/uri-list").is_some())
            && self.items.iter().any(|i| i.file_path().is_none())
        {
            return Err("unsupported");
        }
        Ok(())
    }
    pub fn canonical(&self) -> Vec<u8> {
        fn integer(out: &mut Vec<u8>, v: usize) {
            out.extend_from_slice(&(v as u64).to_be_bytes());
        }
        fn field(out: &mut Vec<u8>, v: &[u8]) {
            integer(out, v.len());
            out.extend_from_slice(v);
        }
        let mut result = Vec::new();
        integer(&mut result, 1);
        integer(&mut result, self.items.len());
        for item in &self.items {
            let mut reps: Vec<_> = item.representations.iter().collect();
            reps.sort_by(|a, b| a.format.as_bytes().cmp(b.format.as_bytes()));
            integer(&mut result, reps.len());
            for rep in reps {
                field(&mut result, rep.format.as_bytes());
                field(&mut result, &rep.data);
            }
        }
        result
    }
    pub fn fingerprint(&self) -> String {
        hash(&self.canonical())
    }
    pub fn plain_text(&self) -> Option<String> {
        if self.items.is_empty() {
            return None;
        }
        self.items
            .iter()
            .map(|i| i.plain_text.as_deref())
            .collect::<Option<Vec<_>>>()
            .map(|v| v.join("\n"))
    }
    pub fn kind(&self) -> Kind {
        let types: BTreeSet<_> = self
            .items
            .iter()
            .flat_map(|i| &i.representations)
            .map(|r| r.format.as_str())
            .collect();
        if types.contains("text/uri-list") {
            return if self.items.iter().all(|i| i.file_path().is_some()) {
                Kind::File
            } else {
                Kind::Mixed
            };
        }
        if ["image/png", "image/jpeg", "image/x-win-dibv5"]
            .iter()
            .any(|t| types.contains(t))
        {
            return if ["text/plain", "text/uri", "text/rtf", "text/html"]
                .iter()
                .any(|t| types.contains(t))
            {
                Kind::Mixed
            } else {
                Kind::Image
            };
        }
        if types.contains("text/rtf") || types.contains("text/html") {
            return Kind::RichText;
        }
        if types.contains("text/uri") {
            return Kind::Url;
        }
        if self.items.len() == 1
            && self.items[0]
                .representation("text/plain")
                .and_then(|d| std::str::from_utf8(d).ok())
                .and_then(web_link)
                .is_some()
        {
            return Kind::Url;
        }
        Kind::Text
    }
}

pub fn is_image_file(path: &str) -> bool {
    path.rsplit(['/', '\\'])
        .next()
        .and_then(|n| n.rsplit_once('.'))
        .is_some_and(|(stem, ext)| {
            !stem.is_empty()
                && matches!(
                    ext.to_ascii_lowercase().as_str(),
                    "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tif" | "tiff" | "webp" | "ico"
                )
        })
}

#[derive(Clone, PartialEq, Eq)]
pub struct Source {
    pub app_id: Option<String>,
    pub name: String,
    pub basis: String,
}
impl Default for Source {
    fn default() -> Self {
        Self {
            app_id: None,
            name: "未知".into(),
            basis: "unknown".into(),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Settings {
    pub retention_days: u16,
    pub quota_bytes: u64,
    pub capture_enabled: bool,
    pub pause_until: Option<i64>,
    pub excluded_app_ids: BTreeSet<String>,
    pub revision: i64,
    pub move_used_to_front: bool,
    pub preferences: Preferences,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Shortcut {
    pub modifiers: BTreeSet<String>,
    pub key: String,
}
impl Shortcut {
    pub fn validate(&self) -> bool {
        !self.modifiers.is_empty()
            && self
                .modifiers
                .iter()
                .all(|m| ["ctrl", "shift", "alt", "win"].contains(&m.as_str()))
            && ((self.key.len() == 1
                && self
                    .key
                    .bytes()
                    .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()))
                || self
                    .key
                    .strip_prefix('F')
                    .and_then(|n| n.parse::<u8>().ok())
                    .is_some_and(|n| (1..=24).contains(&n))
                || [
                    "Space",
                    "Tab",
                    "Enter",
                    "Escape",
                    "Insert",
                    "Delete",
                    "Home",
                    "End",
                    "PageUp",
                    "PageDown",
                    "ArrowLeft",
                    "ArrowRight",
                    "ArrowUp",
                    "ArrowDown",
                ]
                .contains(&self.key.as_str()))
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Preferences {
    pub default_action: String,
    pub default_format: String,
    pub panel_height: u16,
    pub pause_when_excluded_frontmost: bool,
    pub hide_contents: bool,
    pub appearance: String,
    pub compact_layout: bool,
    pub capture_sounds: bool,
    pub show_menu_icon: bool,
    pub timing_enabled: bool,
    pub auto_check_updates: bool,
    pub ocr_languages: Vec<String>,
    pub shortcut: Shortcut,
    pub queue_shortcut: Shortcut,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            default_action: "directPaste".into(),
            default_format: "original".into(),
            panel_height: 340,
            pause_when_excluded_frontmost: false,
            hide_contents: false,
            appearance: "system".into(),
            compact_layout: false,
            capture_sounds: false,
            show_menu_icon: true,
            timing_enabled: false,
            auto_check_updates: true,
            ocr_languages: vec!["zh-Hans".into(), "zh-Hant".into(), "en-US".into()],
            shortcut: Shortcut {
                modifiers: ["ctrl".into(), "shift".into()].into(),
                key: "V".into(),
            },
            queue_shortcut: Shortcut {
                modifiers: ["ctrl".into(), "shift".into(), "alt".into()].into(),
                key: "V".into(),
            },
        }
    }
}
impl Preferences {
    pub fn validate(&self) -> bool {
        ["directPaste", "copy"].contains(&self.default_action.as_str())
            && ["original", "plainText"].contains(&self.default_format.as_str())
            && (220..=520).contains(&self.panel_height)
            && ["system", "light", "dark"].contains(&self.appearance.as_str())
            && !self.ocr_languages.is_empty()
            && self.ocr_languages.len() <= 3
            && self
                .ocr_languages
                .iter()
                .all(|v| ["zh-Hans", "zh-Hant", "en-US"].contains(&v.as_str()))
            && self.shortcut.validate()
            && self.queue_shortcut.validate()
            && self.shortcut != self.queue_shortcut
    }
    pub fn document(&self) -> serde_json::Value {
        serde_json::json!({"defaultAction":self.default_action,"defaultFormat":self.default_format,"panelHeight":self.panel_height,"pauseWhenExcludedFrontmost":self.pause_when_excluded_frontmost,"hideContents":self.hide_contents,"appearance":self.appearance,"compactLayout":self.compact_layout,"captureSounds":self.capture_sounds,"showMenuIcon":self.show_menu_icon,"timingEnabled":self.timing_enabled,"autoCheckUpdates":self.auto_check_updates,"ocrLanguages":self.ocr_languages,"shortcut":{"modifiers":self.shortcut.modifiers,"key":self.shortcut.key},"queueShortcut":{"modifiers":self.queue_shortcut.modifiers,"key":self.queue_shortcut.key}})
    }
    pub fn from_document(v: &serde_json::Value) -> Option<Self> {
        let shortcut = |key: &str| -> Option<Shortcut> {
            Some(Shortcut {
                modifiers: v[key]["modifiers"]
                    .as_array()?
                    .iter()
                    .map(|m| m.as_str().map(String::from))
                    .collect::<Option<_>>()?,
                key: v[key]["key"].as_str()?.into(),
            })
        };
        let result = Self {
            default_action: v["defaultAction"].as_str()?.into(),
            default_format: v["defaultFormat"].as_str()?.into(),
            panel_height: u16::try_from(v["panelHeight"].as_u64()?).ok()?,
            pause_when_excluded_frontmost: v["pauseWhenExcludedFrontmost"].as_bool()?,
            hide_contents: v["hideContents"].as_bool()?,
            appearance: v["appearance"].as_str()?.into(),
            compact_layout: v["compactLayout"].as_bool()?,
            capture_sounds: v["captureSounds"].as_bool()?,
            show_menu_icon: v["showMenuIcon"].as_bool()?,
            timing_enabled: v["timingEnabled"].as_bool()?,
            auto_check_updates: v["autoCheckUpdates"].as_bool()?,
            ocr_languages: match v.get("ocrLanguages") {
                Some(value) => value
                    .as_array()?
                    .iter()
                    .map(|v| v.as_str().map(String::from))
                    .collect::<Option<Vec<_>>>()?,
                None => Self::default().ocr_languages,
            },
            shortcut: shortcut("shortcut")?,
            queue_shortcut: shortcut("queueShortcut")?,
        };
        result.validate().then_some(result)
    }
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            retention_days: 30,
            quota_bytes: 500 * MIB,
            capture_enabled: true,
            pause_until: None,
            excluded_app_ids: BTreeSet::new(),
            revision: 0,
            move_used_to_front: true,
            preferences: Preferences::default(),
        }
    }
}
impl Settings {
    pub fn rejection(&self, source: &Source, sensitive: bool, now: i64) -> Option<&'static str> {
        if sensitive {
            return Some("sensitive");
        }
        if !self.capture_enabled || self.pause_until.is_some_and(|t| t > now) {
            return Some("paused");
        }
        if source.basis == "owner"
            && source
                .app_id
                .as_ref()
                .is_some_and(|id| self.excluded_app_ids.contains(id))
        {
            return Some("excluded");
        }
        None
    }
    pub fn validate(&self) -> bool {
        self.retention_days <= 365
            && (100 * MIB..=4096 * MIB).contains(&self.quota_bytes)
            && self.preferences.validate()
    }
}
