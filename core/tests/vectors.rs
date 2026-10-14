use base64::Engine;
use localclip_core::{domain, inspect, query, search, transform, version};
use serde_json::Value;
use std::path::Path;

fn vectors(name: &str) -> Vec<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../test-vectors")
        .join(format!("{name}.json"));
    let document: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!(
        document["metadata"]["macosCommit"],
        "c097591431c00982ad34186068064b40965c95fd"
    );
    document["cases"].as_array().unwrap().clone()
}

fn string<'a>(case: &'a Value, key: &str) -> &'a str {
    case[key].as_str().unwrap()
}

fn payload(case: &Value) -> domain::Payload {
    domain::Payload {
        items: case["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| domain::Item {
                representations: item
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|rep| domain::Representation {
                        format: string(rep, "type").into(),
                        data: base64::engine::general_purpose::STANDARD
                            .decode(string(rep, "dataBase64"))
                            .unwrap(),
                    })
                    .collect(),
                declared_types: vec![],
                plain_text: None,
            })
            .collect(),
    }
}

#[test]
fn fingerprint_vectors() {
    for (i, case) in vectors("fingerprint").iter().enumerate() {
        assert_eq!(
            payload(case).fingerprint(),
            string(case, "fingerprint"),
            "fingerprint case {i}"
        );
    }
}

#[test]
fn kind_vectors() {
    for (i, case) in vectors("kind").iter().enumerate() {
        assert_eq!(
            payload(case).kind().as_str(),
            string(case, "kind"),
            "kind case {i}"
        );
    }
}

#[test]
fn imagefile_vectors() {
    for (i, case) in vectors("imagefile").iter().enumerate() {
        assert_eq!(
            domain::is_image_file(string(case, "path")),
            case["isImage"].as_bool().unwrap(),
            "imagefile case {i}"
        );
    }
}

#[test]
fn weblink_vectors() {
    for (i, case) in vectors("weblink").iter().enumerate() {
        assert_eq!(
            domain::web_link(string(case, "text")).as_deref(),
            case["url"].as_str(),
            "weblink case {i}"
        );
    }
}

#[test]
fn policy_vectors() {
    for (i, case) in vectors("policy").iter().enumerate() {
        let settings = domain::Settings {
            capture_enabled: case["settings"]["captureEnabled"].as_bool().unwrap(),
            excluded_app_ids: case["settings"]["excludedAppIDs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().into())
                .collect(),
            ..Default::default()
        };
        let source = domain::Source {
            app_id: case["source"]["sourceAppID"].as_str().map(String::from),
            name: string(&case["source"], "name").into(),
            basis: string(&case["source"], "basis").into(),
        };
        // Reference markers are translated to a privacy decision at the adapter boundary.
        let sensitive = case["types"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|v| v.as_array().unwrap())
            .any(|v| {
                matches!(
                    v.as_str(),
                    Some("org.nspasteboard.ConcealedType" | "org.nspasteboard.TransientType")
                )
            });
        assert_eq!(
            settings.rejection(&source, sensitive, 1760486400000),
            case["skipReason"].as_str(),
            "policy case {i}"
        );
    }
}

#[test]
fn html_vectors() {
    for (i, case) in vectors("html").iter().enumerate() {
        assert_eq!(
            search::html_text(string(case, "html")),
            string(case, "text"),
            "html case {i}"
        );
    }
}

#[test]
fn query_vectors() {
    for (i, case) in vectors("query").iter().enumerate() {
        let sources = case["sources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| domain::Source {
                app_id: v["sourceAppID"].as_str().map(String::from),
                name: string(v, "name").into(),
                basis: "owner".into(),
            })
            .collect::<Vec<_>>();
        let categories = case["categories"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| query::Category {
                id: string(v, "id").into(),
                name: string(v, "name").into(),
            })
            .collect::<Vec<_>>();
        let result = query::parse(
            string(case, "text"),
            &sources,
            &categories,
            case["nowMs"].as_i64().unwrap(),
            case["localOffsetSeconds"].as_i64().unwrap() as i32,
        );
        let expected = &case["expected"];
        assert_eq!(
            result.literal,
            string(expected, "literal"),
            "query literal {i}"
        );
        assert_eq!(
            result.kind.map(|k| k.as_str()),
            expected["kind"].as_str(),
            "query kind {i}"
        );
        assert_eq!(
            result.favorites,
            expected["favorites"].as_bool().unwrap(),
            "query favorites {i}"
        );
        assert_eq!(
            result.source_app_id.as_deref(),
            expected["sourceAppID"].as_str(),
            "query source {i}"
        );
        assert_eq!(
            serde_json::json!(result.category_ids),
            expected["categoryIDs"],
            "query categories {i}"
        );
        assert_eq!(result.from, expected["from"].as_i64(), "query from {i}");
        assert_eq!(result.until, expected["until"].as_i64(), "query until {i}");
        assert_eq!(
            serde_json::json!(result.labels),
            expected["labels"],
            "query labels {i}"
        );
        assert_eq!(
            serde_json::json!(result.problems),
            expected["problems"],
            "query problems {i}"
        );
    }
}

#[test]
fn transform_vectors() {
    for (i, case) in vectors("transform").iter().enumerate() {
        let operation = transform::Transform::parse(string(case, "transform")).unwrap();
        let input = string(case, "input");
        assert_eq!(
            operation.applicable(input, 28800),
            case["applicable"].as_bool().unwrap(),
            "transform applicable {i}"
        );
        let result = operation.apply(input, None, 28800);
        match result {
            Ok(text) => assert_eq!(
                Some(text.as_str()),
                case["output"].as_str(),
                "transform output {i}"
            ),
            Err(error) => assert_eq!(
                Some(error.as_str()),
                case["error"].as_str(),
                "transform error {i}"
            ),
        }
    }
}

#[test]
fn join_vectors() {
    for (i, case) in vectors("join").iter().enumerate() {
        let style = transform::Join::parse(string(case, "style")).unwrap();
        let values = case["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(style.join(&values), string(case, "output"), "join case {i}");
    }
}

#[test]
fn inspect_vectors() {
    let overrides: Value = serde_json::from_str(include_str!("windows-overrides.json")).unwrap();
    for (i, case) in vectors("inspect").iter().enumerate() {
        let text = string(case, "text");
        match string(case, "operation") {
            "inspect" => {
                let now = case["nowMs"].as_i64().unwrap();
                assert_eq!(
                    inspect::badge(text, case["truncated"].as_bool().unwrap(), now, 28800)
                        .as_deref(),
                    case["badge"].as_str(),
                    "inspect badge {i}"
                );
                let result = inspect::full(text, case["language"].as_str(), now, 28800)
                    .unwrap_or(Value::Null);
                for field in ["kind", "rows", "note", "actions"] {
                    assert_eq!(result[field], case[field], "inspect {i} {field}");
                }
            }
            "exportFormat" => {
                let types = case["types"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap())
                    .collect::<Vec<_>>();
                let result = inspect::export_format(
                    text,
                    case["language"].as_str(),
                    &types,
                    case["markup"].as_str(),
                );
                assert_eq!(
                    result.extension,
                    string(case, "exportFormat"),
                    "export extension {i}"
                );
                assert_eq!(
                    result.original_type,
                    case["originalType"].as_str(),
                    "export original {i}"
                );
                assert_eq!(
                    if result.original_type.is_some() {
                        "original"
                    } else {
                        "plain"
                    },
                    string(case, "mode"),
                    "export mode {i}"
                );
            }
            "fileName" | "exportBaseName" => {
                let change = &overrides["inspect"][i.to_string()];
                let expected = if change.is_null() { case } else { change };
                assert_eq!(
                    inspect::sanitized_file_name(text).as_deref(),
                    expected["sanitizedFileName"].as_str(),
                    "filename {i}"
                );
                assert_eq!(
                    inspect::export_base_name(
                        Some(text),
                        case["copiedAt"].as_i64().unwrap(),
                        28800
                    )
                    .unwrap(),
                    string(expected, "exportBaseName"),
                    "export base {i}"
                );
            }
            _ => panic!("unknown inspect operation"),
        }
    }
}

#[test]
fn normalize_vectors() {
    for (index, case) in vectors("normalize").iter().enumerate() {
        assert_eq!(
            search::normalize(string(case, "input")),
            string(case, "normalized"),
            "normalize case {index}"
        );
    }
}

#[test]
fn context_vectors() {
    for (index, case) in vectors("context").iter().enumerate() {
        assert_eq!(
            search::context(string(case, "original"), string(case, "literal")),
            string(case, "context"),
            "context case {index}"
        );
    }
}

#[test]
fn version_vectors() {
    for (index, case) in vectors("version").iter().enumerate() {
        assert_eq!(
            version::is_newer(string(case, "candidate"), string(case, "current")),
            case["newer"].as_bool().unwrap(),
            "version case {index}"
        );
    }
}

#[test]
fn literal_rank_is_byte_literal() {
    assert_eq!(search::literal_rank(b"x\0y", b"\0", ""), 2);
    assert_eq!(search::literal_rank(b"%_\"*", b"%_\"*", ""), 0);
    assert_eq!(search::literal_rank(b"first\nsecond", b"first\n", ""), 1);
    assert_eq!(search::literal_rank(b"body", b"source", "SOURCE"), 3);
    assert_eq!(search::literal_rank(b"body", b"", ""), 4);
}

struct StoreFixture {
    root: std::path::PathBuf,
    store: Option<localclip_core::store::HistoryStore>,
    now: std::sync::Arc<std::sync::atomic::AtomicI64>,
    fault: std::sync::Arc<std::sync::atomic::AtomicBool>,
    sequence: usize,
}
impl StoreFixture {
    fn new(settings: domain::Settings, now: i64, name: &str) -> Self {
        use localclip_core::store::*;
        use std::sync::{
            Arc,
            atomic::{AtomicBool, AtomicI64, Ordering},
        };
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../target/vector-stores")
            .join(format!("{}-{name}", std::process::id()));
        assert!(!root.exists());
        std::fs::create_dir_all(&root).unwrap();
        let now = Arc::new(AtomicI64::new(now));
        let clock = now.clone();
        let fault = Arc::new(AtomicBool::new(false));
        let hook = fault.clone();
        let store = HistoryStore::open(
            root.join("history.sqlite"),
            Options {
                settings,
                now: Arc::new(move || clock.load(Ordering::SeqCst)),
                available_space: Arc::new(|| Ok(u64::MAX)),
                before_commit: Arc::new(move || {
                    if hook.load(Ordering::SeqCst) {
                        Err(StoreError::IoFailure)
                    } else {
                        Ok(())
                    }
                }),
            },
        )
        .unwrap();
        Self {
            root,
            store: Some(store),
            now,
            fault,
            sequence: 0,
        }
    }
    fn store(&self) -> &localclip_core::store::HistoryStore {
        self.store.as_ref().unwrap()
    }
    fn ingest(
        &mut self,
        payload: domain::Payload,
        source: domain::Source,
    ) -> localclip_core::store::Result<localclip_core::store::Ingest> {
        self.sequence += 1;
        self.store().ingest(localclip_core::store::Snapshot {
            capture_id: format!("operation-{}", self.sequence),
            captured_at: self.now.load(std::sync::atomic::Ordering::SeqCst),
            payload,
            source,
            policy_revision: self.store().settings().unwrap().revision,
            sensitive: false,
        })
    }
}
impl Drop for StoreFixture {
    fn drop(&mut self) {
        drop(self.store.take());
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn rank_vectors() {
    use localclip_core::store::*;
    for (i, case) in vectors("rank").iter().enumerate() {
        let mut f = StoreFixture::new(
            domain::Settings::default(),
            1760486400000,
            &format!("rank-{i}"),
        );
        let mut ids = Vec::new();
        for entry in case["corpus"].as_array().unwrap() {
            f.now.store(
                entry["copiedAt"].as_i64().unwrap(),
                std::sync::atomic::Ordering::SeqCst,
            );
            let payload = domain::Payload {
                items: entry["items"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| domain::Payload::text(v.as_str().unwrap()).items.remove(0))
                    .collect(),
            };
            let result = f
                .ingest(
                    payload,
                    domain::Source {
                        name: string(entry, "source").into(),
                        ..Default::default()
                    },
                )
                .unwrap();
            let Ingest::Inserted(id) = result else {
                panic!("expected insert");
            };
            if let Some(title) = entry["title"].as_str() {
                f.store().rename(id.clone(), Some(title.into())).unwrap();
            }
            ids.push(id);
        }
        let actual = f
            .store()
            .search(SearchRequest {
                literal: string(case, "needle").into(),
                ..Default::default()
            })
            .unwrap();
        let positions = actual
            .items
            .iter()
            .map(|item| ids.iter().position(|id| *id == item.id).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            serde_json::json!(positions),
            case["expectedOrder"],
            "rank vector {i}"
        );
    }
}

#[test]
fn store_scenario_vectors() {
    use localclip_core::store::*;
    use std::{collections::BTreeMap, sync::atomic::Ordering};
    for case in vectors("store-scenarios") {
        let name = string(&case, "name");
        let initial = &case["initialSettings"];
        let mut f = StoreFixture::new(
            domain::Settings {
                quota_bytes: initial["quotaBytes"].as_u64().unwrap(),
                retention_days: initial["retentionDays"].as_u64().unwrap() as u16,
                ..Default::default()
            },
            case["initialNowMs"].as_i64().unwrap(),
            name,
        );
        let mut ids = BTreeMap::<String, String>::new();
        let mut categories = BTreeMap::<String, String>::new();
        for (index, step) in case["steps"].as_array().unwrap().iter().enumerate() {
            f.now
                .store(step["nowMs"].as_i64().unwrap(), Ordering::SeqCst);
            let op = &step["operation"];
            let expected = &step["expected"];
            let kind = string(op, "kind");
            let mut result = None;
            let mut error = None;
            let mut removed = None;
            let mapped_ids = || {
                op["ids"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| ids[v.as_str().unwrap()].clone())
                    .collect::<Vec<_>>()
            };
            let change = match kind {
                "favorite" => Some(Change::Favorite(
                    vec![ids[string(op, "id")].clone()],
                    op["value"].as_bool().unwrap(),
                )),
                "assign" => Some(Change::Category(
                    mapped_ids(),
                    categories[string(op, "categoryID")].clone(),
                    op["included"].as_bool().unwrap(),
                )),
                "delete" => Some(Change::Delete(mapped_ids())),
                "setRetention" => Some(Change::Retention(op["days"].as_u64().unwrap() as u16)),
                "setQuota" => Some(Change::Quota(op["bytes"].as_u64().unwrap())),
                _ => None,
            };
            if let Some(change) = change {
                let p = f.store().preview(change).unwrap();
                assert_eq!(
                    p.affected_count as u64,
                    expected["preview"]["affectedCount"].as_u64().unwrap(),
                    "{name} step {index} preview count"
                );
                assert_eq!(
                    p.requires_confirmation,
                    expected["preview"]["needsConfirmation"].as_bool().unwrap(),
                    "{name} step {index} preview confirmation"
                );
                if op["confirm"].as_bool() == Some(true) {
                    removed = Some(f.store().apply(p.token).unwrap().removed as u64);
                }
            } else {
                match kind {
                    "ingest" => {
                        let payload = if let Some(text) = op["text"].as_str() {
                            domain::Payload::text(text)
                        } else {
                            let g = &op["generatedPayload"];
                            domain::Payload {
                                items: vec![domain::Item {
                                    representations: vec![domain::Representation {
                                        format: string(g, "format").into(),
                                        data: vec![
                                            g["byte"].as_u64().unwrap() as u8;
                                            g["byteCount"].as_u64().unwrap() as usize
                                        ],
                                    }],
                                    ..Default::default()
                                }],
                            }
                        };
                        match f.ingest(payload, domain::Source::default()) {
                            Ok(Ingest::Inserted(id)) => {
                                ids.insert(string(op, "id").into(), id);
                                result = Some("inserted");
                            }
                            Ok(Ingest::Deduplicated(id)) => {
                                assert_eq!(ids[string(op, "id")], id);
                                result = Some("deduplicated");
                            }
                            Ok(Ingest::Skipped("quotaFull")) => error = Some("quotaExceeded"),
                            Err(StoreError::IoFailure) => error = Some("ioFailure"),
                            _ => panic!("unexpected ingest outcome"),
                        }
                    }
                    "advanceClock" => {}
                    "createCategory" => {
                        let id = f
                            .store()
                            .create_category(
                                string(op, "name").into(),
                                string(op, "colorHex").into(),
                            )
                            .unwrap();
                        categories.insert(string(op, "id").into(), id);
                    }
                    "touch" => f.store().touch(mapped_ids()).unwrap(),
                    "expire" => removed = Some(f.store().expire().unwrap().removed as u64),
                    "undo" => match f.store().undo_delete() {
                        Ok(()) => {}
                        Err(StoreError::NotFound) => error = Some("notFound"),
                        Err(e) => panic!("unexpected undo error {e}"),
                    },
                    "fault" => f
                        .fault
                        .store(op["enabled"].as_bool().unwrap(), Ordering::SeqCst),
                    _ => panic!("unknown operation"),
                }
            }
            assert_eq!(
                result,
                expected["result"].as_str(),
                "{name} step {index} result"
            );
            assert_eq!(
                error,
                expected["error"].as_str(),
                "{name} step {index} error"
            );
            assert_eq!(
                removed,
                expected["removed"].as_u64(),
                "{name} step {index} removed"
            );
            let settings = f.store().settings().unwrap();
            assert_eq!(
                settings.quota_bytes,
                expected["quotaBytes"].as_u64().unwrap()
            );
            assert_eq!(
                settings.retention_days as u64,
                expected["retentionDays"].as_u64().unwrap()
            );
            assert_eq!(
                settings.revision,
                expected["settingsRevision"].as_i64().unwrap()
            );
            assert_eq!(
                f.store().undo_available().unwrap(),
                expected["undoAvailable"].as_bool().unwrap(),
                "{name} step {index} undo"
            );
            assert_eq!(
                f.store().can_capture().unwrap(),
                expected["canCapture"].as_bool().unwrap(),
                "{name} step {index} capacity"
            );
            let page = f.store().search(SearchRequest::default()).unwrap();
            let reverse = |id: &str| {
                ids.iter()
                    .find(|(_, v)| v.as_str() == id)
                    .unwrap()
                    .0
                    .clone()
            };
            let mut items=page.items.iter().map(|r|serde_json::json!({"id":reverse(&r.id),"copyCount":r.copy_count,"favorite":r.favorite,"protected":r.protected,"firstCopiedAt":r.first_copied_at,"lastCopiedAt":r.last_copied_at})).collect::<Vec<_>>();
            items.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
            assert_eq!(
                serde_json::json!(items),
                expected["items"],
                "{name} step {index} items"
            );
            let mut order = Vec::new();
            for group in expected["orderGroups"].as_array().unwrap() {
                let mut group = group
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| ids[v.as_str().unwrap()].clone())
                    .collect::<Vec<_>>();
                group.sort();
                order.extend(group);
            }
            assert_eq!(
                page.items.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
                order,
                "{name} step {index} ordering"
            );
            let mut protected = page
                .items
                .iter()
                .filter(|r| r.protected)
                .map(|r| reverse(&r.id))
                .collect::<Vec<_>>();
            protected.sort();
            assert_eq!(serde_json::json!(protected), expected["protectedIDs"]);
            for (cat, expected_order) in expected["categoryOrder"].as_object().unwrap() {
                let actual = f
                    .store()
                    .search(SearchRequest {
                        categories: [categories[cat].clone()].into(),
                        ..Default::default()
                    })
                    .unwrap();
                assert_eq!(
                    serde_json::json!(
                        actual
                            .items
                            .iter()
                            .map(|r| reverse(&r.id))
                            .collect::<Vec<_>>()
                    ),
                    *expected_order,
                    "{name} step {index} category order"
                );
            }
        }
        assert_eq!(f.store().integrity_check().unwrap(), "ok");
    }
}
