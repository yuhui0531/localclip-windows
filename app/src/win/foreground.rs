use super::{clipboard::Result, source::Target};
use std::sync::{Mutex, mpsc};
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
    pub fn start() -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        let thread = std::thread::spawn(move || unsafe {
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
