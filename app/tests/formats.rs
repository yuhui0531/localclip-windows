use localclip_app::win::formats::*;

#[test]
fn text_round_trip_keeps_crlf_and_rejects_invalid_utf16() {
    let text = "中文 🙂\r\nline\n";
    assert_eq!(decode_utf16(&encode_utf16(text)), Some(text.into()));
    assert_eq!(decode_utf16(&[0, 0xd8, 0, 0]), None);
    assert_eq!(decode_utf16(&[1]), None);
    assert_eq!(decode_utf16(&[65, 0, 0, 0, 66, 0]), Some("A".into()));
}

#[test]
fn html_offsets_count_utf8_bytes_and_keep_fragment() {
    let fragment = "<b>中文 🙂</b>\r\nnext";
    let bytes = encode_html(fragment.as_bytes()).unwrap();
    assert_eq!(decode_html(&bytes), Some(fragment.as_bytes().to_vec()));
    let header =
        "Version:1.0\rStartHTML:-1\rEndHTML:-1\rStartFragment:0000000089\rEndFragment:0000000093\r";
    let header = header
        .replace("0000000089", &format!("{:010}", header.len()))
        .replace("0000000093", &format!("{:010}", header.len() + 4));
    assert_eq!(
        decode_html(format!("{header}test").as_bytes()),
        Some(b"test".to_vec())
    );
    let bad = b"Version:1.0\r\nStartHTML:0000000000\r\nEndHTML:9999999999\r\nStartFragment:0000000000\r\nEndFragment:9999999999\r\n";
    assert!(decode_html(bad).is_none());
    assert!(decode_html(b"<b>no header</b>").is_none());
}

#[test]
fn file_urls_and_dropfiles_preserve_order_and_paths() {
    let mut ansi = vec![0; 20];
    ansi[..4].copy_from_slice(&20u32.to_le_bytes());
    ansi.extend_from_slice(b"C:\\one.txt\0C:\\two.txt\0\0");
    assert_eq!(
        decode_dropfiles_with(&ansi, |v| std::str::from_utf8(v).ok().map(String::from)),
        Some(vec![r"C:\one.txt".into(), r"C:\two.txt".into()])
    );
    let paths: Vec<String> = vec![
        r"C:\测试 文件\one.txt".into(),
        r"\\server\share\a#b%.png".into(),
    ];
    for path in &paths {
        let url = path_to_url(path).unwrap();
        assert_eq!(url_to_path(&url).as_ref(), Some(path));
    }
    let block = encode_dropfiles(&paths).unwrap();
    assert_eq!(decode_dropfiles(&block).unwrap(), paths);
    assert!(path_to_url("relative.txt").is_none());
    assert!(url_to_path("https://example.com/a").is_none());
    assert!(url_to_path("file:///C:/a%00b").is_none());
    assert!(encode_dropfiles(&vec![r"C:\a".into(); 33]).is_none());
    assert!(decode_dropfiles(&block[..block.len() - 2]).is_none());
}
