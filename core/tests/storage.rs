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
