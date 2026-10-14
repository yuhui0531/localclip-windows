use localclip_app::output::compose;
use localclip_core::domain::Payload;
#[test]
fn output_preserves_selection_order_and_never_silently_drops_nontext() {
    let records = vec![Payload::text("b\r\n"), Payload::text("a")];
    let result = compose(records, true, "jsonArray", None, None, 0).unwrap();
    assert_eq!(result.plain_text().unwrap(), "[\"b\", \"a\"]");
    assert!(compose(vec![], true, "newline", None, None, 0).is_err());
}

#[test]
fn original_merge_rejects_unrepresentable_image_and_text_without_dropping_images() {
    use localclip_core::domain::{Item, Representation};
    let image = Payload {
        items: vec![Item {
            representations: vec![Representation {
                format: "image/png".into(),
                data: vec![1, 2, 3],
            }],
            declared_types: vec!["image/png".into()],
            plain_text: None,
        }],
    };
    assert!(
        compose(
            vec![Payload::text("text"), image],
            false,
            "newline",
            None,
            None,
            0
        )
        .is_err()
    );
}

#[cfg(windows)]
#[test]
fn rich_merge_preserves_two_formatted_fragments() {
    use localclip_core::domain::Representation;
    let mut first = Payload::text("first");
    first.items[0].representations.push(Representation {
        format: "text/rtf".into(),
        data: br"{\rtf1\ansi\b first\b0}".to_vec(),
    });
    let mut second = Payload::text("second");
    second.items[0].representations.push(Representation {
        format: "text/rtf".into(),
        data: br"{\rtf1\ansi\i second\i0}".to_vec(),
    });
    let merged = compose(vec![first, second], false, "newline", None, None, 0).unwrap();
    assert_eq!(merged.plain_text().as_deref(), Some("first\nsecond"));
    let rtf = String::from_utf8_lossy(merged.items[0].representation("text/rtf").unwrap());
    assert!(rtf.contains("first") && rtf.contains("second"));
    assert!(rtf.contains(r"\b") && rtf.contains(r"\i"));
}
