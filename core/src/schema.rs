use crate::{
    domain::{Settings, is_image_file},
    search,
    store::{Result, StoreError},
};
use rusqlite::{Connection, functions::FunctionFlags};

pub fn functions(db: &Connection) -> Result<()> {
    let flags = FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC;
    db.create_scalar_function("clip_literal_rank", 3, flags, |ctx| {
        let body = ctx
            .get_raw(0)
            .as_blob()
            .map_err(|e| rusqlite::Error::UserFunctionError(Box::new(e)))?;
        let needle = ctx
            .get_raw(1)
            .as_blob()
            .map_err(|e| rusqlite::Error::UserFunctionError(Box::new(e)))?;
        let source = ctx
            .get_raw(2)
            .as_str()
            .map_err(|e| rusqlite::Error::UserFunctionError(Box::new(e)))?;
        Ok(search::literal_rank(body, needle, source) as i64)
    })?;
    db.create_scalar_function("clip_image_path", 1, flags, |ctx| {
        Ok(i64::from(is_image_file(&ctx.get::<String>(0)?)))
    })?;
    Ok(())
}
pub fn initialize(db: &Connection, settings: &Settings) -> Result<()> {
    let version: i64 = db.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if version != 0 && version != 1 {
        return Err(StoreError::IncompatibleSchema);
    }
    if version == 0 {
        let tables: i64 = db.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
            [],
            |r| r.get(0),
        )?;
        if tables != 0 {
            return Err(StoreError::IncompatibleSchema);
        }
        db.execute_batch("PRAGMA auto_vacuum=FULL;")?;
    }
    db.execute_batch("PRAGMA journal_mode=WAL;PRAGMA synchronous=FULL;PRAGMA foreign_keys=ON;PRAGMA secure_delete=ON;PRAGMA journal_size_limit=16777216;PRAGMA wal_autocheckpoint=0;")?;
    functions(db)?;
    if version == 1 {
        read_settings(db)?;
        return Ok(());
    }
    let tx = rusqlite::Transaction::new_unchecked(db, rusqlite::TransactionBehavior::Immediate)?;
    tx.execute_batch(include_str!("schema.sql"))?;
    write_settings(&tx, settings)?;
    tx.commit()?;
    Ok(())
}
pub fn write_settings(db: &Connection, s: &Settings) -> Result<()> {
    let mut value = serde_json::json!({"retentionDays":s.retention_days,"quotaBytes":s.quota_bytes,"captureEnabled":s.capture_enabled,"pauseUntil":s.pause_until,"excludedAppIDs":s.excluded_app_ids,"revision":s.revision,"moveUsedToFront":s.move_used_to_front});
    value
        .as_object_mut()
        .unwrap()
        .extend(s.preferences.document().as_object().unwrap().clone());
    db.execute(
        "INSERT OR REPLACE INTO settings VALUES(1,1,?)",
        [value.to_string()],
    )?;
    Ok(())
}
pub fn read_settings(db: &Connection) -> Result<Settings> {
    let (version, document): (i64, String) = db.query_row(
        "SELECT schemaVersion,document FROM settings WHERE singletonID=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    if version != 1 {
        return Err(StoreError::IncompatibleSchema);
    }
    let v: serde_json::Value =
        serde_json::from_str(&document).map_err(|_| StoreError::InvalidSettings)?;
    Ok(Settings {
        preferences: crate::domain::Preferences::from_document(&v)
            .ok_or(StoreError::InvalidSettings)?,
        retention_days: v["retentionDays"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .filter(|n| *n <= 365)
            .ok_or(StoreError::InvalidSettings)?,
        quota_bytes: v["quotaBytes"]
            .as_u64()
            .ok_or(StoreError::InvalidSettings)?,
        capture_enabled: v["captureEnabled"]
            .as_bool()
            .ok_or(StoreError::InvalidSettings)?,
        pause_until: v["pauseUntil"].as_i64(),
        excluded_app_ids: v["excludedAppIDs"]
            .as_array()
            .ok_or(StoreError::InvalidSettings)?
            .iter()
            .map(|id| {
                id.as_str()
                    .map(String::from)
                    .ok_or(StoreError::InvalidSettings)
            })
            .collect::<Result<_>>()?,
        revision: v["revision"].as_i64().ok_or(StoreError::InvalidSettings)?,
        move_used_to_front: v["moveUsedToFront"]
            .as_bool()
            .ok_or(StoreError::InvalidSettings)?,
    })
}
