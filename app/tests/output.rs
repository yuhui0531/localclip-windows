use localclip_app::output::compose;
use localclip_core::domain::Payload;
#[test]
fn output_preserves_selection_order_and_never_silently_drops_nontext() {
    let records = vec![Payload::text("b\r\n"), Payload::text("a")];
    let result = compose(records, true, "jsonArray", None, None, 0).unwrap();
    assert_eq!(result.plain_text().unwrap(), "[\"b\", \"a\"]");
    assert!(compose(vec![], true, "newline", None, None, 0).is_err());
}
