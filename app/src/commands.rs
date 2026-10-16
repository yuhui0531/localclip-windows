//! 显式 IPC 命令与原生交互边界。by AI.Coding
use crate::{
    desktop::{self, AppState},
    output,
    win::{foreground, listener::now},
};
use localclip_core::{
    domain::{Kind, Preferences, Settings},
    inspect as inspector, query,
    store::{Change, Cursor, HistoryStore, SearchRequest, StoreError, Summary},
    transform::TRANSFORMS,
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    sync::{Arc, atomic::Ordering},
};
use tauri::{Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

type Result<T> = std::result::Result<T, String>;
fn error(e: StoreError) -> String {
    format!("{e:?}")
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v[key].as_str().ok_or_else(|| "invalidRequest".into())
}
fn ids(v: &Value) -> Result<Vec<String>> {
    let list = v["ids"].as_array().ok_or("invalidRequest")?;
    if list.is_empty() || list.len() > 500 {
        return Err("invalidRequest".into());
    }
    let mut seen = BTreeSet::new();
    list.iter()
        .map(|id| {
            id.as_str()
                .filter(|id| id.len() <= 128 && seen.insert((*id).to_string()))
                .map(String::from)
                .ok_or_else(|| "invalidRequest".into())
        })
        .collect()
}
pub fn offset() -> i32 {
    time::UtcOffset::current_local_offset().map_or(0, |v| v.whole_seconds())
}
fn summary(store: &HistoryStore, s: Summary, hide: bool) -> Result<Value> {
    let (types, bytes) = store.metadata(&s.id).map_err(error)?;
    let memberships = store.memberships(&s.id).map_err(error)?;
    let badge = if hide {
        None
    } else {
        inspector::badge(&s.preview, s.preview.len() >= 180, now(), offset())
    };
    Ok(
        json!({"id":s.id,"firstCopiedAtMs":s.first_copied_at,"lastCopiedAtMs":s.last_copied_at,"copyCount":s.copy_count,"favorite":s.favorite,"protected":s.protected,"kind":if s.kind==Kind::RichText {"text"} else {s.kind.as_str()},"sourceName":s.source.name,"sourceAppID":s.source.app_id,"sourceBasis":s.source.basis,"preview":if hide {""} else {&s.preview},"context":if hide {""} else {&s.context},"title":s.title,"categoryIDs":memberships,"types":types,"byteCount":bytes,"badge":badge}),
    )
}
fn settings_document(s: &Settings) -> Value {
    let mut value = s.preferences.document();
    value.as_object_mut().unwrap().extend(json!({"retentionDays":s.retention_days,"quotaBytes":s.quota_bytes,"captureEnabled":s.capture_enabled,"pauseUntil":s.pause_until,"excludedAppIDs":s.excluded_app_ids,"revision":s.revision,"moveUsedToFront":s.move_used_to_front}).as_object().unwrap().clone());
    value
}
fn changed(app: &tauri::AppHandle, kind: &str) {
    let state = app.state::<AppState>();
    if let Ok(revision) = state.runtime.store.revision() {
        let _ = app.emit("store-changed", json!({"revision":revision,"kind":kind}));
    }
}
fn change(store: &HistoryStore, action: Change) -> Result<Value> {
    let p = store.preview(action).map_err(error)?;
    Ok(
        json!({"token":p.token,"affectedCount":p.affected_count,"needsConfirmation":p.requires_confirmation,"message":format!("将移除 {} 条记录",p.affected_count),"estimatedReclaimBytes":p.estimated_reclaim_bytes}),
    )
}
fn perform_output(app: &tauri::AppHandle, request: &Value) -> Result<Value> {
    let state = app.state::<AppState>();
    if !app
        .get_webview_window("main")
        .is_some_and(|w| w.is_visible().unwrap_or(false))
    {
        return Err("panelHidden".into());
    }
    let ids = ids(request)?;
    // 正文载入之前只读计数，防止多选输出先分配过量内存。
    let total = ids.iter().try_fold(0u64, |sum, id| {
        state
            .runtime
            .store
            .metadata(id)
            .map_err(error)
            .and_then(|(_, n)| sum.checked_add(n).ok_or("outputTooLarge".into()))
    })?;
    if total > output::LIMIT {
        return Err("outputTooLarge".into());
    }
    let settings = state.runtime.store.settings().map_err(error)?;
    if settings.preferences.hide_contents && request["transform"].is_string() {
        return Err("contentsHidden".into());
    }
    let records = ids
        .iter()
        .map(|id| state.runtime.store.payload(id).map_err(error))
        .collect::<Result<Vec<_>>>()?;
    let format = string(request, "format")?;
    if !["original", "plainText"].contains(&format) {
        return Err("invalidRequest".into());
    }
    // 智能模式仍先保留文件表示；确认目标后由原生层决定是否转换。by AI.Coding
    let smart_files = settings.preferences.smart_code_file_paste
        && !settings.preferences.hide_contents
        && request["transform"].is_null()
        && request["join"].as_str().unwrap_or("newline") == "newline"
        && records
            .iter()
            .flat_map(|p| &p.items)
            .all(|i| i.file_path().is_some());
    let payload = output::compose(
        records,
        format == "plainText" && !smart_files,
        request["join"].as_str().unwrap_or("newline"),
        request["transform"].as_str(),
        request["language"].as_str(),
        offset(),
    )
    .map_err(String::from)?;
    let action = string(request, "action")?;
    if !["copy", "directPaste"].contains(&action) {
        return Err("invalidRequest".into());
    }
    if request["drag"].as_bool() == Some(true) {
        if settings.preferences.hide_contents {
            return Err("contentsHidden".into());
        }
        // 原生拖拽必须回到拥有窗口输入队列的 UI STA，不能从后台线程启动。
        if state.modal.swap(true, Ordering::SeqCst) {
            return Err("interactionBusy".into());
        }
        let language = request["language"].as_str().map(String::from);
        let drag_app = app.clone();
        let result = (|| {
            let (tx, rx) = std::sync::mpsc::channel();
            app.run_on_main_thread(move || {
                let result = drag_app
                    .get_webview_window("main")
                    .and_then(|w| w.hwnd().ok())
                    .ok_or("dragUnavailable")
                    .and_then(|h| {
                        crate::win::drag::export_drag(
                            windows::Win32::Foundation::HWND(h.0),
                            payload,
                            language,
                            smart_files,
                            &drag_app
                                .state::<AppState>()
                                .runtime
                                .drag_cache
                                .lock()
                                .map_err(|_| "exportCacheUnavailable")?,
                        )
                    });
                let _ = tx.send(result);
            })
            .map_err(|_| "dragUnavailable")?;
            rx.recv().map_err(|_| "dragUnavailable")?
        })();
        state.modal.store(false, Ordering::SeqCst);
        // 拖动期间抑制的失焦不会再重发，因此结束后重新检查。
        desktop::dismiss_if_deactivated(app);
        if result.map_err(String::from)? {
            state.runtime.store.touch(ids).map_err(error)?;
            changed(app, "used");
        }
        return Ok(json!({"status":"dragFinished"}));
    }
    let target = if state.pinned.load(Ordering::SeqCst) {
        foreground::recent()
    } else {
        *state.target.lock().map_err(|_| "targetUnavailable")?
    };
    let result = state
        .runtime
        .output(payload, target, action == "directPaste", true, || {
            desktop::hide_panel(app)
        })
        .map_err(String::from)?;
    if !matches!(result.status, "copied" | "commandSent")
        || result.missing_files > 0
        || result.smart_paste.is_some_and(|s| s != "converted")
    {
        desktop::feedback(
            app,
            json!({"status":result.status,"missingFiles":result.missing_files,"smartPaste":result.smart_paste}),
        );
    }
    state.runtime.store.touch(ids).map_err(error)?;
    desktop::hide_panel(app);
    changed(app, "used");
    Ok(
        json!({"status":result.status,"missingFiles":result.missing_files,"smartPaste":result.smart_paste}),
    )
}
fn dispatch(app: &tauri::AppHandle, name: &str, request: Value) -> Result<Value> {
    let state = app.state::<AppState>();
    let store = &state.runtime.store;
    let settings = store.settings().map_err(error)?;
    let hide = settings.preferences.hide_contents;
    match name {
        "search" => {
            let text = string(&request, "text")?;
            if text.len() > 16_384 {
                return Err("invalidRequest".into());
            }
            let sources = store.sources().map_err(error)?;
            let cats = store
                .categories()
                .map_err(error)?
                .into_iter()
                .map(|(id, name, _)| query::Category { id, name })
                .collect::<Vec<_>>();
            let parsed = query::parse(text, &sources, &cats, now(), offset());
            let explicit_kind = request["kind"].as_str().and_then(Kind::parse);
            let explicit_source = request["sourceAppID"].as_str().filter(|v| !v.is_empty());
            let contradiction = explicit_source
                .zip(parsed.source_app_id.as_deref())
                .is_some_and(|(a, b)| a != b);
            let mut categories = parsed.category_ids;
            if let Some(list) = request["categoryIDs"].as_array() {
                for id in list {
                    categories.insert(id.as_str().ok_or("invalidRequest")?.into());
                }
            }
            let cursor = if request["cursor"].is_object() {
                Some(Cursor {
                    revision: request["cursor"]["revision"]
                        .as_i64()
                        .ok_or("invalidRequest")?,
                    offset: request["cursor"]["offset"]
                        .as_u64()
                        .ok_or("invalidRequest")? as usize,
                    signature: request["cursor"]["signature"]
                        .as_str()
                        .ok_or("invalidRequest")?
                        .into(),
                })
            } else {
                None
            };
            if contradiction {
                return Ok(
                    json!({"items":[],"cursor":null,"revision":store.revision().map_err(error)?,"labels":parsed.labels,"problems":parsed.problems}),
                );
            }
            let page = store
                .search(SearchRequest {
                    literal: parsed.literal,
                    favorites: request["favoritesOnly"].as_bool().unwrap_or(false)
                        || parsed.favorites,
                    kind: explicit_kind.or(parsed.kind),
                    kind_intersection: if explicit_kind.is_some() {
                        parsed.kind
                    } else {
                        None
                    },
                    categories,
                    source_app_id: explicit_source.map(String::from).or(parsed.source_app_id),
                    from: request["from"]
                        .as_i64()
                        .into_iter()
                        .chain(parsed.from)
                        .max(),
                    until: request["until"]
                        .as_i64()
                        .into_iter()
                        .chain(parsed.until)
                        .min(),
                    page_size: request["pageSize"].as_u64().unwrap_or(20).clamp(1, 50) as usize,
                    cursor,
                })
                .map_err(error)?;
            let next = page
                .next
                .map(|c| json!({"revision":c.revision,"offset":c.offset,"signature":c.signature}));
            let items = page
                .items
                .into_iter()
                .map(|s| summary(store, s, hide))
                .collect::<Result<Vec<_>>>()?;
            Ok(
                json!({"items":items,"cursor":next,"revision":page.revision,"labels":parsed.labels,"problems":parsed.problems}),
            )
        }
        "clip_detail" => {
            let id = string(&request, "id")?;
            let s = summary(store, store.summary(id).map_err(error)?, hide)?;
            let payload = store.payload(id).map_err(error)?;
            if request["previewOnly"].as_bool() == Some(true) {
                if hide {
                    return Err("contentsHidden".into());
                }
                let result = crate::win::preview::info(&payload);
                if store.settings().map_err(error)?.preferences.hide_contents {
                    return Err("contentsHidden".into());
                }
                store.summary(id).map_err(error)?;
                return Ok(result);
            }
            let items = payload.items.iter().map(|i|json!({"types":i.representations.iter().map(|r|&r.format).collect::<Vec<_>>(),"plainText":if hide {None} else {i.plain_text.as_deref()},"html":if hide {None} else {i.representation("text/html").and_then(|b|std::str::from_utf8(b).ok())},"hasImage":i.representations.iter().any(|r|r.format.starts_with("image/")) || i.file_path().is_some()})).collect::<Vec<_>>();
            Ok(json!({"categoryIDs":s["categoryIDs"],"summary":s,"items":items}))
        }
        "perform" => perform_output(app, &request),
        "transform_menu" => {
            if hide {
                return Ok(json!([]));
            }
            let text = ids(&request)?
                .iter()
                .map(|id| {
                    store
                        .payload(id)
                        .map_err(error)?
                        .plain_text()
                        .ok_or_else(|| "unsupported".into())
                })
                .collect::<Result<Vec<_>>>()?
                .join("\n");
            Ok(json!(TRANSFORMS.iter().map(|(t,name,title)|json!({"transform":name,"title":title,"applicable":t.applicable(&text,offset())})).collect::<Vec<_>>()))
        }
        "inspect" => {
            if hide {
                return Ok(json!({"rows":[],"actions":[]}));
            }
            let payload = store.payload(string(&request, "id")?).map_err(error)?;
            let text = payload.plain_text().unwrap_or_default();
            let mut result = inspector::full(&text, request["language"].as_str(), now(), offset())
                .unwrap_or(json!({"rows":[],"actions":[]}));
            result["badge"] = json!(inspector::badge(&text, false, now(), offset()));
            Ok(result)
        }
        "save_as" => {
            let id = string(&request, "id")?;
            let s = store.summary(id).map_err(error)?;
            let payload = store.payload(id).map_err(error)?;
            let file = localclip_core::export::prepare(
                &payload,
                request["language"].as_str(),
                request["markup"].as_str(),
            )
            .ok_or("unsupported")?;
            let base = inspector::export_base_name(s.title.as_deref(), s.last_copied_at, offset())
                .ok_or("invalidName")?;
            state.modal.store(true, Ordering::SeqCst);
            let result = crate::win::dialog::save(app, &base, &file);
            state.modal.store(false, Ordering::SeqCst);
            Ok(json!({"saved":result.map_err(String::from)?}))
        }
        "queue_start" => {
            desktop::start_queue(app, ids(&request)?).map_err(String::from)?;
            Ok(state
                .queue
                .0
                .lock()
                .map_err(|_| "queueUnavailable")?
                .document())
        }
        "queue_stop" => {
            desktop::stop_queue(app);
            Ok(json!({"active":false,"position":0,"total":0}))
        }
        "queue_status" => Ok(state
            .queue
            .0
            .lock()
            .map_err(|_| "queueUnavailable")?
            .document()),
        "set_pinned" => {
            state.pinned.store(
                request["value"].as_bool().ok_or("invalidRequest")?,
                Ordering::SeqCst,
            );
            if request["hide"].as_bool() == Some(true) {
                desktop::hide_panel(app);
            }
            Ok(Value::Null)
        }
        "ocr_copy" => {
            let id = string(&request, "id")?;
            let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
            {
                let mut task = state.ocr.lock().map_err(|_| "ocrUnavailable")?;
                if task.is_some() {
                    return Err("ocrBusy".into());
                }
                *task = Some(cancel.clone());
            }
            let languages = request["languages"]
                .as_array()
                .map(|list| {
                    list.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_else(|| vec!["zh-Hans".into(), "zh-Hant".into(), "en-US".into()]);
            let result = state.runtime.ocr(id, &languages, &cancel);
            if let Ok(mut task) = state.ocr.lock() {
                *task = None;
            }
            let (lines, missing) = result.map_err(String::from)?;
            Ok(
                json!({"status":if lines>0 {"ocrCopied"} else {"noText"},"lines":lines,"missingLanguages":missing}),
            )
        }
        "ocr_cancel" => {
            if let Some(task) = state.ocr.lock().map_err(|_| "ocrUnavailable")?.as_ref() {
                task.store(true, Ordering::SeqCst);
            }
            Ok(Value::Null)
        }
        "set_favorite" => {
            let id = string(&request, "id")?;
            let preview = store
                .preview(Change::Favorite(
                    vec![id.into()],
                    request["value"].as_bool().ok_or("invalidRequest")?,
                ))
                .map_err(error)?;
            if preview.requires_confirmation && request["confirmed"].as_bool() != Some(true) {
                return Ok(
                    json!({"confirmation":true,"token":preview.token,"affectedCount":preview.affected_count}),
                );
            }
            store.apply(preview.token).map_err(error)?;
            changed(app, "favorite");
            store
                .summary(id)
                .map_err(error)
                .and_then(|s| summary(store, s, hide))
                .or(Ok(Value::Null))
        }
        "set_title" => {
            let id = string(&request, "id")?;
            store
                .rename(id.into(), request["value"].as_str().map(String::from))
                .map_err(error)?;
            changed(app, "title");
            summary(store, store.summary(id).map_err(error)?, hide)
        }
        "delete" => {
            let p = store
                .preview(Change::Delete(ids(&request)?))
                .map_err(error)?;
            let protected = ids(&request)?
                .iter()
                .filter(|id| store.summary(id).is_ok_and(|s| s.protected))
                .count();
            if p.affected_count > 1 || protected > 0 {
                return Ok(
                    json!({"confirmation":true,"token":p.token,"affectedCount":p.affected_count,"protectedCount":protected}),
                );
            }
            store.apply(p.token).map_err(error)?;
            changed(app, "deleted");
            Ok(json!({"undoAvailable":store.undo_available().map_err(error)?}))
        }
        "undo_delete" => {
            store.undo_delete().map_err(error)?;
            changed(app, "undo");
            Ok(json!({"undoAvailable":false}))
        }
        "categories" => Ok(
            json!({"categories":store.categories().map_err(error)?.into_iter().map(|(id,name,color)|json!({"id":id,"name":name,"color":color})).collect::<Vec<_>>(),"sources":store.sources().map_err(error)?.into_iter().map(|s|json!({"id":s.app_id,"name":s.name})).collect::<Vec<_>>()}),
        ),
        "upsert_category" => {
            if let Some(id) = request["id"].as_str() {
                if let Some(delta) = request["delta"].as_i64() {
                    store
                        .move_category(id.into(), delta.clamp(-1, 1) as i32)
                        .map_err(error)?;
                } else {
                    store
                        .rename_category(
                            id.into(),
                            string(&request, "name")?.into(),
                            string(&request, "color")?.into(),
                        )
                        .map_err(error)?;
                }
            } else {
                store
                    .create_category(
                        string(&request, "name")?.into(),
                        string(&request, "color")?.into(),
                    )
                    .map_err(error)?;
            }
            changed(app, "categories");
            Ok(Value::Null)
        }
        "delete_category" => change(
            store,
            Change::DeleteCategory(string(&request, "id")?.into()),
        ),
        "assign_categories" => {
            if let Some(delta) = request["delta"].as_i64() {
                store
                    .move_item(
                        string(&request, "id")?.into(),
                        string(&request, "categoryID")?.into(),
                        delta.clamp(-1, 1) as i32,
                    )
                    .map_err(error)?;
                changed(app, "categories");
                return Ok(Value::Null);
            }
            change(
                store,
                Change::Category(
                    ids(&request)?,
                    string(&request, "categoryID")?.into(),
                    request["value"].as_bool().ok_or("invalidRequest")?,
                ),
            )
        }
        "settings" => {
            let mut doc = settings_document(&settings);
            doc["autostart"] = json!(app.autolaunch().is_enabled().unwrap_or(false));
            if request["runningApps"].as_bool() == Some(true) {
                doc["runningApps"] = json!(
                    crate::win::source::running_applications()
                        .into_iter()
                        .map(|(id, name)| json!({"id":id,"name":name}))
                        .collect::<Vec<_>>()
                );
            }
            Ok(doc)
        }
        "update_settings" => {
            if request["browseExcluded"].as_bool() == Some(true) {
                state.modal.store(true, Ordering::SeqCst);
                let picked = crate::win::dialog::pick_exe(app);
                state.modal.store(false, Ordering::SeqCst);
                let mut next = settings;
                if let Some(path) = picked.map_err(String::from)? {
                    next.excluded_app_ids.insert(path.to_lowercase());
                    return store
                        .update_settings(next)
                        .map(|s| settings_document(&s))
                        .map_err(error);
                }
                return Ok(settings_document(&next));
            }
            let mut next = settings.clone();
            next.preferences = Preferences::from_document(&request).ok_or("invalidSettings")?;
            next.revision = request["revision"].as_i64().ok_or("invalidSettings")?;
            next.move_used_to_front = request["moveUsedToFront"]
                .as_bool()
                .ok_or("invalidSettings")?;
            next.capture_enabled = request["captureEnabled"]
                .as_bool()
                .ok_or("invalidSettings")?;
            next.pause_until = request["pauseUntil"].as_i64();
            next.excluded_app_ids = request["excludedAppIDs"]
                .as_array()
                .ok_or("invalidSettings")?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(|s| s.to_lowercase())
                        .ok_or("invalidSettings".to_string())
                })
                .collect::<Result<_>>()?;
            if next.revision != settings.revision {
                return Err("stale".into());
            }
            desktop::set_main_shortcut(app, &next.preferences.shortcut).map_err(String::from)?;
            let result = store.update_settings(next);
            if result.is_err() {
                let _ = desktop::set_main_shortcut(app, &settings.preferences.shortcut);
            }
            let saved = result.map_err(error)?;
            if let Some(enabled) = request["autostart"].as_bool() {
                if enabled {
                    app.autolaunch().enable()
                } else {
                    app.autolaunch().disable()
                }
                .map_err(|_| "autostartUnavailable")?;
            }
            state.thumbs.lock().map_err(|_| "cacheUnavailable")?.clear();
            changed(app, "settings");
            desktop::status(app, "ready");
            Ok(settings_document(&saved))
        }
        "preview_change" => {
            let action = if let Some(days) = request["retention"].as_u64() {
                Change::Retention(u16::try_from(days).map_err(|_| "invalidSettings")?)
            } else if let Some(bytes) = request["quota"].as_u64() {
                Change::Quota(bytes)
            } else if request["clearOrdinary"].as_bool() == Some(true) {
                Change::ClearOrdinary
            } else {
                return Err("invalidRequest".into());
            };
            change(store, action)
        }
        "apply_change" => {
            let result = store
                .apply(string(&request, "token")?.into())
                .map_err(error)?;
            changed(app, "changed");
            Ok(
                json!({"undoAvailable":store.undo_available().map_err(error)?,"affectedCount":result.removed,"needsConfirmation":false,"message":"设置已更新"}),
            )
        }
        "usage" => {
            // 手动清理与正在进行的拖放互斥，避免接收方尚未读取就删除。
            let cache = state
                .runtime
                .drag_cache
                .lock()
                .map_err(|_| "exportCacheUnavailable")?;
            if request["clearDragImages"].as_bool() == Some(true) {
                if request["confirmed"].as_bool() != Some(true)
                    || state.modal.load(Ordering::SeqCst)
                {
                    return Err("interactionBusy".into());
                }
                cache.clear().map_err(|_| "exportCacheUnavailable")?;
            }
            let drag_bytes = cache.bytes().map_err(|_| "exportCacheUnavailable")?;
            let u = store.usage().map_err(error)?;
            Ok(
                json!({"dragImageBytes":drag_bytes,"totalBytes":u.database,"ordinaryBytes":u.ordinary_payload,"protectedBytes":u.protected_payload,"walBytes":u.wal,"freeDiskBytes":state.runtime.free_disk_bytes().map_err(String::from)?,"undoAvailable":store.undo_available().map_err(error)?}),
            )
        }
        "capture_status" => Ok(state
            .capture
            .lock()
            .map_err(|_| "stateUnavailable")?
            .clone()),
        "pause" | "resume" => {
            let mut next = settings;
            next.capture_enabled = name == "resume" || request["untilMs"].is_i64();
            next.pause_until = if name == "resume" {
                None
            } else {
                request["untilMs"].as_i64()
            };
            store.update_settings(next).map_err(error)?;
            desktop::status(app, if name == "resume" { "ready" } else { "paused" });
            Ok(Value::Null)
        }
        _ => Err("unknownCommand".into()),
    }
}

#[tauri::command]
pub async fn search(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "search", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn clip_detail(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "clip_detail", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn perform(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "perform", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn transform_menu(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "transform_menu", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn save_as(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "save_as", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn inspect(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "inspect", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn queue_start(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "queue_start", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn queue_stop(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "queue_stop", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn queue_status(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "queue_status", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn set_pinned(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "set_pinned", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn ocr_copy(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "ocr_copy", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn ocr_cancel(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "ocr_cancel", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn set_favorite(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "set_favorite", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn set_title(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "set_title", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn delete(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "delete", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn undo_delete(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "undo_delete", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn categories(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "categories", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn upsert_category(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "upsert_category", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn delete_category(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "delete_category", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn assign_categories(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "assign_categories", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn settings(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "settings", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn update_settings(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "update_settings", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn preview_change(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "preview_change", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn apply_change(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "apply_change", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn usage(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "usage", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn capture_status(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "capture_status", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn pause(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "pause", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}

#[tauri::command]
pub async fn resume(app: tauri::AppHandle, request: Value) -> Result<Value> {
    tauri::async_runtime::spawn_blocking(move || dispatch(&app, "resume", request))
        .await
        .map_err(|_| "commandUnavailable".to_string())?
}
