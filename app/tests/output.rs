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
