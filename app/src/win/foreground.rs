//! 前台窗口跟踪与失焦通知；不采集窗口内容。by AI.Coding
use super::{clipboard::Result, source::Target};
use std::{
    cell::RefCell,
    sync::{Arc, Mutex, mpsc},
};

// 回调绑定到安装 WinEventHook 的消息线程，避免跨实例共享生命周期。
thread_local! {
    static ON_CHANGE: RefCell<Option<Arc<dyn Fn() + Send + Sync>>> = RefCell::new(None);
}
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{Accessibility::*, WindowsAndMessaging::*},
};

static LAST: Mutex<Option<Target>> = Mutex::new(None);
pub fn recent() -> Option<Target> {
    LAST.lock().ok().and_then(|v| *v)
}
unsafe extern "system" fn changed(
    _: HWINEVENTHOOK,
    _: u32,
    window: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    // 先通知面板；受保护进程无法读取元数据时也必须处理失焦。
    ON_CHANGE.with(|handler| {
        if let Some(handler) = handler.borrow().as_ref() {
            handler();
        }
    });
    if let Some(target) = Target::capture(window)
        && let Ok(mut last) = LAST.lock()
    {
        *last = Some(target);
    }
}
pub struct Tracker {
    id: u32,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Tracker {
    /// 在现有前台事件线程通知调用方，由 UI 线程复核当前前台再隐藏。
    pub fn start(on_change: Arc<dyn Fn() + Send + Sync>) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let thread = std::thread::spawn(move || unsafe {
            ON_CHANGE.with(|handler| *handler.borrow_mut() = Some(on_change));
            let mut message = MSG::default();
            let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE);
            let hook = SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(changed),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            );
            if hook.is_invalid() {
                let _ = tx.send(Err("accessUnavailable"));
                return;
            }
            changed(
                hook,
                EVENT_SYSTEM_FOREGROUND,
                GetForegroundWindow(),
                0,
                0,
                0,
                0,
            );
            let _ = tx.send(Ok(GetCurrentThreadId()));
            while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            let _ = UnhookWinEvent(hook);
        });
        let id = rx.recv().map_err(|_| "accessUnavailable")??;
        Ok(Self {
            id,
            thread: Some(thread),
        })
    }
}
impl Drop for Tracker {
    fn drop(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
