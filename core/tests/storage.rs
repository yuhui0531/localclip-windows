use localclip_core::{
    domain::{Item, MIB, Payload, Representation, Settings, Source},
    store::*,
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicI64, AtomicU64, Ordering},
    },
};

static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Fixture {
    root: PathBuf,
    now: Arc<AtomicI64>,
    store: Option<HistoryStore>,
}
impl Fixture {
    fn new(settings: Settings) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../target/test-stores")
            .join(format!(
                "{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::SeqCst)
            ));
        std::fs::create_dir_all(&root).unwrap();
        let now = Arc::new(AtomicI64::new(1760486400000));
        let clock = now.clone();
        let options = Options {
            settings,
            now: Arc::new(move || clock.load(Ordering::SeqCst)),
            available_space: Arc::new(|| Ok(u64::MAX)),
            before_commit: Arc::new(|| Ok(())),
        };
        let store = HistoryStore::open(root.join("history.sqlite"), options).unwrap();
        Self {
            root,
            now,
            store: Some(store),
        }
    }
    fn store(&self) -> &HistoryStore {
        self.store.as_ref().unwrap()
    }
    fn ingest(&self, payload: Payload) -> Ingest {
        self.store()
            .ingest(Snapshot {
                capture_id: format!("capture-{}", COUNTER.fetch_add(1, Ordering::SeqCst)),
                captured_at: self.now.load(Ordering::SeqCst),
                payload,
                source: Source::default(),
                policy_revision: self.store().settings().unwrap().revision,
                sensitive: false,
            })
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.store.take());
        std::fs::remove_dir_all(&self.root).unwrap();
    }
}

#[test]
fn legacy_rich_text_remains_searchable_without_changing_saved_bytes() {
    let f = Fixture::new(Settings::default());
    let payload = Payload {
        items: vec![Item {
            plain_text: Some("formatted body".into()),
            representations: vec![
                Representation {
                    format: "text/html".into(),
                    data: b"<b>formatted body</b>".to_vec(),
                },
                Representation {
                    format: "text/rtf".into(),
                    data: br"{\rtf1 formatted body}".to_vec(),
                },
            ],
            ..Default::default()
        }],
    };
    let Ingest::Inserted(id) = f.ingest(payload.clone()) else {
        panic!("insert")
    };
    // 模拟旧版分类，验证新筛选不会丢失旧记录或改写原始表示。
    let db = rusqlite::Connection::open(f.root.join("history.sqlite")).unwrap();
    db.execute("UPDATE clips SET kind='richText' WHERE id=?", [&id])
        .unwrap();
    let found = f
        .store()
        .search(SearchRequest {
            kind: Some(localclip_core::domain::Kind::Text),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(found.items.len(), 1);
    assert_eq!(found.items[0].id, id);
    let restored = f.store().payload(&id).unwrap();
    for rep in &payload.items[0].representations {
        assert_eq!(
            restored.items[0].representation(&rep.format),
            Some(rep.data.as_slice())
        );
    }
    assert_eq!(
        db.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert_eq!(f.store().integrity_check().unwrap(), "ok");
}

#[test]
fn renaming_requires_visible_record_and_unchanged_name_preserves_cursor() {
    let f = Fixture::new(Settings {
        retention_days: 1,
        ..Settings::default()
    });
    let Ingest::Inserted(id) = f.ingest(Payload::text("one")) else {
        panic!("insert")
    };
    f.ingest(Payload::text("two"));
    let page = f
        .store()
        .search(SearchRequest {
            page_size: 1,
            ..Default::default()
        })
        .unwrap();
    f.store().rename(id.clone(), None).unwrap();
    assert!(
        f.store()
            .search(SearchRequest {
                page_size: 1,
                cursor: page.next,
                ..Default::default()
            })
            .is_ok()
    );
    f.now.fetch_add(86_400_000, Ordering::SeqCst);
    assert_eq!(
        f.store().rename(id, Some("expired".into())),
        Err(StoreError::NotFound)
    );
}

#[test]
fn original_representations_round_trip_and_deduplicate() {
    let f = Fixture::new(Settings::default());
    let payload = Payload {
        items: vec![Item {
            representations: ["image/png", "image/jpeg", "image/x-win-dibv5"]
                .iter()
                .enumerate()
                .map(|(i, t)| Representation {
                    format: (*t).into(),
                    data: vec![i as u8, 0, 255, 1, 2, 3],
                })
                .collect(),
            ..Default::default()
        }],
    };
    let Ingest::Inserted(id) = f.ingest(payload.clone()) else {
        panic!("insert");
    };
    let restored = f.store().payload(&id).unwrap();
    assert_eq!(restored.items.len(), payload.items.len());
    assert_eq!(
        restored.items[0].representations.len(),
        payload.items[0].representations.len()
    );
    for rep in &payload.items[0].representations {
        assert_eq!(
            restored.items[0].representation(&rep.format),
            Some(rep.data.as_slice())
        );
    }
    let bytes = f.store().usage().unwrap().database;
    assert_eq!(f.ingest(payload), Ingest::Deduplicated(id.clone()));
    assert_eq!(
        f.store().search(SearchRequest::default()).unwrap().items[0].copy_count,
        2
    );
    assert_eq!(f.store().usage().unwrap().database, bytes);
    assert_eq!(f.store().integrity_check().unwrap(), "ok");
}

#[test]
fn stale_preview_cursor_and_policy_cannot_commit() {
    let f = Fixture::new(Settings::default());
    let Ingest::Inserted(id) = f.ingest(Payload::text("one")) else {
        panic!("insert");
    };
    f.ingest(Payload::text("two"));
    let page = f
        .store()
        .search(SearchRequest {
            page_size: 1,
            ..Default::default()
        })
        .unwrap();
    let preview = f.store().preview(Change::Delete(vec![id])).unwrap();
    f.ingest(Payload::text("three"));
    assert_eq!(
        f.store().apply(preview.token),
        Err(StoreError::StalePreview)
    );
    assert!(matches!(
        f.store().search(SearchRequest {
            page_size: 1,
            cursor: page.next,
            ..Default::default()
        }),
        Err(StoreError::InvalidCursor)
    ));
    let mut settings = f.store().settings().unwrap();
    settings.capture_enabled = false;
    f.store().update_settings(settings).unwrap();
    let result = f
        .store()
        .ingest(Snapshot {
            capture_id: "old-policy".into(),
            captured_at: f.now.load(Ordering::SeqCst),
            payload: Payload::text("rejected-private-body"),
            source: Source::default(),
            policy_revision: 0,
            sensitive: false,
        })
        .unwrap();
    assert_eq!(result, Ingest::Skipped("paused"));
    assert_eq!(
        f.store()
            .search(SearchRequest::default())
            .unwrap()
            .items
            .len(),
        3
    );
    for name in ["history.sqlite", "history.sqlite-wal"] {
        if let Ok(bytes) = std::fs::read(f.root.join(name)) {
            assert!(!bytes.windows(21).any(|w| w == b"rejected-private-body"));
        }
    }
}

#[test]
fn quota_rollback_preserves_ordinary_history() {
    let f = Fixture::new(Settings {
        quota_bytes: 512 * 1024,
        ..Default::default()
    });
    let image = |size, byte| Payload {
        items: vec![Item {
            representations: vec![Representation {
                format: "image/png".into(),
                data: vec![byte; size],
            }],
            ..Default::default()
        }],
    };
    let Ingest::Inserted(id) = f.ingest(image(300 * 1024, 1)) else {
        panic!("insert");
    };
    let p = f.store().preview(Change::Favorite(vec![id], true)).unwrap();
    f.store().apply(p.token).unwrap();
    let Ingest::Inserted(old) = f.ingest(image(64 * 1024, 2)) else {
        panic!("insert");
    };
    assert_eq!(f.ingest(image(220 * 1024, 3)), Ingest::Skipped("quotaFull"));
    assert!(f.store().payload(&old).is_ok());
    assert!(f.store().usage().unwrap().database <= 512 * 1024);
}

#[test]
fn schema_pragmas_unknown_version_and_disk_gate() {
    let f = Fixture::new(Settings::default());
    let db = rusqlite::Connection::open(f.root.join("history.sqlite")).unwrap();
    assert_eq!(
        db.query_row("PRAGMA journal_mode", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "wal"
    );
    assert_eq!(
        db.query_row("PRAGMA auto_vacuum", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    let other = f.root.join("future.sqlite");
    let future = rusqlite::Connection::open(&other).unwrap();
    future
        .execute_batch("PRAGMA user_version=2;CREATE TABLE sentinel(value TEXT);")
        .unwrap();
    drop(future);
    let before = std::fs::read(&other).unwrap();
    let options = Options {
        available_space: Arc::new(|| Ok(64 * MIB - 1)),
        ..Options::default()
    };
    assert!(matches!(
        HistoryStore::open(&other, options),
        Err(StoreError::IncompatibleSchema)
    ));
    assert_eq!(before, std::fs::read(&other).unwrap());
    let low = HistoryStore::open(
        f.root.join("low-space.sqlite"),
        Options {
            available_space: Arc::new(|| Ok(64 * MIB - 1)),
            ..Default::default()
        },
    )
    .unwrap();
    let result = low
        .ingest(Snapshot {
            capture_id: "low-space".into(),
            captured_at: 1760486400000,
            payload: Payload::text("synthetic-disk-gate-body"),
            source: Source::default(),
            policy_revision: 0,
            sensitive: false,
        })
        .unwrap();
    assert_eq!(result, Ingest::Skipped("diskFull"));
    assert!(
        low.search(SearchRequest::default())
            .unwrap()
            .items
            .is_empty()
    );
    drop(low);
}

#[test]
fn settings_categories_and_undo_preserve_metadata() {
    let f = Fixture::new(Settings::default());
    let mut s = f.store().settings().unwrap();
    assert_eq!(s.preferences.default_action, "directPaste");
    assert_eq!(s.preferences.default_format, "original");
    s.preferences.hide_contents = true;
    s.preferences.appearance = "dark".into();
    s.preferences.panel_height = 400;
    let next = f.store().update_settings(s.clone()).unwrap();
    assert_eq!(next.revision, 1);
    assert!(f.store().settings().unwrap().preferences == next.preferences);
    assert!(matches!(
        f.store().update_settings(s),
        Err(StoreError::StalePreview)
    ));
    let Ingest::Inserted(a) = f.ingest(Payload::text("a")) else {
        panic!("insert");
    };
    let Ingest::Inserted(b) = f.ingest(Payload::text("b")) else {
        panic!("insert");
    };
    let cat = f
        .store()
        .create_category("e\u{301}".into(), "ABCDEF".into())
        .unwrap();
    assert!(matches!(
        f.store().create_category("é".into(), "FFFFFF".into()),
        Err(StoreError::DuplicateName)
    ));
    let p = f
        .store()
        .preview(Change::Category(
            vec![a.clone(), b.clone()],
            cat.clone(),
            true,
        ))
        .unwrap();
    f.store().apply(p.token).unwrap();
    f.store().move_item(a.clone(), cat.clone(), 1).unwrap();
    f.store()
        .rename_category(cat.clone(), "Work".into(), "123456".into())
        .unwrap();
    f.store()
        .rename(a.clone(), Some("e\u{301}".into()))
        .unwrap();
    let p = f.store().preview(Change::Delete(vec![a.clone()])).unwrap();
    f.store().apply(p.token).unwrap();
    f.store().undo_delete().unwrap();
    let page = f
        .store()
        .search(SearchRequest {
            categories: [cat].into(),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        page.items.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec![b.as_str(), a.as_str()]
    );
    assert_eq!(page.items[1].title.as_deref(), Some("é"));
}

#[test]
fn wal_reader_can_search_during_an_uncommitted_write() {
    use std::sync::{Barrier, atomic::AtomicBool};
    let f = Fixture::new(Settings::default());
    let gate = Arc::new(Barrier::new(2));
    let resume = Arc::new(Barrier::new(2));
    let armed = Arc::new(AtomicBool::new(false));
    let hook = armed.clone();
    let entered = gate.clone();
    let release = resume.clone();
    let store = Arc::new(
        HistoryStore::open(
            f.root.join("concurrent.sqlite"),
            Options {
                now: Arc::new(|| 1760486400000),
                available_space: Arc::new(|| Ok(u64::MAX)),
                before_commit: Arc::new(move || {
                    if hook.load(Ordering::SeqCst) {
                        entered.wait();
                        release.wait();
                    }
                    Ok(())
                }),
                ..Default::default()
            },
        )
        .unwrap(),
    );
    let snapshot = |id: &str| Snapshot {
        capture_id: id.into(),
        captured_at: 1760486400000,
        payload: Payload::text(id),
        source: Source::default(),
        policy_revision: 0,
        sensitive: false,
    };
    store.ingest(snapshot("baseline")).unwrap();
    armed.store(true, Ordering::SeqCst);
    let writer = store.clone();
    let thread = std::thread::spawn(move || writer.ingest(snapshot("uncommitted")));
    gate.wait();
    let result = store.search(SearchRequest::default());
    resume.wait();
    assert_eq!(result.unwrap().items.len(), 1);
    assert!(matches!(
        thread.join().unwrap().unwrap(),
        Ingest::Inserted(_)
    ));
    assert_eq!(
        store.search(SearchRequest::default()).unwrap().items.len(),
        2
    );
}

#[test]
fn detail_metadata_uses_visibility_and_category_order() {
    let f = Fixture::new(Settings {
        retention_days: 1,
        ..Settings::default()
    });
    let Ingest::Inserted(id) = f.ingest(Payload::text("metadata test")) else {
        panic!("insert")
    };
    let category = f
        .store()
        .create_category("work".into(), "3388AA".into())
        .unwrap();
    let preview = f
        .store()
        .preview(Change::Category(vec![id.clone()], category.clone(), true))
        .unwrap();
    f.store().apply(preview.token).unwrap();
    assert_eq!(f.store().summary(&id).unwrap().id, id);
    assert_eq!(f.store().memberships(&id).unwrap(), vec![category]);
    assert_eq!(f.store().sources().unwrap().len(), 1);
    let Ingest::Inserted(expiring) = f.ingest(Payload::text("expires")) else {
        panic!("insert")
    };
    f.now.fetch_add(86_400_001, Ordering::SeqCst);
    assert!(matches!(
        f.store().summary(&expiring),
        Err(StoreError::NotFound)
    ));
    assert!(f.store().summary(&id).is_ok());
}

#[test]
fn image_and_file_filters_intersect_without_losing_image_files() {
    let f = Fixture::new(Settings::default());
    let file = |path: &str| Payload {
        items: vec![Item {
            representations: vec![Representation {
                format: "text/uri-list".into(),
                data: format!("file:///C:/{path}").into_bytes(),
            }],
            declared_types: vec!["text/uri-list".into()],
            plain_text: Some(format!("C:\\{path}")),
        }],
    };
    let Ingest::Inserted(image_file) = f.ingest(file("test.png")) else {
        panic!("insert")
    };
    f.ingest(file("test.txt"));
    let page = f
        .store()
        .search(SearchRequest {
            kind: Some(localclip_core::domain::Kind::Image),
            kind_intersection: Some(localclip_core::domain::Kind::File),
            page_size: 50,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(
        page.items.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
        vec![image_file.as_str()]
    );
}

#[test]
fn update_snapshot_is_consistent_and_never_overwrites_existing_backup() {
    let f = Fixture::new(Settings::default());
    f.ingest(Payload::text("generated snapshot payload"));
    let store = f.store.as_ref().unwrap();
    let destination = f.root.join("snapshot.sqlite");
    store.snapshot(destination.clone()).unwrap();
    let db = rusqlite::Connection::open(&destination).unwrap();
    assert_eq!(
        db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert_eq!(
        db.query_row("SELECT COUNT(*) FROM clips", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    drop(db);
    let before = std::fs::read(&destination).unwrap();
    assert!(store.snapshot(destination.clone()).is_err());
    assert_eq!(std::fs::read(destination).unwrap(), before);
    assert_eq!(store.integrity_check().unwrap(), "ok");
}

// 编辑必须更新正文索引与输出表示，同时保留用户组织信息。by AI.Coding
#[test]
fn edit_text_preserves_metadata_and_updates_search_and_deduplication() {
    let f = Fixture::new(Settings::default());
    let Ingest::Inserted(id) = f.ingest(Payload::text("old generated text")) else {
        panic!("insert")
    };
    f.store()
        .rename(id.clone(), Some("Keep title".into()))
        .unwrap();
    let p = f
        .store()
        .preview(Change::Favorite(vec![id.clone()], true))
        .unwrap();
    f.store().apply(p.token).unwrap();
    let category = f
        .store()
        .create_category("Keep category".into(), "00aa00".into())
        .unwrap();
    let p = f
        .store()
        .preview(Change::Category(vec![id.clone()], category.clone(), true))
        .unwrap();
    f.store().apply(p.token).unwrap();
    let before = f.store().summary(&id).unwrap();
    let draft = f.store().editable_text(&id).unwrap();
    let text = format!("{}\nconst answer = 42;", "generated ".repeat(100));
    f.store()
        .edit_text(id.clone(), draft.fingerprint, text.clone())
        .unwrap();
    let after = f.store().summary(&id).unwrap();
    assert_eq!(after.title, before.title);
    assert!(after.favorite && after.protected);
    assert_eq!(after.copy_count, before.copy_count);
    assert_eq!(after.last_copied_at, before.last_copied_at);
    assert_eq!(after.first_copied_at, before.first_copied_at);
    assert_eq!(f.store().memberships(&id).unwrap(), vec![category]);
    assert_eq!(
        f.store().payload(&id).unwrap().canonical(),
        Payload::text(&text).canonical()
    );
    assert!(
        f.store()
            .search(SearchRequest {
                literal: "old generated".into(),
                ..Default::default()
            })
            .unwrap()
            .items
            .is_empty()
    );
    assert_eq!(
        f.store()
            .search(SearchRequest {
                literal: "answer = 42".into(),
                ..Default::default()
            })
            .unwrap()
            .items[0]
            .id,
        id
    );
    assert_eq!(f.ingest(Payload::text(&text)), Ingest::Deduplicated(id));
}

// 冲突、重复和非法输入不能覆盖任一记录。by AI.Coding
#[test]
fn edit_text_rejects_stale_duplicate_invalid_and_hidden_content() {
    let f = Fixture::new(Settings::default());
    let Ingest::Inserted(id) = f.ingest(Payload::text("first")) else {
        panic!("insert")
    };
    f.ingest(Payload::text("second"));
    let draft = f.store().editable_text(&id).unwrap();
    for text in ["", "bad\0text"] {
        assert_eq!(
            f.store()
                .edit_text(id.clone(), draft.fingerprint.clone(), text.into()),
            Err(StoreError::InvalidText)
        );
    }
    assert_eq!(
        f.store()
            .edit_text(id.clone(), draft.fingerprint.clone(), "second".into()),
        Err(StoreError::DuplicateContent)
    );
    f.store()
        .edit_text(id.clone(), draft.fingerprint.clone(), "changed".into())
        .unwrap();
    assert_eq!(
        f.store()
            .edit_text(id.clone(), draft.fingerprint, "stale".into()),
        Err(StoreError::StaleEdit)
    );
    let draft = f.store().editable_text(&id).unwrap();
    let mut settings = f.store().settings().unwrap();
    settings.preferences.hide_contents = true;
    f.store().update_settings(settings).unwrap();
    assert!(matches!(
        f.store().editable_text(&id),
        Err(StoreError::ContentsHidden)
    ));
    assert_eq!(
        f.store()
            .edit_text(id.clone(), draft.fingerprint, "hidden".into()),
        Err(StoreError::ContentsHidden)
    );
    assert_eq!(
        f.store().payload(&id).unwrap().plain_text().unwrap(),
        "changed"
    );
}

// 未改正文不剥离格式；改动后不再输出过期 HTML/RTF。by AI.Coding
#[test]
fn edit_text_rich_formats_noop_and_url_reclassification() {
    let f = Fixture::new(Settings::default());
    let mut payload = Payload::text("original");
    payload.items[0].representations.push(Representation {
        format: "text/html".into(),
        data: b"<b>original</b>".to_vec(),
    });
    let Ingest::Inserted(id) = f.ingest(payload.clone()) else {
        panic!("insert")
    };
    let draft = f.store().editable_text(&id).unwrap();
    assert!(draft.rich_text);
    let revision = f.store().revision().unwrap();
    f.store()
        .edit_text(id.clone(), draft.fingerprint.clone(), draft.text)
        .unwrap();
    assert_eq!(f.store().revision().unwrap(), revision);
    assert_eq!(
        f.store().payload(&id).unwrap().canonical(),
        payload.canonical()
    );
    f.store()
        .edit_text(id.clone(), draft.fingerprint, "https://example.com".into())
        .unwrap();
    assert_eq!(
        f.store().summary(&id).unwrap().kind,
        localclip_core::domain::Kind::Url
    );
    assert_eq!(
        f.store().payload(&id).unwrap().items[0]
            .representations
            .len(),
        1
    );
    let draft = f.store().editable_text(&id).unwrap();
    f.store()
        .edit_text(id.clone(), draft.fingerprint, "ordinary text".into())
        .unwrap();
    assert_eq!(
        f.store().summary(&id).unwrap().kind,
        localclip_core::domain::Kind::Text
    );
}

// 不可编辑类型、删除及到期由存储层拒绝，不能靠 UI 绕过。by AI.Coding
#[test]
fn edit_text_rejects_non_text_deleted_and_expired() {
    let f = Fixture::new(Settings::default());
    for payload in [
        Payload {
            items: vec![Item {
                representations: vec![Representation {
                    format: "image/png".into(),
                    data: vec![1, 2, 3],
                }],
                ..Default::default()
            }],
        },
        Payload {
            items: vec![Item {
                representations: vec![Representation {
                    format: "text/uri-list".into(),
                    data: b"file:///C:/generated.txt".to_vec(),
                }],
                ..Default::default()
            }],
        },
        Payload {
            items: vec![
                Payload::text("one").items.remove(0),
                Payload::text("two").items.remove(0),
            ],
        },
    ] {
        let Ingest::Inserted(id) = f.ingest(payload.clone()) else {
            panic!("insert")
        };
        assert!(matches!(
            f.store().editable_text(&id),
            Err(StoreError::NotEditable)
        ));
        assert_eq!(
            f.store()
                .edit_text(id.clone(), payload.fingerprint(), "text".into()),
            Err(StoreError::NotEditable)
        );
        assert_eq!(
            f.store().payload(&id).unwrap().canonical(),
            payload.canonical()
        );
    }
    let Ingest::Inserted(id) = f.ingest(Payload::text("expiring")) else {
        panic!("insert")
    };
    let draft = f.store().editable_text(&id).unwrap();
    f.now
        .fetch_add(31 * localclip_core::domain::DAY_MS, Ordering::SeqCst);
    assert_eq!(
        f.store()
            .edit_text(id.clone(), draft.fingerprint.clone(), "new".into()),
        Err(StoreError::NotFound)
    );
    let p = f.store().preview(Change::Delete(vec![id.clone()])).unwrap();
    f.store().apply(p.token).unwrap();
    assert_eq!(
        f.store().edit_text(id, draft.fingerprint, "new".into()),
        Err(StoreError::NotFound)
    );
}

// 编辑配额不足不驱逐别的记录；提交失败回滚所有正文和索引。by AI.Coding
#[test]
fn edit_text_quota_and_commit_failure_roll_back() {
    let mut f = Fixture::new(Settings {
        quota_bytes: 512 * 1024,
        ..Default::default()
    });
    let Ingest::Inserted(id) = f.ingest(Payload::text("keep original")) else {
        panic!("insert")
    };
    let draft = f.store().editable_text(&id).unwrap();
    assert_eq!(
        f.store()
            .edit_text(id.clone(), draft.fingerprint.clone(), "x".repeat(400_000)),
        Err(StoreError::QuotaFull)
    );
    assert_eq!(
        f.store().payload(&id).unwrap().plain_text().unwrap(),
        "keep original"
    );
    let revision = f.store().revision().unwrap();
    drop(f.store.take());
    f.store = Some(
        HistoryStore::open(
            f.root.join("history.sqlite"),
            Options {
                available_space: Arc::new(|| Ok(u64::MAX)),
                before_commit: Arc::new(|| Err(StoreError::IoFailure)),
                now: Arc::new({
                    let clock = f.now.clone();
                    move || clock.load(Ordering::SeqCst)
                }),
                ..Default::default()
            },
        )
        .unwrap(),
    );
    assert_eq!(
        f.store()
            .edit_text(id.clone(), draft.fingerprint, "failed".into()),
        Err(StoreError::IoFailure)
    );
    assert_eq!(
        f.store().payload(&id).unwrap().plain_text().unwrap(),
        "keep original"
    );
    assert_eq!(f.store().revision().unwrap(), revision);
    assert_eq!(f.store().integrity_check().unwrap(), "ok");
}

// 精确组合字符计数覆盖重音、CRLF 与联字 emoji，且文件路径不冒充正文。by AI.Coding
#[test]
fn card_character_count_uses_full_composed_text() {
    assert_eq!(Payload::text("e\u{301}\r\n👨‍👩‍👧‍👦").character_count(), Some(3));
    let payload = Payload {
        items: vec![Item {
            representations: vec![Representation {
                format: "text/uri-list".into(),
                data: b"file:///C:/generated.txt".to_vec(),
            }],
            plain_text: Some("file:///C:/generated.txt".into()),
            ..Default::default()
        }],
    };
    assert_eq!(payload.character_count(), None);
    let f = Fixture::new(Settings::default());
    let Ingest::Inserted(id) = f.ingest(Payload::text(&"中文".repeat(300))) else {
        panic!("insert")
    };
    assert_eq!(f.store().summary(&id).unwrap().preview.chars().count(), 180);
    assert_eq!(f.store().payload(&id).unwrap().character_count(), Some(600));
}
