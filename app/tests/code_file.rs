use localclip_app::code_file::{decode, supported};
use localclip_core::domain::Preferences;

#[test]
fn upgrade_defaults_off_and_explicit_preference_round_trips() {
    let mut old = Preferences::default().document();
    old.as_object_mut().unwrap().remove("smartCodeFilePaste");
    assert!(
        !Preferences::from_document(&old)
            .unwrap()
            .smart_code_file_paste
    );
    old["smartCodeFilePaste"] = true.into();
    assert!(
        Preferences::from_document(&old)
            .unwrap()
            .smart_code_file_paste
    );
    old["smartCodeFilePaste"] = "true".into();
    assert!(Preferences::from_document(&old).is_none());
}
#[test]
fn source_types_and_strict_encodings_preserve_content() {
    for name in [
        "a.Swift",
        "index.tsx",
        "data.jsonc",
        "Dockerfile.dev",
        "Makefile",
        "main.rs",
        "main.ps1",
    ] {
        assert!(supported(name));
    }
    for name in ["data.txt", "a.exe", "a.png", "a.py.exe", "README"] {
        assert!(!supported(name));
    }
    let text = "  let 名称 = \"🦀\";\r\n\t\n";
    assert_eq!(decode(text.as_bytes()).unwrap(), text);
    assert_eq!(
        decode(&[b"\xef\xbb\xbf".as_slice(), text.as_bytes()].concat()).unwrap(),
        text
    );
    for little in [true, false] {
        let mut bytes = if little {
            vec![0xff, 0xfe]
        } else {
            vec![0xfe, 0xff]
        };
        for word in text.encode_utf16() {
            bytes.extend(if little {
                word.to_le_bytes()
            } else {
                word.to_be_bytes()
            });
        }
        assert_eq!(decode(&bytes).unwrap(), text);
        let mut bytes = if little {
            vec![0xff, 0xfe, 0, 0]
        } else {
            vec![0, 0, 0xfe, 0xff]
        };
        for c in text.chars() {
            bytes.extend(if little {
                (c as u32).to_le_bytes()
            } else {
                (c as u32).to_be_bytes()
            });
        }
        assert_eq!(decode(&bytes).unwrap(), text);
    }
    for bytes in [
        b"a\0b".as_slice(),
        b"\xff",
        b"\xff\xfe\x00\xd8",
        b"\xff\xfe\x01",
        b"\xff\xfe\0\0\0\0\x11\0",
        b"\x7f",
        b"\x01",
    ] {
        assert!(decode(bytes).is_err());
    }
}

#[cfg(windows)]
#[test]
fn windows_reader_bounds_locks_and_all_or_nothing() {
    use localclip_app::{
        code_file::{PER_FILE_LIMIT, TOTAL_LIMIT},
        win::{code_file::Files, formats::path_to_url},
    };
    use localclip_core::domain::{Item, Payload, Representation};
    let root = std::env::temp_dir().join(format!("localclip-code-reader-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let payload = |names: &[&str]| Payload {
        items: names
            .iter()
            .map(|name| Item {
                representations: vec![Representation {
                    format: "text/uri-list".into(),
                    data: path_to_url(root.join(name).to_str().unwrap())
                        .unwrap()
                        .into_bytes(),
                }],
                declared_types: vec!["text/uri-list".into()],
                plain_text: None,
            })
            .collect(),
    };
    std::fs::write(root.join("one.rs"), b"// generated\r\n").unwrap();
    std::fs::write(root.join("two.json"), b"{\"generated\":true}").unwrap();
    let original = payload(&["one.rs", "two.json"]);
    let mut files = Files::open(&original).unwrap();
    assert!(std::fs::write(root.join("one.rs"), b"overwrite").is_err());
    assert!(std::fs::rename(root.join("one.rs"), root.join("moved.rs")).is_err());
    assert_eq!(
        files.read().unwrap(),
        "// generated\r\n\n{\"generated\":true}"
    );
    assert!(files.is_current());
    drop(files);
    std::fs::write(root.join("binary.py"), b"a\0b").unwrap();
    assert!(
        Files::open(&payload(&["one.rs", "binary.py"]))
            .unwrap()
            .read()
            .is_err()
    );
    assert!(Files::open(&payload(&["one.rs", "missing.py"])).is_err());
    assert!(Files::open(&payload(&["one.rs", "one.exe"])).is_err());
    std::fs::write(root.join("big.rs"), vec![b'a'; PER_FILE_LIMIT + 1]).unwrap();
    assert!(Files::open(&payload(&["big.rs"])).is_err());
    std::fs::write(root.join("big.rs"), vec![b'a'; PER_FILE_LIMIT]).unwrap();
    assert!(
        Files::open(&payload(&["big.rs", "big.rs"]))
            .unwrap()
            .read()
            .unwrap()
            .len()
            == TOTAL_LIMIT + 1
    );
    assert!(Files::open(&payload(&["big.rs", "big.rs", "one.rs"])).is_err());
    std::fs::remove_dir_all(root).unwrap();
}
