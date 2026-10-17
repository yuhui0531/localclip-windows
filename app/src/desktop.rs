//! 桌面面板及原生交互协调。by AI.Coding
use crate::{
    queue::Queue,
    runtime::Runtime,
    win::{foreground, listener::now, paste, source::Target},
};
use localclip_core::domain::{Payload, Shortcut as StoredShortcut};
use serde_json::json;
use std::sync::{Arc, Condvar, Mutex, atomic::AtomicBool};
use tauri::{Emitter, Manager, menu::MenuBuilder, tray::TrayIconBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

pub struct AppState {
    pub thumbs: crate::thumbs::SharedCache,
    pub ocr: Mutex<Option<Arc<AtomicBool>>>,
    pub modal: AtomicBool,
    pub capture: Mutex<serde_json::Value>,
    pub feedback_generation: std::sync::atomic::AtomicU64,
    pub runtime: Arc<Runtime>,
    pub target: Mutex<Option<Target>>,
    pub pinned: AtomicBool,
    pub main_shortcut: Mutex<Option<String>>,
    pub queue: Arc<(Mutex<Queue>, Condvar)>,
}
pub fn shortcut_text(shortcut: &StoredShortcut) -> String {
    let mut parts = Vec::new();
    for modifier in &shortcut.modifiers {
        parts.push(
            match modifier.as_str() {
                "ctrl" => "Control",
                "alt" => "Alt",
                "shift" => "Shift",
                "win" => "Super",
                _ => "",
            }
            .to_string(),
        );
    }
    parts.push(shortcut.key.clone());
    parts.join("+")
}
pub fn request_panel(app: &tauri::AppHandle, settings: bool) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if let Ok(mut target) = state.target.lock() {
        *target = foreground::recent();
    }
    let _ = state.runtime.store.expire();
    if let Some(window) = app.get_webview_window("main") {
        position_panel(app, &window);
        let _ = window.show();
        let _ = window.set_focus();
    }
    let _ = app.emit("panel-shown", json!({"settings":settings}));
}
pub fn set_main_shortcut(
    app: &tauri::AppHandle,
    next: &StoredShortcut,
) -> Result<(), &'static str> {
    let state = app.state::<AppState>();
    let mut old = state
        .main_shortcut
        .lock()
        .map_err(|_| "shortcutUnavailable")?;
    let text = shortcut_text(next);
    if old.as_ref() == Some(&text) {
        return Ok(());
    }
    app.global_shortcut()
        .register(text.as_str())
        .map_err(|_| "shortcutConflict")?;
    if let Some(old) = old.as_ref() {
        let _ = app.global_shortcut().unregister(old.as_str());
    }
    *old = Some(text);
    Ok(())
}
fn icon(status: &str) -> tauri::image::Image<'static> {
    let color = match status {
        "paused" => [220, 155, 40, 255],
        "excluded" | "sensitive" => [125, 132, 148, 255],
        "quotaFull" | "diskFull" => [215, 100, 35, 255],
        "storageUnavailable" | "accessUnavailable" => [210, 55, 65, 255],
        _ => [35, 160, 135, 255],
    };
    let mut pixels = vec![0; 16 * 16 * 4];
    for y in 2..15 {
        for x in 3..13 {
            let index = (y * 16 + x) * 4;
            pixels[index..index + 4].copy_from_slice(&color);
        }
    }
    for y in [6, 9, 12] {
        for x in 5..11 {
            let index = (y * 16 + x) * 4;
            pixels[index..index + 4].copy_from_slice(&[255, 255, 255, 255]);
        }
    }
    tauri::image::Image::new_owned(pixels, 16, 16)
}
pub fn status(app: &tauri::AppHandle, reason: &'static str) {
    let display = app
        .try_state::<AppState>()
        .and_then(|s| s.runtime.store.settings().ok())
        .filter(|s| !s.capture_enabled || s.pause_until.is_some_and(|until| until > now()))
        .map_or(reason, |_| "paused");
    if let Some(state) = app.try_state::<AppState>()
        && let Ok(mut capture) = state.capture.lock()
    {
        *capture = json!({"state":display,"lastSkip":if matches!(reason,"ready"|"inserted"|"deduplicated"|"expired"|"selfWrite") {serde_json::Value::Null} else {json!({"reason":reason,"atMs":now()})}});
    }
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_icon(Some(icon(display)));
        let _ = tray.set_tooltip(Some(format!("LocalClip · {display}")));
    }
    let _ = app.emit("capture-status", json!({"state":display}));
    if matches!(reason, "inserted" | "deduplicated")
        && app
            .try_state::<AppState>()
            .and_then(|s| s.runtime.store.settings().ok())
            .is_some_and(|s| s.preferences.capture_sounds)
    {
        unsafe {
            let _ = windows::Win32::System::Diagnostics::Debug::MessageBeep(
                windows::Win32::UI::WindowsAndMessaging::MB_OK,
            );
        }
    }
    if matches!(reason, "inserted" | "deduplicated" | "expired") {
        let revision = app
            .try_state::<AppState>()
            .and_then(|s| s.runtime.store.revision().ok())
            .unwrap_or(0);
        let _ = app.emit("store-changed", json!({"revision":revision,"kind":reason}));
    }
}
pub fn start_queue(app: &tauri::AppHandle, ids: Vec<String>) -> Result<(), &'static str> {
    if ids.len() < 2 {
        return Err("selectionTooSmall");
    }
    let state = app.state::<AppState>();
    let settings = state
        .runtime
        .store
        .settings()
        .map_err(|_| "storageUnavailable")?;
    let next = shortcut_text(&settings.preferences.queue_shortcut);
    let (lock, wake) = &*state.queue;
    let mut queue = lock.lock().map_err(|_| "queueUnavailable")?;
    if queue.shortcut.as_ref() != Some(&next) {
        app.global_shortcut()
            .register(next.as_str())
            .map_err(|_| "queueShortcutConflict")?;
        if let Some(old) = queue.shortcut.take() {
            let _ = app.global_shortcut().unregister(old.as_str());
        }
    }
    queue.start(ids, now())?;
    queue.shortcut = Some(next);
    feedback(
        app,
        json!({"status":"queueStarted","total":queue.ids.len()}),
    );
    wake.notify_all();
    let _ = app.emit("queue-changed", queue.document());
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    Ok(())
}
pub fn stop_queue(app: &tauri::AppHandle) {
    let state = app.state::<AppState>();
    let (lock, wake) = &*state.queue;
    if let Ok(mut queue) = lock.lock() {
        if let Some(shortcut) = queue.shortcut.take() {
            let _ = app.global_shortcut().unregister(shortcut.as_str());
        }
        queue.stop();
        wake.notify_all();
        let _ = app.emit("queue-changed", queue.document());
    }
}
fn queue_next(app: &tauri::AppHandle) {
    let target = foreground::recent();
    let state = app.state::<AppState>();
    let generation = {
        let (lock, wake) = &*state.queue;
        let Ok(mut queue) = lock.lock() else {
            return;
        };
        if !queue.begin(now(), target.is_some()) {
            return;
        }
        wake.notify_all();
        queue.generation
    };
    let app = app.clone();
    std::thread::spawn(move || {
        let released = paste::wait_for_modifiers();
        let state = app.state::<AppState>();
        let mut skipped = Vec::new();
        loop {
            let next = {
                let Ok(mut queue) = state.queue.0.lock() else {
                    return;
                };
                if queue.generation != generation {
                    return;
                }
                queue.next_item()
            };
            let Some((index, id)) = next else {
                if !skipped.is_empty() {
                    feedback(&app, json!({"status":"queueSkipped","skipped":skipped}));
                }
                break;
            };
            let payload = match state.runtime.store.payload(&id) {
                Ok(v) => v,
                Err(localclip_core::store::StoreError::NotFound) => {
                    skipped.push(index);
                    continue;
                }
                Err(_) => {
                    feedback(&app, json!({"status":"storageUnavailable"}));
                    break;
                }
            };
            let Ok(settings) = state.runtime.store.settings() else {
                break;
            };
            let payload = if settings.preferences.default_format == "plainText" {
                match payload.plain_text() {
                    Some(text) => Payload::text(&text),
                    None => {
                        feedback(&app, json!({"status":"unsupported"}));
                        break;
                    }
                }
            } else {
                payload
            };
            // Serialize replacement with dispatch so a superseded queue cannot send later.
            let Ok(queue_guard) = state.queue.0.lock() else {
                return;
            };
            if queue_guard.generation != generation {
                return;
            }
            let result = state.runtime.output(
                payload,
                target,
                settings.preferences.default_action == "directPaste",
                released,
                || {},
            );
            drop(queue_guard);
            let reason = match result {
                Ok(result) => {
                    if matches!(result.status, "commandSent" | "copied" | "modifiersHeld") {
                        let _ = state.runtime.store.touch(vec![id]);
                    }
                    result.status
                }
                Err(reason) => reason,
            };
            feedback(
                &app,
                json!({"status":reason,"position":index,"skipped":skipped}),
            );
            break;
        }
        let done = {
            let Ok(mut queue) = state.queue.0.lock() else {
                return;
            };
            if queue.generation != generation {
                return;
            }
            queue.finish(generation);
            let _ = app.emit("queue-changed", queue.document());
            !queue.active()
        };
        if done {
            stop_queue(&app);
        }
    });
}
fn idle_watch(app: tauri::AppHandle, queue: Arc<(Mutex<Queue>, Condvar)>) {
    std::thread::spawn(move || {
        let (lock, wake) = &*queue;
        let Ok(mut state) = lock.lock() else {
            return;
        };
        loop {
            if state.stopped {
                return;
            }
            if !state.active() {
                state = match wake.wait(state) {
                    Ok(v) => v,
                    Err(_) => return,
                };
                continue;
            }
            if state.expired(now()) {
                drop(state);
                stop_queue(&app);
                state = match lock.lock() {
                    Ok(v) => v,
                    Err(_) => return,
                };
                continue;
            }
            let delay = 600_000i64
                .saturating_sub(now().saturating_sub(state.last_activity))
                .max(1) as u64;
            state = match wake.wait_timeout(state, std::time::Duration::from_millis(delay)) {
                Ok((v, _)) => v,
                Err(_) => return,
            };
        }
    });
}
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let startup_began = std::time::Instant::now();
    tauri::Builder::default()
        .manage(crate::update::State::default())
        .invoke_handler(tauri::generate_handler![
            crate::commands::search,
            crate::commands::clip_detail,
            crate::commands::perform,
            crate::commands::transform_menu,
            crate::commands::save_as,
            crate::commands::inspect,
            crate::commands::open_item,
            crate::commands::queue_start,
            crate::commands::queue_stop,
            crate::commands::queue_status,
            crate::commands::set_pinned,
            crate::commands::set_detail_view,
            crate::commands::start_window_drag,
            crate::commands::start_window_resize,
            crate::commands::ocr_copy,
            crate::commands::ocr_cancel,
            crate::commands::set_favorite,
            crate::commands::set_title,
            crate::commands::edit_text,
            crate::commands::delete,
            crate::commands::undo_delete,
            crate::commands::categories,
            crate::commands::upsert_category,
            crate::commands::delete_category,
            crate::commands::assign_categories,
            crate::commands::settings,
            crate::commands::update_settings,
            crate::commands::preview_change,
            crate::commands::apply_change,
            crate::commands::usage,
            crate::commands::capture_status,
            crate::commands::pause,
            crate::commands::resume,
            crate::update::update_check,
            crate::update::update_install
        ])
        .register_asynchronous_uri_scheme_protocol("localclip", |context, request, responder| {
            let app = context.app_handle().clone();
            std::thread::spawn(move || {
                responder.respond(crate::thumbs::response(&app, request.uri()))
            });
        })
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let tauri::WindowEvent::Focused(false) = event {
                dismiss_if_deactivated(window.app_handle());
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                hide_panel(window.app_handle());
            }
        })
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            request_panel(app, false)
        }))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state != ShortcutState::Pressed {
                        return;
                    }
                    let state = app.state::<AppState>();
                    let main = state
                        .main_shortcut
                        .lock()
                        .ok()
                        .and_then(|s| s.as_ref().and_then(|v| v.parse::<Shortcut>().ok()));
                    if main.as_ref() == Some(shortcut) {
                        request_panel(app, false);
                    } else {
                        queue_next(app);
                    }
                })
                .build(),
        )
        .setup(move |app| {
            let handle = app.handle().clone();
            let root = std::env::args()
                .collect::<Vec<_>>()
                .windows(2)
                .find(|v| v[0] == "--data-dir")
                .map(|v| std::path::PathBuf::from(&v[1]))
                .unwrap_or(app.path().local_data_dir()?.join("LocalClip/History"));
            let foreground_handle = app.handle().clone();
            let runtime = Runtime::open(
                &root,
                Arc::new(move |reason| status(&handle, reason)),
                Arc::new(move || {
                    // WebView2 的 LostFocus 不等价于顶层窗口失活；使用现有 WinEventHook。
                    let handle = foreground_handle.clone();
                    let _ = foreground_handle
                        .run_on_main_thread(move || dismiss_if_deactivated(&handle));
                }),
            )
            .map_err(std::io::Error::other)?;
            let settings = runtime
                .store
                .settings()
                .map_err(|_| std::io::Error::other("storageUnavailable"))?;
            let queue = Arc::new((Mutex::new(Queue::default()), Condvar::new()));
            app.manage(AppState {
                thumbs: Mutex::new(crate::thumbs::Cache::default()),
                ocr: Mutex::new(None),
                modal: AtomicBool::new(false),
                capture: Mutex::new(json!({"state":"ready"})),
                feedback_generation: std::sync::atomic::AtomicU64::new(0),
                runtime: Arc::new(runtime),
                target: Mutex::new(None),
                pinned: AtomicBool::new(false),
                main_shortcut: Mutex::new(None),
                queue: queue.clone(),
            });
            let menu = MenuBuilder::new(app)
                .text("history", "打开历史")
                .text("pause15", "暂停 15 分钟")
                .text("pause60", "暂停 1 小时")
                .text("pause", "暂停直到恢复")
                .text("resume", "恢复采集")
                .separator()
                .text("settings", "设置")
                .text("about", "关于 LocalClip")
                .text("updates", "检查更新")
                .text("stopQueue", "结束粘贴队列")
                .separator()
                .text("quit", "退出")
                .build()?;
            if let Some(item) = menu.get("updates").and_then(|v| v.as_menuitem().cloned()) {
                *app.state::<crate::update::State>().menu.lock().unwrap() = Some(item);
            }
            TrayIconBuilder::with_id("main")
                .icon(icon("ready"))
                .tooltip("LocalClip")
                .menu(&menu)
                .on_menu_event(|app, event| {
                    let action = event.id.as_ref();
                    match action {
                        "history" => request_panel(app, false),
                        "settings" => request_panel(app, true),
                        "about" | "updates" => {
                            request_panel(app, true);
                            let _ = app.emit("settings-page", json!({"tab":"about"}));
                            if action == "updates" {
                                let handle = app.clone();
                                tauri::async_runtime::spawn(async move {
                                    if let Err(reason) =
                                        crate::update::check(handle.clone(), true).await
                                    {
                                        feedback(&handle, json!({"status":reason}));
                                    }
                                });
                            }
                        }
                        "stopQueue" => stop_queue(app),
                        "quit" => app.exit(0),
                        "pause15" | "pause60" | "pause" | "resume" => {
                            let state = app.state::<AppState>();
                            if let Ok(mut settings) = state.runtime.store.settings() {
                                settings.capture_enabled = action != "pause";
                                settings.pause_until = match action {
                                    "pause15" => Some(now() + 900_000),
                                    "pause60" => Some(now() + 3_600_000),
                                    _ => None,
                                };
                                let result = state.runtime.store.update_settings(settings);
                                status(
                                    app,
                                    if result.is_err() {
                                        "storageUnavailable"
                                    } else if action == "resume" {
                                        "ready"
                                    } else {
                                        "paused"
                                    },
                                );
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;
            if let Err(reason) = set_main_shortcut(app.handle(), &settings.preferences.shortcut) {
                status(app.handle(), reason);
            }
            if app.get_webview_window("main").is_none() {
                tauri::WebviewWindowBuilder::new(
                    app,
                    "main",
                    tauri::WebviewUrl::App("index.html".into()),
                )
                .title("LocalClip")
                // Windows 默认拦截器会阻断卡片的 HTML dragstart。
                .disable_drag_drop_handler()
                .visible(false)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .inner_size(1280.0, 340.0)
                .min_inner_size(640.0, 220.0)
                .resizable(false)
                .build()?;
            }
            if app.get_webview_window("feedback").is_none() {
                tauri::WebviewWindowBuilder::new(
                    app,
                    "feedback",
                    tauri::WebviewUrl::App("index.html#feedback".into()),
                )
                .title("LocalClip feedback")
                .visible(false)
                .focused(false)
                .focusable(false)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .transparent(true)
                .inner_size(660.0, 74.0)
                .resizable(false)
                .build()?
                .set_ignore_cursor_events(true)?;
            }
            status(app.handle(), "ready");
            idle_watch(app.handle().clone(), queue);
            crate::update::startup(app.handle(), startup_began);
            Ok(())
        })
        .build(tauri::generate_context!())?
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                let state = app.state::<AppState>();
                state.runtime.shutdown();
                let _ = app.global_shortcut().unregister_all();
                if let Ok(mut queue) = state.queue.0.lock() {
                    queue.stopped = true;
                    state.queue.1.notify_all();
                }
            }
        });
    Ok(())
}

/// 在执行隐藏时复核真实前台，避免排队的旧事件隐藏刚重新打开的面板。
pub fn dismiss_if_deactivated(app: &tauri::AppHandle) {
    use std::sync::atomic::Ordering;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if state.pinned.load(Ordering::SeqCst) || state.modal.load(Ordering::SeqCst) {
        return;
    }
    let foreground = unsafe { GetForegroundWindow() };
    // 同进程的面板、子窗口或对话框之间切换不应触发收起。
    if !foreground.is_invalid() && crate::win::source::window_pid(foreground) != std::process::id()
    {
        hide_panel(app);
    }
}

pub fn hide_panel(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    if let Some(state) = app.try_state::<AppState>()
        && let Ok(mut cache) = state.thumbs.lock()
    {
        cache.clear();
    }
}
/// 普通历史沿屏幕底部显示；每次召回都恢复用户保存的高度。
fn position_panel(app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
    let Ok(cursor) = app.cursor_position() else {
        return;
    };
    let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y) else {
        return;
    };
    let _ = layout_panel(app, window, &monitor, PanelView::History);
}

/// 详情和编辑共用系统可缩放布局，历史和设置保持原有固定布局。
#[derive(Clone, Copy, PartialEq, Eq)]
enum PanelView {
    History,
    Detail,
    Editor,
    Settings,
}

/// 仅切换受限的历史、详情或编辑布局，不开放任意窗口尺寸或位置。
pub fn set_detail_view(
    app: &tauri::AppHandle,
    expanded: bool,
    editor: bool,
    detail: bool,
) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("panelHidden")?;
    if expanded && !window.is_visible().unwrap_or(false) {
        return Err("panelHidden".into());
    }
    let monitor = window
        .current_monitor()
        .map_err(|_| "previewUnavailable")?
        .ok_or("previewUnavailable")?;
    let view = if !expanded {
        PanelView::History
    } else if editor {
        PanelView::Editor
    } else if detail {
        PanelView::Detail
    } else {
        PanelView::Settings
    };
    layout_panel(app, &window, &monitor, view)
}

/// 标题栏使用系统拖动当前可见窗口，不接收外部路径或任意窗口句柄。
pub fn start_window_drag(app: &tauri::AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("panelHidden")?;
    if !window.is_visible().unwrap_or(false) {
        return Err("panelHidden".into());
    }
    window
        .start_dragging()
        .map_err(|_| "previewUnavailable".into())
}

/// 边缘与四角调用系统缩放，历史和设置模式禁止启动缩放会话。
pub fn start_window_resize(app: &tauri::AppHandle, direction: &str) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("panelHidden")?;
    if !window.is_visible().unwrap_or(false) || !window.is_resizable().unwrap_or(false) {
        return Err("panelHidden".into());
    }
    // 由 Tauri 的封闭枚举校验八个方向，拒绝其他字符串。
    let direction = serde_json::from_value(json!(direction)).map_err(|_| "invalidRequest")?;
    let webview: &tauri::Webview = window.as_ref();
    webview
        .window()
        .start_resize_dragging(direction)
        .map_err(|_| "previewUnavailable".into())
}

/// 编辑和详情在当前工作区居中，退出时恢复底部面板；不修改设置或粘贴目标。
fn layout_panel(
    app: &tauri::AppHandle,
    window: &tauri::WebviewWindow,
    monitor: &tauri::Monitor,
    view: PanelView,
) -> Result<(), String> {
    let area = monitor.work_area();
    let scale = monitor.scale_factor();
    let dialog = matches!(view, PanelView::Detail | PanelView::Editor);
    // 详情切换到编辑时保留用户刚拖动、缩放的几何，避免重置位置和草稿视口。
    if dialog && window.is_resizable().unwrap_or(false) {
        return Ok(());
    }
    window
        .set_resizable(dialog)
        .map_err(|_| "previewUnavailable")?;
    let logical_height = app
        .state::<AppState>()
        .runtime
        .store
        .settings()
        .map(|s| s.preferences.panel_height)
        .unwrap_or(340);
    let (width, height) = if view != PanelView::History {
        let margin = (24.0 * scale).round() as u32;
        let (logical_width, logical_height) = if dialog {
            (800.0, 600.0)
        } else {
            (1200.0, 800.0)
        };
        (
            ((logical_width * scale).round() as u32)
                .min(area.size.width.saturating_sub(margin).max(1)),
            ((logical_height * scale).round() as u32)
                .min(area.size.height.saturating_sub(margin).max(1)),
        )
    } else {
        (
            area.size.width,
            ((logical_height as f64 * scale).round() as u32).min(area.size.height),
        )
    };
    let x = area.position.x + (area.size.width - width) as i32 / 2;
    let y = area.position.y
        + if view != PanelView::History {
            (area.size.height - height) as i32 / 2
        } else {
            (area.size.height - height) as i32
        };
    // 两种弹窗共用最小尺寸；低分辨率/高 DPI 工作区进一步收紧，保证仍可关闭。
    let min_width: f64 = if dialog { 480.0 } else { 640.0 };
    let min_height: f64 = if dialog { 360.0 } else { 220.0 };
    window
        .set_min_size(Some(tauri::LogicalSize::new(
            min_width.min(width as f64 / scale),
            min_height.min(height as f64 / scale),
        )))
        .map_err(|_| "previewUnavailable")?;
    window
        .set_size(tauri::PhysicalSize::new(width, height))
        .map_err(|_| "previewUnavailable")?;
    window
        .set_position(tauri::PhysicalPosition::new(x, y))
        .map_err(|_| "previewUnavailable".to_string())
}

pub fn feedback(app: &tauri::AppHandle, value: serde_json::Value) {
    let state = app.state::<AppState>();
    let generation = state
        .feedback_generation
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        + 1;
    if let Some(window) = app.get_webview_window("feedback") {
        if let Ok(cursor) = app.cursor_position()
            && let Ok(Some(monitor)) = app.monitor_from_point(cursor.x, cursor.y)
        {
            let area = monitor.work_area();
            let scale = monitor.scale_factor();
            let width = (660.0 * scale) as u32;
            let height = (74.0 * scale) as u32;
            let _ = window.set_size(tauri::PhysicalSize::new(width, height));
            let _ = window.set_position(tauri::PhysicalPosition::new(
                area.position.x + (area.size.width.saturating_sub(width) / 2) as i32,
                area.position.y + area.size.height as i32 - height as i32,
            ));
        }
        let _ = window.show();
    }
    let _ = app.emit("feedback", value);
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(4));
        if app
            .state::<AppState>()
            .feedback_generation
            .load(std::sync::atomic::Ordering::SeqCst)
            == generation
            && let Some(window) = app.get_webview_window("feedback")
        {
            let _ = window.hide();
        }
    });
}
