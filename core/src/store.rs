//! 本地历史事务、索引与显式编辑。by AI.Coding
use crate::{
    domain::{self, DAY_MS, Item, Kind, MIB, Payload, Representation, Settings, Source},
    schema, search,
};
use rusqlite::{
    Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior, params,
    params_from_iter, types::Value,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

pub type Result<T> = std::result::Result<T, StoreError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    DiskFull,
    QuotaFull,
    IoFailure,
    CorruptPayload,
    IncompatibleSchema,
    StalePreview,
    NotFound,
    InvalidSettings,
    InvalidCursor,
    Unavailable,
    PolicyChanged,
    DuplicateName,
    DuplicateContent,
    StaleEdit,
    InvalidText,
    NotEditable,
    ContentsHidden,
}
impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for StoreError {}
impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::SqliteFailure(error, _)
                if error.code == rusqlite::ErrorCode::DiskFull =>
            {
                Self::DiskFull
            }
            rusqlite::Error::QueryReturnedNoRows => Self::NotFound,
            _ => Self::IoFailure,
        }
    }
}
pub struct Options {
    pub settings: Settings,
    pub now: Arc<dyn Fn() -> i64 + Send + Sync>,
    pub available_space: Arc<dyn Fn() -> Result<u64> + Send + Sync>,
    pub before_commit: Arc<dyn Fn() -> Result<()> + Send + Sync>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            now: Arc::new(|| {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as i64
            }),
            available_space: Arc::new(|| Err(StoreError::Unavailable)),
            before_commit: Arc::new(|| Ok(())),
        }
    }
}
/// 编辑快照仅返回完整文本和乐观锁，不写剪贴板。by AI.Coding
pub struct EditableText {
    pub text: String,
    pub fingerprint: String,
    pub rich_text: bool,
}
pub struct Snapshot {
    pub capture_id: String,
    pub captured_at: i64,
    pub payload: Payload,
    pub source: Source,
    pub policy_revision: i64,
    pub sensitive: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Ingest {
    Inserted(String),
    Deduplicated(String),
    Skipped(&'static str),
}
#[derive(Clone, Default)]
pub struct SearchRequest {
    pub literal: String,
    pub favorites: bool,
    pub kind: Option<Kind>,
    pub kind_intersection: Option<Kind>,
    pub categories: BTreeSet<String>,
    pub source_app_id: Option<String>,
    pub from: Option<i64>,
    pub until: Option<i64>,
    pub page_size: usize,
    pub cursor: Option<Cursor>,
}
impl SearchRequest {
    fn signature(&self) -> String {
        domain::hash(
            serde_json::json!([
                self.literal,
                self.favorites,
                self.kind.map(Kind::as_str),
                self.kind_intersection.map(Kind::as_str),
                self.categories,
                self.source_app_id,
                self.from,
                self.until
            ])
            .to_string()
            .as_bytes(),
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cursor {
    pub revision: i64,
    pub offset: usize,
    pub signature: String,
}
pub struct SearchPage {
    pub revision: i64,
    pub items: Vec<Summary>,
    pub next: Option<Cursor>,
}
#[derive(Clone)]
pub struct Summary {
    pub id: String,
    pub first_copied_at: i64,
    pub last_copied_at: i64,
    pub copy_count: i64,
    pub favorite: bool,
    pub protected: bool,
    pub kind: Kind,
    pub source: Source,
    pub preview: String,
    pub title: Option<String>,
    pub context: String,
}
#[derive(Clone)]
pub enum Change {
    Favorite(Vec<String>, bool),
    Category(Vec<String>, String, bool),
    DeleteCategory(String),
    Delete(Vec<String>),
    ClearOrdinary,
    Retention(u16),
    Quota(u64),
}
#[derive(Clone)]
pub struct Preview {
    pub token: String,
    pub affected_count: usize,
    pub requires_confirmation: bool,
    pub store_revision: i64,
    pub settings_revision: i64,
    pub estimated_reclaim_bytes: u64,
    change: Change,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ChangeResult {
    pub removed: usize,
    pub reclaimed_bytes: u64,
    pub revision: i64,
}
pub struct Usage {
    pub database: u64,
    pub wal: u64,
    pub protected_payload: u64,
    pub ordinary_payload: u64,
    pub common: u64,
}

struct Commit<'a> {
    settings: &'a Settings,
    growth: u64,
    trim: bool,
    allow_over: bool,
    protect: Option<&'a str>,
    operation: &'a str,
}

type Job = Box<dyn FnOnce(&mut Writer) + Send>;
pub struct HistoryStore {
    sender: Option<mpsc::Sender<Job>>,
    worker: Option<thread::JoinHandle<()>>,
    reader: Mutex<Connection>,
    path: PathBuf,
    now: Arc<dyn Fn() -> i64 + Send + Sync>,
}
struct Writer {
    db: Connection,
    options: Options,
    previews: BTreeMap<String, Preview>,
    undo: Option<Undo>,
    last_write: Instant,
}
struct Undo {
    expires: i64,
    records: Vec<Saved>,
    bytes: u64,
}
struct Saved {
    summary: Summary,
    payload: Payload,
    memberships: Vec<(String, i64)>,
}
const PROTECTED: &str =
    "(isFavorite=1 OR EXISTS(SELECT 1 FROM clip_categories WHERE clipID=clips.id))";
const ORDINARY: &str =
    "(isFavorite=0 AND NOT EXISTS(SELECT 1 FROM clip_categories WHERE clipID=clips.id))";

impl HistoryStore {
    pub fn open(path: impl AsRef<Path>, options: Options) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_| StoreError::IoFailure)?;
        }
        let db = Connection::open(&path)?;
        db.busy_timeout(Duration::from_secs(5))?;
        schema::initialize(&db, &options.settings)?;
        let reader = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        reader.busy_timeout(Duration::from_secs(5))?;
        // Keep the search working set across writer commits; budget is 32 MiB, not the full database.
        reader.execute_batch("PRAGMA cache_size=-32768;")?;
        schema::functions(&reader)?;
        let now = options.now.clone();
        let (sender, receiver) = mpsc::channel::<Job>();
        let worker = thread::Builder::new()
            .name("localclip-writer".into())
            .spawn(move || {
                let mut state = Writer {
                    db,
                    options,
                    previews: BTreeMap::new(),
                    undo: None,
                    last_write: Instant::now(),
                };
                loop {
                    match receiver.recv_timeout(Duration::from_millis(1000)) {
                        Ok(job) => {
                            state.clear_expired_undo();
                            job(&mut state);
                        }
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            state.clear_expired_undo();
                            if state.last_write.elapsed() >= Duration::from_secs(1) {
                                let _ = state.db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                }
                let _ = state.db.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);");
            })
            .map_err(|_| StoreError::Unavailable)?;
        Ok(Self {
            sender: Some(sender),
            worker: Some(worker),
            reader: Mutex::new(reader),
            path,
            now,
        })
    }
    fn write<T: Send + 'static>(
        &self,
        f: impl FnOnce(&mut Writer) -> Result<T> + Send + 'static,
    ) -> Result<T> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.sender
            .as_ref()
            .ok_or(StoreError::Unavailable)?
            .send(Box::new(move |w| {
                let _ = tx.send(f(w));
            }))
            .map_err(|_| StoreError::Unavailable)?;
        rx.recv().map_err(|_| StoreError::Unavailable)?
    }
    fn read<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let db = self.reader.lock().map_err(|_| StoreError::Unavailable)?;
        let tx = Transaction::new_unchecked(&db, TransactionBehavior::Deferred)?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
    pub fn settings(&self) -> Result<Settings> {
        self.read(schema::read_settings)
    }
    pub fn ingest(&self, snapshot: Snapshot) -> Result<Ingest> {
        self.write(move |w| w.ingest(snapshot))
    }
    pub fn payload(&self, id: &str) -> Result<Payload> {
        let now = (self.now)();
        self.read(|db| {
            let settings = schema::read_settings(db)?;
            let summary = summary_by_id(db, id)?;
            if !visible(&summary, &settings, now) {
                return Err(StoreError::NotFound);
            }
            read_payload(db, id)
        })
    }
    pub fn summary(&self, id: &str) -> Result<Summary> {
        self.read(|db| {
            let summary = summary_by_id(db, id)?;
            if !visible(&summary, &schema::read_settings(db)?, (self.now)()) {
                return Err(StoreError::NotFound);
            }
            Ok(summary)
        })
    }
    pub fn metadata(&self, id: &str) -> Result<(Vec<String>, u64)> {
        self.read(|db| {
            let mut query = db.prepare("SELECT DISTINCT type FROM representations WHERE clipID=? ORDER BY type")?;
            let types = query.query_map([id], |r| r.get(0))?.collect::<std::result::Result<_, _>>()?;
            let bytes = scalar(db, "SELECT COALESCE(SUM(blobs.byteCount),0) FROM representations JOIN blobs ON blobs.digest=blobDigest WHERE clipID=?", [id])?;
            Ok((types, bytes as u64))
        })
    }
    pub fn memberships(&self, id: &str) -> Result<Vec<String>> {
        self.summary(id)?;
        self.read(|db| {
            let mut stmt = db.prepare("SELECT categoryID FROM clip_categories JOIN categories ON categories.id=categoryID WHERE clipID=? ORDER BY categories.position,categoryID")?;
            Ok(stmt.query_map([id], |r| r.get(0))?.collect::<std::result::Result<_, _>>()?)
        })
    }
    pub fn sources(&self) -> Result<Vec<Source>> {
        self.read(|db| {
            let mut stmt = db.prepare("SELECT sourceAppID,sourceName,MIN(sourceBasis) FROM clips GROUP BY sourceAppID,sourceName ORDER BY sourceName,sourceAppID")?;
            Ok(stmt.query_map([], |r| Ok(Source { app_id:r.get(0)?,name:r.get(1)?,basis:r.get(2)? }))?.collect::<std::result::Result<_, _>>()?)
        })
    }
    pub fn revision(&self) -> Result<i64> {
        self.read(revision)
    }
    pub fn search(&self, request: SearchRequest) -> Result<SearchPage> {
        let now = (self.now)();
        self.read(|db| search_db(db, request, now))
    }
    pub fn preview(&self, change: Change) -> Result<Preview> {
        self.write(move |w| w.preview(change))
    }
    pub fn apply(&self, token: String) -> Result<ChangeResult> {
        self.write(move |w| w.apply(token))
    }
    pub fn expire(&self) -> Result<ChangeResult> {
        self.write(|w| {
            let settings = schema::read_settings(&w.db)?;
            let now = (w.options.now)();
            w.mutate(
                Commit {
                    settings: &settings,
                    growth: 0,
                    trim: false,
                    allow_over: true,
                    protect: None,
                    operation: "",
                },
                |db| {
                    expire_db(db, &settings, now)?;
                    Ok(())
                },
            )
            .map(|(_, r)| r)
        })
    }
    pub fn touch(&self, ids: Vec<String>) -> Result<()> {
        self.write(move |w| {
            let s = schema::read_settings(&w.db)?;
            let now = (w.options.now)();
            let ids = distinct(ids);
            w.mutate(
                Commit {
                    settings: &s,
                    growth: 8192,
                    trim: false,
                    allow_over: true,
                    protect: None,
                    operation: "",
                },
                |db| {
                    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
                    for id in &ids {
                        db.execute(
                            "UPDATE clips SET lastCopiedAtMs=?,copyCount=copyCount+1 WHERE id=?",
                            params![now, id],
                        )?;
                        for cat in strings(
                            db,
                            "SELECT categoryID FROM clip_categories WHERE clipID=?",
                            [id],
                        )? {
                            groups.entry(cat).or_default().push(id.clone());
                        }
                    }
                    for (cat, ids) in groups {
                        prepend(db, &cat, &ids, w.undo.as_ref())?;
                    }
                    Ok(())
                },
            )?;
            Ok(())
        })
    }
    pub fn create_category(&self, name: String, color: String) -> Result<String> {
        self.write(move|w|{let name=category_name(&name)?;if color.len()!=6||!color.bytes().all(|b|b.is_ascii_hexdigit()){return Err(StoreError::InvalidSettings);}let s=schema::read_settings(&w.db)?;let id=new_id(&w.db)?;let now=(w.options.now)();w.mutate(Commit {settings: &s, growth: 8192, trim: false, allow_over: false, protect: None, operation: ""}, |db|{if scalar(db,"SELECT COUNT(*) FROM categories WHERE normalizedName=?",[search::normalize(&name)])?>0{return Err(StoreError::DuplicateName);}db.execute("INSERT INTO categories VALUES(?,?,?,?,?,(SELECT COALESCE(MAX(position),-1)+1 FROM categories))",params![id,name,search::normalize(&name),now,color])?;Ok(())})?;Ok(id)})
    }
    pub fn categories(&self) -> Result<Vec<(String, String, String)>> {
        self.read(|db| {
            let mut q =
                db.prepare("SELECT id,name,colorHex FROM categories ORDER BY position,id")?;
            Ok(q.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                .collect::<rusqlite::Result<_>>()?)
        })
    }
    pub fn rename_category(&self, id: String, name: String, color: String) -> Result<()> {
        self.write(move |w| {
            let name = category_name(&name)?;
            if color.len() != 6 || !color.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(StoreError::InvalidSettings);
            }
            let settings = schema::read_settings(&w.db)?;
            w.mutate(
                Commit {
                    settings: &settings,
                    growth: 8192,
                    trim: false,
                    allow_over: false,
                    protect: None,
                    operation: "",
                },
                |db| {
                    if scalar(db, "SELECT COUNT(*) FROM categories WHERE id=?", [&id])? == 0 {
                        return Err(StoreError::NotFound);
                    }
                    if scalar(
                        db,
                        "SELECT COUNT(*) FROM categories WHERE normalizedName=? AND id<>?",
                        params![search::normalize(&name), id],
                    )? > 0
                    {
                        return Err(StoreError::DuplicateName);
                    }
                    db.execute(
                        "UPDATE categories SET name=?,normalizedName=?,colorHex=? WHERE id=?",
                        params![name, search::normalize(&name), color, id],
                    )?;
                    Ok(())
                },
            )?;
            Ok(())
        })
    }
    pub fn move_category(&self, id: String, delta: i32) -> Result<()> {
        self.write(move |w| {
            let mut ids = strings(&w.db, "SELECT id FROM categories ORDER BY position,id", [])?;
            let from = ids
                .iter()
                .position(|v| *v == id)
                .ok_or(StoreError::NotFound)?;
            let to = (from as i64 + delta as i64).clamp(0, ids.len() as i64 - 1) as usize;
            ids.swap(from, to);
            let settings = schema::read_settings(&w.db)?;
            w.mutate(
                Commit {
                    settings: &settings,
                    growth: 8192,
                    trim: false,
                    allow_over: false,
                    protect: None,
                    operation: "",
                },
                |db| {
                    for (i, id) in ids.iter().enumerate() {
                        db.execute(
                            "UPDATE categories SET position=? WHERE id=?",
                            params![i as i64, id],
                        )?;
                    }
                    Ok(())
                },
            )?;
            Ok(())
        })
    }
    pub fn move_item(&self, id: String, category: String, delta: i32) -> Result<()> {
        self.write(move |w| {
            let mut ids = strings(
                &w.db,
                "SELECT clipID FROM clip_categories WHERE categoryID=? ORDER BY position,clipID",
                [&category],
            )?;
            let from = ids
                .iter()
                .position(|v| *v == id)
                .ok_or(StoreError::NotFound)?;
            let to = (from as i64 + delta as i64).clamp(0, ids.len() as i64 - 1) as usize;
            ids.swap(from, to);
            let settings = schema::read_settings(&w.db)?;
            w.mutate(
                Commit {
                    settings: &settings,
                    growth: 8192,
                    trim: false,
                    allow_over: false,
                    protect: None,
                    operation: "",
                },
                |db| {
                    for (i, id) in ids.iter().enumerate() {
                        db.execute(
                            "UPDATE clip_categories SET position=? WHERE categoryID=? AND clipID=?",
                            params![i as i64, category, id],
                        )?;
                    }
                    Ok(())
                },
            )?;
            Ok(())
        })
    }
    /// 只允许单条纯文本/链接/富文本，隐藏及过期记录不可读取。
    pub fn editable_text(&self, id: &str) -> Result<EditableText> {
        self.read(|db| editable_text(db, id, (self.now)()))
    }
    /// 原位替换正文；元数据不动，重复或并发冲突不合并、不覆盖。
    pub fn edit_text(&self, id: String, expected: String, text: String) -> Result<()> {
        if text.is_empty() || text.contains('\0') || text.len() as u64 > 50 * MIB {
            return Err(StoreError::InvalidText);
        }
        self.write(move |w| {
            let current = editable_text(&w.db, &id, (w.options.now)())?;
            if current.fingerprint != expected {
                return Err(StoreError::StaleEdit);
            }
            // 未改正文保留原始富文本表示及存储修订号。
            if current.text == text {
                return Ok(());
            }
            let payload = Payload::text(&text);
            let hash = payload.fingerprint();
            if scalar(
                &w.db,
                "SELECT COUNT(*) FROM clips WHERE contentHash=? AND id<>?",
                params![hash, id],
            )? > 0
            {
                return Err(StoreError::DuplicateContent);
            }
            let settings = schema::read_settings(&w.db)?;
            w.mutate(
                Commit {
                    settings: &settings,
                    growth: 8192 + payload.byte_count() * 6,
                    trim: false,
                    allow_over: false,
                    protect: Some(&id),
                    operation: "",
                },
                |db| {
                    // 只替换子项，不删除 clip，保留收藏、分类位置、标题、来源与复制时间。
                    db.execute("DELETE FROM items WHERE clipID=?", [&id])?;
                    db.execute("DELETE FROM thumbnails WHERE clipID=?", [&id])?;
                    db.execute(
                        "UPDATE clips SET contentHash=?,kind=?,previewText=? WHERE id=?",
                        params![
                            hash,
                            payload.kind().as_str(),
                            text.graphemes(true).take(180).collect::<String>(),
                            id
                        ],
                    )?;
                    insert_items(db, &id, &payload)?;
                    Ok(())
                },
            )?;
            Ok(())
        })
    }
    pub fn rename(&self, id: String, title: Option<String>) -> Result<()> {
        self.write(move |w| {
            let s = schema::read_settings(&w.db)?;
            let title = title
                .map(|s| s.trim().nfc().collect::<String>())
                .filter(|s| !s.is_empty());
            if title.as_ref().is_some_and(|t| {
                t.len() > 1024
                    || t.chars()
                        .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
            }) {
                return Err(StoreError::InvalidSettings);
            }
            let current = summary_by_id(&w.db, &id)?;
            if !visible(&current, &s, (w.options.now)()) {
                return Err(StoreError::NotFound);
            }
            if current.title == title {
                return Ok(());
            }
            let growth = 8192 + title.as_ref().map_or(0, |v| v.len() as u64 * 6);
            w.mutate(
                Commit {
                    settings: &s,
                    growth,
                    trim: false,
                    allow_over: false,
                    protect: None,
                    operation: "",
                },
                |db| {
                    if db.execute(
                        "UPDATE clips SET title=?,normalizedTitle=? WHERE id=?",
                        params![title, title.as_deref().map(search::normalize), id],
                    )? == 0
                    {
                        return Err(StoreError::NotFound);
                    }
                    Ok(())
                },
            )?;
            Ok(())
        })
    }
    pub fn update_settings(&self, next: Settings) -> Result<Settings> {
        self.write(move |w| {
            let current = schema::read_settings(&w.db)?;
            if !next.validate() {
                return Err(StoreError::InvalidSettings);
            }
            if next.revision != current.revision
                || next.retention_days != current.retention_days
                || next.quota_bytes != current.quota_bytes
            {
                return Err(StoreError::StalePreview);
            }
            let mut next = next;
            next.revision += 1;
            w.mutate(
                Commit {
                    settings: &current,
                    growth: 8192,
                    trim: false,
                    allow_over: true,
                    protect: None,
                    operation: "",
                },
                |db| schema::write_settings(db, &next),
            )?;
            Ok(next)
        })
    }
    pub fn undo_available(&self) -> Result<bool> {
        self.write(|w| {
            w.clear_expired_undo();
            Ok(w.undo.is_some())
        })
    }
    pub fn undo_delete(&self) -> Result<()> {
        self.write(|w| w.undo_delete())
    }
    pub fn can_capture(&self) -> Result<bool> {
        self.write(|w| {
            let settings = schema::read_settings(&w.db)?;
            if used_bytes(&w.db)? < settings.quota_bytes {
                return Ok(true);
            }
            let tx = Transaction::new_unchecked(&w.db, TransactionBehavior::Immediate)?;
            tx.execute(&format!("DELETE FROM clips WHERE {ORDINARY}"), [])?;
            tx.execute("DELETE FROM thumbnails", [])?;
            gc(&tx)?;
            let can = used_bytes(&tx)? < settings.quota_bytes;
            tx.rollback()?;
            Ok(can)
        })
    }
    pub fn usage(&self) -> Result<Usage> {
        let wal = std::fs::metadata(format!("{}-wal", self.path.display()))
            .map(|m| m.len())
            .unwrap_or(0);
        self.read(|db|{let database=used_bytes(db)?;let all=scalar(db,"SELECT COALESCE(SUM(byteCount),0) FROM blobs",[])? as u64;let protected=scalar(db,&format!("SELECT COALESCE(SUM(byteCount),0) FROM blobs WHERE EXISTS(SELECT 1 FROM representations JOIN clips ON clips.id=representations.clipID WHERE blobDigest=blobs.digest AND {PROTECTED})"),[])? as u64;Ok(Usage{database,wal,protected_payload:protected,ordinary_payload:all-protected,common:database.saturating_sub(all)})})
    }
    // Queue the snapshot on the sole writer, excluding concurrent mutations.
    pub fn snapshot(&self, destination: PathBuf) -> Result<()> {
        self.write(move |writer| {
            if destination.exists() {
                return Err(StoreError::IoFailure);
            }
            writer.db.execute(
                "VACUUM INTO ?",
                [destination.to_str().ok_or(StoreError::IoFailure)?],
            )?;
            std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&destination)
                .and_then(|file| file.sync_all())
                .map_err(|_| StoreError::IoFailure)?;
            Ok(())
        })
    }
    pub fn integrity_check(&self) -> Result<String> {
        self.read(|db| Ok(db.query_row("PRAGMA integrity_check", [], |r| r.get(0))?))
    }
}
impl Drop for HistoryStore {
    fn drop(&mut self) {
        drop(self.sender.take());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Writer {
    fn clear_expired_undo(&mut self) {
        if self
            .undo
            .as_ref()
            .is_some_and(|u| u.expires <= (self.options.now)())
        {
            self.undo = None;
        }
    }
    fn mutate<T>(
        &self,
        commit: Commit<'_>,
        f: impl FnOnce(&Connection) -> Result<T>,
    ) -> Result<(T, ChangeResult)> {
        let Commit {
            settings,
            growth,
            trim,
            allow_over,
            protect,
            operation,
        } = commit;
        if growth > 0
            && (self.options.available_space)()? < growth.saturating_mul(2).saturating_add(64 * MIB)
        {
            return Err(StoreError::DiskFull);
        }
        let tx = Transaction::new_unchecked(&self.db, TransactionBehavior::Immediate)?;
        let before = used_bytes(&tx)?;
        let old_count = scalar(&tx, "SELECT COUNT(*) FROM clips", [])?;
        let result = f(&tx)?;
        gc(&tx)?;
        if trim {
            fit(&tx, settings.quota_bytes, protect)?;
        }
        if !allow_over && used_bytes(&tx)? > settings.quota_bytes {
            return Err(StoreError::QuotaFull);
        }
        tx.execute("UPDATE store_meta SET storeRevision=storeRevision+1,lastOperationID=? WHERE singletonID=1",[operation])?;
        (self.options.before_commit)()?;
        let revision = revision(&tx)?;
        let removed = (old_count - scalar(&tx, "SELECT COUNT(*) FROM clips", [])?).max(0) as usize;
        let reclaimed = before.saturating_sub(used_bytes(&tx)?);
        tx.commit()?;
        Ok((
            result,
            ChangeResult {
                removed,
                reclaimed_bytes: reclaimed,
                revision,
            },
        ))
    }
    fn ingest(&mut self, snapshot: Snapshot) -> Result<Ingest> {
        let s = schema::read_settings(&self.db)?;
        if let Some(reason) =
            s.rejection(&snapshot.source, snapshot.sensitive, (self.options.now)())
        {
            return Ok(Ingest::Skipped(reason));
        }
        if snapshot.policy_revision != s.revision {
            return Err(StoreError::PolicyChanged);
        }
        if let Err(reason) = snapshot.payload.validate() {
            return Ok(Ingest::Skipped(reason));
        }
        let hash = snapshot.payload.fingerprint();
        let existing: Option<String> = self
            .db
            .query_row("SELECT id FROM clips WHERE contentHash=?", [&hash], |r| {
                r.get(0)
            })
            .optional()?;
        let existing = match existing {
            Some(id) if visible(&summary_by_id(&self.db, &id)?, &s, snapshot.captured_at) => {
                Some(id)
            }
            _ => None,
        };
        if let Some(id) = &existing {
            if read_payload(&self.db, id)?.canonical() != snapshot.payload.canonical() {
                return Err(StoreError::CorruptPayload);
            }
            let last: String = self.db.query_row(
                "SELECT lastOperationID FROM store_meta WHERE singletonID=1",
                [],
                |r| r.get(0),
            )?;
            if !snapshot.capture_id.is_empty() && last == snapshot.capture_id {
                return Ok(Ingest::Deduplicated(id.clone()));
            }
        }
        let id = existing.clone().unwrap_or(new_id(&self.db)?);
        let growth =
            snapshot.payload.byte_count() + snapshot.payload.items.len() as u64 * 8192 + 65536;
        let write=self.mutate(Commit {settings: &s, growth, trim: true, allow_over: false, protect: Some(&id), operation: &snapshot.capture_id}, |db|{
            if existing.is_some(){db.execute("UPDATE clips SET lastCopiedAtMs=?,copyCount=copyCount+1,sourceAppID=?,sourceName=?,sourceBasis=? WHERE id=?",params![snapshot.captured_at,snapshot.source.app_id,snapshot.source.name,snapshot.source.basis,id])?;for cat in strings(db,"SELECT categoryID FROM clip_categories WHERE clipID=?",[&id])?{prepend(db,&cat,std::slice::from_ref(&id),self.undo.as_ref())?;}}
            else{db.execute("DELETE FROM clips WHERE contentHash=?",[&hash])?;insert_payload(db,&id,&snapshot.payload,&snapshot.source,snapshot.captured_at)?;}Ok(())
        });
        match write {
            Ok(_) => {
                self.last_write = Instant::now();
                Ok(if existing.is_some() {
                    Ingest::Deduplicated(id)
                } else {
                    Ingest::Inserted(id)
                })
            }
            Err(StoreError::QuotaFull) => Ok(Ingest::Skipped("quotaFull")),
            Err(StoreError::DiskFull) => Ok(Ingest::Skipped("diskFull")),
            Err(e) => Err(e),
        }
    }
    fn preview(&mut self, change: Change) -> Result<Preview> {
        let settings = schema::read_settings(&self.db)?;
        let now = (self.options.now)();
        validate_change(&change)?;
        let before = used_bytes(&self.db)?;
        let old_count = scalar(&self.db, "SELECT COUNT(*) FROM clips", [])?;
        let current = revision(&self.db)?;
        let tx = Transaction::new_unchecked(&self.db, TransactionBehavior::Immediate)?;
        let next = apply_change(&tx, &settings, &change, now, self.undo.as_ref())?;
        if matches!(change, Change::Quota(_)) {
            gc(&tx)?;
            fit(&tx, next.quota_bytes, None)?;
        }
        let affected_count =
            (old_count - scalar(&tx, "SELECT COUNT(*) FROM clips", [])?).max(0) as usize;
        gc(&tx)?;
        let reclaimed = before.saturating_sub(used_bytes(&tx)?);
        tx.rollback()?;
        let confirmation = match &change {
            Change::Delete(_) => false,
            Change::ClearOrdinary | Change::DeleteCategory(_) => true,
            Change::Retention(days) => {
                *days != 0 && (settings.retention_days == 0 || *days < settings.retention_days)
            }
            Change::Quota(bytes) => *bytes < settings.quota_bytes,
            _ => affected_count > 0,
        };
        let preview = Preview {
            token: new_id(&self.db)?,
            affected_count,
            requires_confirmation: confirmation,
            store_revision: current,
            settings_revision: settings.revision,
            estimated_reclaim_bytes: reclaimed,
            change,
        };
        if self.previews.len() >= 64 {
            self.previews.clear();
        }
        self.previews.insert(preview.token.clone(), preview.clone());
        Ok(preview)
    }
    fn apply(&mut self, token: String) -> Result<ChangeResult> {
        let preview = self
            .previews
            .remove(&token)
            .ok_or(StoreError::StalePreview)?;
        let settings = schema::read_settings(&self.db)?;
        if preview.store_revision != revision(&self.db)?
            || preview.settings_revision != settings.revision
        {
            return Err(StoreError::StalePreview);
        }
        let now = (self.options.now)();
        let undo = if let Change::Delete(ids) = &preview.change {
            make_undo(&self.db, ids, now)?
        } else {
            None
        };
        let mut next = settings.clone();
        if let Change::Quota(bytes) = preview.change {
            next.quota_bytes = bytes;
        }
        let (_, result) = self.mutate(
            Commit {
                settings: &next,
                growth: 8192,
                trim: matches!(preview.change, Change::Quota(_)),
                allow_over: true,
                protect: None,
                operation: "",
            },
            |db| {
                apply_change(db, &settings, &preview.change, now, self.undo.as_ref())?;
                Ok(())
            },
        )?;
        if matches!(preview.change, Change::Delete(_)) {
            self.undo = undo;
        }
        self.last_write = Instant::now();
        Ok(result)
    }
    fn undo_delete(&mut self) -> Result<()> {
        self.clear_expired_undo();
        let packet = self.undo.as_ref().ok_or(StoreError::NotFound)?;
        let settings = schema::read_settings(&self.db)?;
        let now = (self.options.now)();
        self.mutate(Commit {settings: &settings, growth: packet.bytes, trim: false, allow_over: false, protect: None, operation: ""}, |db|{for saved in &packet.records{
            let mut summary=saved.summary.clone();if summary.source.basis=="owner"&&summary.source.app_id.as_ref().is_some_and(|id|settings.excluded_app_ids.contains(id)){return Err(StoreError::PolicyChanged);}
            let mut memberships = Vec::new();
            for member in &saved.memberships {
                if scalar(db, "SELECT COUNT(*) FROM categories WHERE id=?", [&member.0])? > 0 { memberships.push(member); }
            }
            summary.protected=summary.favorite||!memberships.is_empty();
            if !visible(&summary,&settings,now){return Err(StoreError::NotFound);}
            if scalar(db,"SELECT COUNT(*) FROM clips WHERE id=? OR contentHash=?",params![summary.id,saved.payload.fingerprint()])?>0{return Err(StoreError::StalePreview);}
            insert_payload(db,&summary.id,&saved.payload,&summary.source,summary.first_copied_at)?;
            db.execute("UPDATE clips SET lastCopiedAtMs=?,copyCount=?,isFavorite=?,title=?,normalizedTitle=? WHERE id=?",params![summary.last_copied_at,summary.copy_count,summary.favorite,summary.title,summary.title.as_deref().map(search::normalize),summary.id])?;
            for(cat,position)in memberships{db.execute("INSERT INTO clip_categories VALUES(?,?,?)",params![cat,summary.id,position])?;}
        }Ok(())})?;
        self.undo = None;
        self.last_write = Instant::now();
        Ok(())
    }
}

fn scalar(db: &Connection, sql: &str, values: impl rusqlite::Params) -> Result<i64> {
    Ok(db.query_row(sql, values, |r| r.get(0))?)
}
fn strings(db: &Connection, sql: &str, values: impl rusqlite::Params) -> Result<Vec<String>> {
    let mut q = db.prepare(sql)?;
    Ok(q.query_map(values, |r| r.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}
fn revision(db: &Connection) -> Result<i64> {
    scalar(
        db,
        "SELECT storeRevision FROM store_meta WHERE singletonID=1",
        [],
    )
}
fn new_id(db: &Connection) -> Result<String> {
    Ok(db.query_row("SELECT lower(hex(randomblob(16)))", [], |r| r.get(0))?)
}
fn used_bytes(db: &Connection) -> Result<u64> {
    Ok(
        ((scalar(db, "PRAGMA page_count", [])? - scalar(db, "PRAGMA freelist_count", [])?)
            * scalar(db, "PRAGMA page_size", [])?) as u64,
    )
}
fn gc(db: &Connection) -> Result<()> {
    db.execute("DELETE FROM blobs WHERE NOT EXISTS(SELECT 1 FROM representations WHERE blobDigest=blobs.digest)",[])?;
    Ok(())
}
fn fit(db: &Connection, quota: u64, protected_id: Option<&str>) -> Result<()> {
    if used_bytes(db)? <= quota {
        return Ok(());
    }
    let ids = strings(
        db,
        &format!(
            "SELECT id FROM clips WHERE {ORDINARY} AND (? IS NULL OR id<>?) ORDER BY lastCopiedAtMs,firstCopiedAtMs,id"
        ),
        params![protected_id, protected_id],
    )?;
    for id in ids {
        db.execute("DELETE FROM clips WHERE id=?", [id])?;
        gc(db)?;
        if used_bytes(db)? <= quota {
            break;
        }
    }
    Ok(())
}
fn visible(summary: &Summary, settings: &Settings, now: i64) -> bool {
    summary.protected
        || settings.retention_days == 0
        || summary.last_copied_at > now.saturating_sub(settings.retention_days as i64 * DAY_MS)
}
fn expire_db(db: &Connection, settings: &Settings, now: i64) -> Result<usize> {
    if settings.retention_days == 0 {
        return Ok(0);
    }
    Ok(db.execute(
        &format!("DELETE FROM clips WHERE {ORDINARY} AND lastCopiedAtMs<=?"),
        [now.saturating_sub(settings.retention_days as i64 * DAY_MS)],
    )?)
}
fn extracted(item: &Item) -> Option<String> {
    if let Some(path) = item.file_path() {
        return Some(path);
    }
    if let Some(text) = &item.plain_text {
        return Some(text.clone());
    }
    for t in ["text/plain", "text/uri", "text/html"] {
        if let Some(data) = item.representation(t)
            && let Ok(text) = std::str::from_utf8(data)
        {
            return Some(if t == "text/html" {
                search::html_text(text)
            } else {
                text.into()
            });
        }
    }
    None
}
fn insert_payload(
    db: &Connection,
    id: &str,
    payload: &Payload,
    source: &Source,
    at: i64,
) -> Result<()> {
    let texts: Vec<_> = payload.items.iter().map(extracted).collect();
    let preview = texts
        .iter()
        .filter_map(|t| t.as_deref())
        .collect::<Vec<_>>()
        .join("\n")
        .graphemes(true)
        .take(180)
        .collect::<String>();
    db.execute(
        "INSERT INTO clips VALUES(?,?,?,?,1,?,0,?,?,?,?,NULL,NULL)",
        params![
            id,
            payload.fingerprint(),
            at,
            at,
            payload.kind().as_str(),
            source.app_id,
            source.name,
            source.basis,
            preview
        ],
    )?;
    insert_items(db, id, payload)
}
/// 捕获与编辑共用表示、去重 blob 与全文索引写入。
fn insert_items(db: &Connection, id: &str, payload: &Payload) -> Result<()> {
    let texts: Vec<_> = payload.items.iter().map(extracted).collect();
    for (i, item) in payload.items.iter().enumerate() {
        db.execute(
            "INSERT INTO items VALUES(?,?,?)",
            params![
                id,
                i as i64,
                serde_json::json!(item.declared_types).to_string()
            ],
        )?;
        for rep in &item.representations {
            let digest = domain::hash(&rep.data);
            let blob: Option<Vec<u8>> = db
                .query_row("SELECT payload FROM blobs WHERE digest=?", [&digest], |r| {
                    r.get(0)
                })
                .optional()?;
            if let Some(blob) = blob {
                if blob != rep.data {
                    return Err(StoreError::CorruptPayload);
                }
            } else {
                db.execute(
                    "INSERT INTO blobs VALUES(?,?,?)",
                    params![digest, rep.data.len() as i64, rep.data],
                )?;
            }
            db.execute(
                "INSERT INTO representations VALUES(?,?,?,?)",
                params![id, i as i64, rep.format, digest],
            )?;
        }
        if let Some(text) = &texts[i] {
            db.execute(
                "INSERT INTO text_content VALUES(?,?,?,?)",
                params![id, i as i64, text, search::normalize(text)],
            )?;
        }
    }
    Ok(())
}
/// 在同一读事务或写入队列中取得完整正文及指纹，避免摘要截断或竞态。
fn editable_text(db: &Connection, id: &str, now: i64) -> Result<EditableText> {
    let settings = schema::read_settings(db)?;
    if settings.preferences.hide_contents {
        return Err(StoreError::ContentsHidden);
    }
    let summary = summary_by_id(db, id)?;
    if !visible(&summary, &settings, now) {
        return Err(StoreError::NotFound);
    }
    let payload = read_payload(db, id)?;
    if payload.items.len() != 1
        || !matches!(payload.kind(), Kind::Text | Kind::Url | Kind::RichText)
    {
        return Err(StoreError::NotEditable);
    }
    let text = extracted(&payload.items[0]).ok_or(StoreError::NotEditable)?;
    Ok(EditableText {
        text,
        fingerprint: payload.fingerprint(),
        rich_text: payload.items[0]
            .representations
            .iter()
            .any(|r| matches!(r.format.as_str(), "text/html" | "text/rtf")),
    })
}
fn read_payload(db: &Connection, id: &str) -> Result<Payload> {
    let mut q =
        db.prepare("SELECT itemIndex,declaredTypes FROM items WHERE clipID=? ORDER BY itemIndex")?;
    let rows = q
        .query_map([id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if rows.is_empty() {
        return Err(StoreError::NotFound);
    }
    let mut items = Vec::new();
    for (index, declared) in rows {
        let mut r=db.prepare("SELECT type,payload FROM representations JOIN blobs ON digest=blobDigest WHERE clipID=? AND itemIndex=? ORDER BY type")?;
        let representations = r
            .query_map(params![id, index], |r| {
                Ok(Representation {
                    format: r.get(0)?,
                    data: r.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        let plain_text = db
            .query_row(
                "SELECT plainText FROM text_content WHERE clipID=? AND itemIndex=?",
                params![id, index],
                |r| r.get(0),
            )
            .optional()?;
        items.push(Item {
            representations,
            declared_types: serde_json::from_str(&declared)
                .map_err(|_| StoreError::CorruptPayload)?,
            plain_text,
        });
    }
    Ok(Payload { items })
}
fn summary_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Summary> {
    let kind: String = row.get("kind")?;
    Ok(Summary {
        id: row.get("id")?,
        first_copied_at: row.get("firstCopiedAtMs")?,
        last_copied_at: row.get("lastCopiedAtMs")?,
        copy_count: row.get("copyCount")?,
        favorite: row.get("isFavorite")?,
        protected: row.get("protected")?,
        kind: Kind::parse(&kind).unwrap_or(Kind::Mixed),
        source: Source {
            app_id: row.get("sourceAppID")?,
            name: row.get("sourceName")?,
            basis: row.get("sourceBasis")?,
        },
        preview: row.get("previewText")?,
        title: row.get("title")?,
        context: String::new(),
    })
}
fn summary_by_id(db: &Connection, id: &str) -> Result<Summary> {
    Ok(db.query_row(
        &format!("SELECT *,{PROTECTED} AS protected FROM clips WHERE id=?"),
        [id],
        summary_row,
    )?)
}
fn search_db(db: &Connection, request: SearchRequest, now: i64) -> Result<SearchPage> {
    let revision = revision(db)?;
    let signature = request.signature();
    let offset = if let Some(c) = &request.cursor {
        if c.revision != revision || c.signature != signature {
            return Err(StoreError::InvalidCursor);
        }
        c.offset
    } else {
        0
    };
    let settings = schema::read_settings(db)?;
    let mut clauses = vec![format!("(?=0 OR {PROTECTED} OR lastCopiedAtMs>?)")];
    let mut values = vec![
        Value::Integer(settings.retention_days as i64),
        Value::Integer(now.saturating_sub(settings.retention_days as i64 * DAY_MS)),
    ];
    if request.favorites {
        clauses.push("isFavorite=1".into());
    }
    for kind in request.kind.into_iter().chain(request.kind_intersection) {
        if kind == Kind::Text {
            // 旧 Windows 数据原地兼容，不清库或丢弃富文本表示。by AI.Coding
            clauses.push("kind IN ('text','richText')".into());
        } else if kind == Kind::Image {
            clauses.push("(kind='image' OR (kind='file' AND EXISTS(SELECT 1 FROM text_content WHERE clipID=clips.id) AND NOT EXISTS(SELECT 1 FROM text_content WHERE clipID=clips.id AND clip_image_path(plainText)=0)))".into());
        } else {
            clauses.push("kind=?".into());
            values.push(Value::Text(kind.as_str().into()));
        }
    }
    if let Some(source) = &request.source_app_id {
        clauses.push("COALESCE(sourceAppID,'')=?".into());
        values.push(Value::Text(source.clone()));
    }
    if let Some(from) = request.from {
        clauses.push("lastCopiedAtMs>=?".into());
        values.push(Value::Integer(from));
    }
    if let Some(until) = request.until {
        clauses.push("lastCopiedAtMs<?".into());
        values.push(Value::Integer(until));
    }
    if !request.categories.is_empty() {
        clauses.push(format!(
            "EXISTS(SELECT 1 FROM clip_categories WHERE clipID=clips.id AND categoryID IN({}))",
            vec!["?"; request.categories.len()].join(",")
        ));
        values.extend(request.categories.iter().cloned().map(Value::Text));
    }
    let needle = search::normalize(&request.literal);
    let body = "COALESCE((SELECT group_concat(normalizedText,char(10)) FROM (SELECT normalizedText FROM text_content WHERE clipID=clips.id ORDER BY itemIndex)), '')";
    let rank = if needle.is_empty() {
        "0".into()
    } else {
        format!(
            "min(clip_literal_rank(CAST({body} AS BLOB),?,sourceName),CASE WHEN instr(CAST(COALESCE(normalizedTitle,'') AS BLOB),?)>0 THEN 1 ELSE 4 END)"
        )
    };
    let mut bindings = Vec::new();
    if !needle.is_empty() {
        bindings.push(Value::Blob(needle.as_bytes().into()));
        bindings.push(Value::Blob(needle.into_bytes()));
    }
    bindings.extend(values);
    let order = if request.literal.is_empty() && request.categories.len() == 1 {
        bindings.push(Value::Text(request.categories.first().unwrap().clone()));
        "(SELECT position FROM clip_categories WHERE clipID=ranked.id AND categoryID=?),id"
    } else {
        "matchRank,lastCopiedAtMs DESC,id"
    };
    let limit = if request.page_size == 0 {
        50
    } else {
        request.page_size.min(50)
    };
    bindings.push(Value::Integer((limit + 1) as i64));
    bindings.push(Value::Integer(
        i64::try_from(offset).map_err(|_| StoreError::InvalidCursor)?,
    ));
    let sql = format!(
        "WITH ranked AS (SELECT *,{PROTECTED} AS protected,{rank} AS matchRank FROM clips WHERE {}) SELECT * FROM ranked WHERE matchRank<4 ORDER BY {order} LIMIT ? OFFSET ?",
        clauses.join(" AND ")
    );
    let mut q = db.prepare(&sql)?;
    let mut items = q
        .query_map(params_from_iter(bindings), summary_row)?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let has_next = items.len() > limit;
    items.truncate(limit);
    for item in &mut items {
        if request.literal.is_empty() {
            item.context = item.preview.clone();
        } else {
            let original = strings(
                db,
                "SELECT plainText FROM text_content WHERE clipID=? ORDER BY itemIndex",
                [&item.id],
            )?
            .join("\n");
            item.context = search::context(&original, &request.literal);
        }
    }
    Ok(SearchPage {
        revision,
        next: has_next.then_some(Cursor {
            revision,
            offset: offset + limit,
            signature,
        }),
        items,
    })
}
fn distinct(ids: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    ids.into_iter()
        .filter(|id| seen.insert(id.clone()))
        .collect()
}
fn category_name(value: &str) -> Result<String> {
    let name = value.trim().nfc().collect::<String>();
    if name.is_empty()
        || name.graphemes(true).count() > 64
        || name
            .chars()
            .any(|c| c.is_control() || matches!(c, '\u{2028}' | '\u{2029}'))
    {
        return Err(StoreError::InvalidSettings);
    }
    Ok(name)
}
fn prepend(db: &Connection, category: &str, ids: &[String], undo: Option<&Undo>) -> Result<()> {
    let mut minimum = scalar(
        db,
        "SELECT COALESCE(MIN(position),0) FROM clip_categories WHERE categoryID=?",
        [category],
    )?;
    if let Some(packet) = undo {
        for saved in &packet.records {
            for (cat, pos) in &saved.memberships {
                if cat == category {
                    minimum = minimum.min(*pos);
                }
            }
        }
    }
    let first = minimum
        .checked_sub(ids.len() as i64)
        .ok_or(StoreError::Unavailable)?;
    for (i, id) in ids.iter().enumerate() {
        db.execute("INSERT INTO clip_categories VALUES(?,?,?) ON CONFLICT(categoryID,clipID) DO UPDATE SET position=excluded.position",params![category,id,first+i as i64])?;
    }
    Ok(())
}
fn validate_change(change: &Change) -> Result<()> {
    match change {
        Change::Retention(n) if *n > 365 => Err(StoreError::InvalidSettings),
        Change::Quota(n) if !(100 * MIB..=4096 * MIB).contains(n) => {
            Err(StoreError::InvalidSettings)
        }
        _ => Ok(()),
    }
}
fn apply_change(
    db: &Connection,
    s: &Settings,
    change: &Change,
    now: i64,
    undo: Option<&Undo>,
) -> Result<Settings> {
    let mut next = s.clone();
    match change {
        Change::Favorite(ids, value) => {
            for id in ids {
                let row = summary_by_id(db, id)?;
                if !visible(&row, s, now) {
                    return Err(StoreError::NotFound);
                }
                db.execute(
                    "UPDATE clips SET isFavorite=? WHERE id=?",
                    params![value, id],
                )?;
                if !value && s.retention_days > 0 {
                    db.execute(
                        &format!(
                            "DELETE FROM clips WHERE id=? AND {ORDINARY} AND lastCopiedAtMs<=?"
                        ),
                        params![id, now.saturating_sub(s.retention_days as i64 * DAY_MS)],
                    )?;
                }
            }
        }
        Change::Category(ids, category, included) => {
            if scalar(db, "SELECT COUNT(*) FROM categories WHERE id=?", [category])? == 0 {
                return Err(StoreError::NotFound);
            }
            let ids = distinct(ids.clone());
            for id in &ids {
                if !visible(&summary_by_id(db, id)?, s, now) {
                    return Err(StoreError::NotFound);
                }
                if !included {
                    db.execute(
                        "DELETE FROM clip_categories WHERE categoryID=? AND clipID=?",
                        params![category, id],
                    )?;
                }
            }
            if *included {
                prepend(db, category, &ids, undo)?;
            }
            expire_db(db, s, now)?;
        }
        Change::DeleteCategory(id) => {
            db.execute("DELETE FROM categories WHERE id=?", [id])?;
            expire_db(db, s, now)?;
        }
        Change::Delete(ids) => {
            for id in ids {
                db.execute("DELETE FROM clips WHERE id=?", [id])?;
            }
        }
        Change::ClearOrdinary => {
            db.execute(&format!("DELETE FROM clips WHERE {ORDINARY}"), [])?;
        }
        Change::Retention(days) => {
            next.retention_days = *days;
            next.revision += 1;
            schema::write_settings(db, &next)?;
            expire_db(db, &next, now)?;
        }
        Change::Quota(bytes) => {
            next.quota_bytes = *bytes;
            next.revision += 1;
            schema::write_settings(db, &next)?;
            expire_db(db, &next, now)?;
        }
    }
    Ok(next)
}
fn make_undo(db: &Connection, ids: &[String], now: i64) -> Result<Option<Undo>> {
    let ids = distinct(ids.to_vec());
    if ids.is_empty() || ids.len() > 4096 {
        return Ok(None);
    }
    let mut bytes = ids.len() as u64 * 8192;
    for id in &ids {
        bytes += scalar(
            db,
            "SELECT COALESCE(SUM(byteCount),0) FROM blobs WHERE digest IN(SELECT blobDigest FROM representations WHERE clipID=?)",
            [id],
        )? as u64;
        bytes += scalar(
            db,
            "SELECT COALESCE(SUM(length(CAST(previewText AS BLOB))+COALESCE(length(CAST(title AS BLOB))*2,0)+COALESCE(length(CAST(sourceAppID AS BLOB)),0)+length(CAST(sourceName AS BLOB))+length(CAST(sourceBasis AS BLOB))),0) FROM clips WHERE id=?",
            [id],
        )? as u64;
        bytes += scalar(
            db,
            "SELECT COALESCE(SUM(length(CAST(declaredTypes AS BLOB))),0) FROM items WHERE clipID=?",
            [id],
        )? as u64;
        bytes += scalar(
            db,
            "SELECT COALESCE(SUM(length(CAST(plainText AS BLOB))+length(CAST(normalizedText AS BLOB))),0) FROM text_content WHERE clipID=?",
            [id],
        )? as u64;
    }
    if bytes > 50 * MIB {
        return Ok(None);
    }
    let mut records = Vec::new();
    for id in ids {
        let summary = match summary_by_id(db, &id) {
            Ok(summary) => summary,
            Err(StoreError::NotFound) => continue,
            Err(error) => return Err(error),
        };
        let payload = read_payload(db, &id)?;
        let mut q = db.prepare("SELECT categoryID,position FROM clip_categories WHERE clipID=?")?;
        let memberships = q
            .query_map([&id], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?;
        records.push(Saved {
            summary,
            payload,
            memberships,
        });
    }
    Ok((!records.is_empty()).then_some(Undo {
        expires: now.saturating_add(30000),
        records,
        bytes,
    }))
}
