use localclip_core::{
    domain::{Item, Payload, Representation},
    export, inspect,
};

#[test]
fn windows_exports_preserve_bytes_and_choose_native_extensions() {
    let payload = Payload::text("https://example.com/a?b=1");
    let file = export::prepare(&payload, None, None).unwrap();
    assert_eq!(file.extension, "url");
    assert_eq!(
        file.bytes,
        b"[InternetShortcut]\r\nURL=https://example.com/a?b=1\r\n"
    );
    let mut payload = Payload::text("let x = 1\r\n");
    let file = export::prepare(&payload, Some("swift"), None).unwrap();
    assert_eq!(file.extension, "swift");
    assert_eq!(file.bytes, b"let x = 1\r\n");
    payload.items[0].representations.push(Representation {
        format: "text/rtf".into(),
        data: b"{\\rtf1 test}".to_vec(),
    });
    let file = export::prepare(&payload, Some("swift"), None).unwrap();
    assert_eq!(file.extension, "rtf");
    assert_eq!(file.bytes, b"{\\rtf1 test}");
    for (format, extension) in [("image/png", "png"), ("image/jpeg", "jpg")] {
        let bytes = vec![0, 1, 255, 0, 23];
        let image = Payload {
            items: vec![Item {
                representations: vec![Representation {
                    format: format.into(),
                    data: bytes.clone(),
                }],
                ..Default::default()
            }],
        };
        let file = export::prepare(&image, None, None).unwrap();
        assert_eq!(file.extension, extension);
        assert_eq!(file.bytes, bytes);
    }
    let mut dib = vec![0u8; 128];
    dib[..4].copy_from_slice(&124u32.to_le_bytes());
    dib[4..8].copy_from_slice(&1i32.to_le_bytes());
    dib[8..12].copy_from_slice(&1i32.to_le_bytes());
    dib[12..14].copy_from_slice(&1u16.to_le_bytes());
    dib[14..16].copy_from_slice(&32u16.to_le_bytes());
    dib[20..24].copy_from_slice(&4u32.to_le_bytes());
    let bmp = export::dibv5_to_bmp(&dib).unwrap();
    assert_eq!(&bmp[..2], b"BM");
    assert_eq!(&bmp[14..], dib);
    assert_eq!(u32::from_le_bytes(bmp[10..14].try_into().unwrap()), 138);
}

#[test]
fn filenames_cannot_overwrite_or_escape_the_destination() {
    let long_reserved = format!("CON.{}", "a".repeat(196));
    let safe = inspect::sanitized_file_name(&long_reserved).unwrap();
    assert!(safe.starts_with("_CON."));
    assert!(safe.len() <= 200);
    for (input, expected) in [
        ("CON.txt", "_CON.txt"),
        ("name... ", "name"),
        ("a\\b:c*?\"<>|", "a-b-c------"),
        ("LPT¹", "_LPT¹"),
    ] {
        assert_eq!(
            inspect::sanitized_file_name(input).as_deref(),
            Some(expected)
        );
    }
    assert_eq!(
        export::available_name("name.txt", |v| v == "name.txt" || v == "name 2.txt"),
        Some("name 3.txt".into())
    );
    assert_eq!(export::available_name("name.txt", |_| true), None);
    assert_eq!(export::available_name("../escape.txt", |_| false), None);
    assert!(inspect::file_name("safe", None, "../txt").is_none());
}
