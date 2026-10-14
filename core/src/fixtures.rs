use crate::domain::{Item, Payload, Representation};
use image::{ExtendedColorType, ImageEncoder};

pub const NOW: i64 = 1760486400000;
pub const VERSION: &str = "localclip-fixtures-v2-windows-1";
pub fn text(index: usize) -> String {
    let stems = [
        "今天整理需求，明天处理代码。",
        "The local clipboard keeps original representations.",
        "https://example.invalid/path?a=1&b=2#section",
        "let foo_bar = 50%; a->b; \"value\"; path\\test\n",
    ];
    let base = format!("fixture-{index} {}\n", stems[index % 4]);
    let desired = if index < 9000 {
        256 + index % 1500
    } else if index < 9900 {
        2048 + index % 14000
    } else {
        16384 + index % 100000
    };
    format!("{} END-{index}", base.repeat((desired / base.len()).max(1)))
}
pub fn image(index: usize) -> Result<Payload, image::ImageError> {
    let (width, height) = (1920, 1080);
    let mut state = 0xC11Fu64 + index as u64;
    let mut pixels = vec![255u8; width * height * 4];
    for (pixel, rgba) in pixels.chunks_exact_mut(4).enumerate() {
        if pixel % 2 == 0 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
        }
        let gray = ((state >> 40) as u8) & 0x7f;
        rgba[..3].fill(gray);
    }
    let mut bytes = Vec::new();
    let format = match index % 100 {
        0..20 => {
            image::codecs::png::PngEncoder::new(&mut bytes).write_image(
                &pixels,
                width as u32,
                height as u32,
                ExtendedColorType::Rgba8,
            )?;
            "image/png"
        }
        20..30 => {
            // Windows has DIBV5 instead of TIFF. Generated fixture pixels are not captured originals.
            bytes.resize(124, 0);
            bytes[..4].copy_from_slice(&124u32.to_le_bytes());
            bytes[4..8].copy_from_slice(&(width as i32).to_le_bytes());
            bytes[8..12].copy_from_slice(&(-(height as i32)).to_le_bytes());
            bytes[12..14].copy_from_slice(&1u16.to_le_bytes());
            bytes[14..16].copy_from_slice(&32u16.to_le_bytes());
            bytes[16..20].copy_from_slice(&3u32.to_le_bytes());
            bytes[20..24].copy_from_slice(&(pixels.len() as u32).to_le_bytes());
            for (offset, mask) in [
                (40, 0x00ff0000u32),
                (44, 0x0000ff00),
                (48, 0x000000ff),
                (52, 0xff000000),
                (56, 0x73524742),
            ] {
                bytes[offset..offset + 4].copy_from_slice(&mask.to_le_bytes());
            }
            bytes.extend_from_slice(&pixels);
            "image/x-win-dibv5"
        }
        _ => {
            let rgb = pixels
                .chunks_exact(4)
                .flat_map(|p| p[..3].iter().copied())
                .collect::<Vec<_>>();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 72).encode(
                &rgb,
                width as u32,
                height as u32,
                ExtendedColorType::Rgb8,
            )?;
            "image/jpeg"
        }
    };
    Ok(Payload {
        items: vec![Item {
            declared_types: vec![format.into()],
            representations: vec![Representation {
                format: format.into(),
                data: bytes,
            }],
            plain_text: None,
        }],
    })
}
