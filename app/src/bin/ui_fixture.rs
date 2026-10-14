//! 仅生成测试历史，保持采集暂停。by AI.Coding
use localclip_core::{
    domain::{Item, Payload, Representation, Settings, Source},
    store::{HistoryStore, Options, Snapshot},
};
use std::{io::Cursor, path::PathBuf, sync::Arc};
/// 可选第二个参数引用由测试驱动创建的 Windows 文件，用于真实拖出验收。
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_url = std::env::args().nth(2);
    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("new output directory required")?,
    );
    if root.exists() {
        return Err("output directory must be new".into());
    }
    std::fs::create_dir_all(&root)?;
    std::fs::write(
        root.join(".localclip-test-data"),
        b"generated UI fixture; capture disabled",
    )?;
    let settings = Settings {
        capture_enabled: false,
        ..Settings::default()
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_millis() as i64;
    let store = HistoryStore::open(
        root.join("history.sqlite"),
        Options {
            settings,
            available_space: Arc::new(|| Ok(u64::MAX)),
            ..Default::default()
        },
    )?;
    let mut active = store.settings()?;
    active.capture_enabled = true;
    active = store.update_settings(active)?;
    for i in 0..120 {
        let text = match i % 5 {
            0 => format!("生成的测试记录 {i}\n项目交付清单：设计评审、接口验证、验收记录。"),
            1 => format!("https://example.com/generated/{i}"),
            2 => format!("{{\"generated\":true,\"index\":{i}}}"),
            3 => format!("function generated{i}() {{ return true; }}"),
            _ => format!("测试便笺 {i}\r\n保留换行与中文。"),
        };
        let payload = if i == 0 && file_url.is_some() {
            // 文件只由外部测试驱动在隔离目录创建，不读取个人文件。
            Payload {
                items: vec![Item {
                    representations: vec![Representation {
                        format: "text/uri-list".into(),
                        data: file_url.as_ref().unwrap().as_bytes().to_vec(),
                    }],
                    declared_types: vec!["text/uri-list".into()],
                    plain_text: None,
                }],
            }
        } else if i == 1 {
            let picture = image::RgbaImage::from_fn(240, 160, |x, y| {
                image::Rgba([(x % 256) as u8, (y % 256) as u8, 130, 255])
            });
            let mut bytes = Cursor::new(Vec::new());
            image::DynamicImage::ImageRgba8(picture)
                .write_to(&mut bytes, image::ImageFormat::Png)?;
            Payload {
                items: vec![Item {
                    representations: vec![Representation {
                        format: "image/png".into(),
                        data: bytes.into_inner(),
                    }],
                    declared_types: vec!["image/png".into()],
                    plain_text: None,
                }],
            }
        } else {
            Payload::text(&text)
        };
        store.ingest(Snapshot {
            capture_id: format!("generated-{i}"),
            captured_at: now - i * 120_000,
            payload,
            source: Source {
                app_id: Some("c:\\generated\\notepad.exe".into()),
                name: "Generated Notepad".into(),
                basis: "owner".into(),
            },
            policy_revision: active.revision,
            sensitive: false,
        })?;
    }
    let mut paused = store.settings()?;
    paused.capture_enabled = false;
    store.update_settings(paused)?;
    println!("generated-records=120 capture=paused");
    Ok(())
}
