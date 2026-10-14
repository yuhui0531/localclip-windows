use super::clipboard::{self, Capture, Receipt, Result};
use localclip_core::domain::{Payload, Settings};
use std::sync::{Arc, mpsc};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::{
            DataExchange::{
                AddClipboardFormatListener, GetClipboardSequenceNumber,
                RemoveClipboardFormatListener,
            },
            LibraryLoader::GetModuleHandleW,
        },
        UI::WindowsAndMessaging::*,
    },
    core::w,
};

type Policy = Arc<dyn Fn() -> Result<Settings> + Send + Sync>;
type Captured = Arc<dyn Fn(Result<Capture>, i64, i64) + Send + Sync>;
enum Command {
    Copy(Payload, Option<u32>, mpsc::Sender<Result<(u32, usize)>>),
}
struct State {
    policy: Policy,
    captured: Captured,
    maintenance: Arc<dyn Fn() + Send + Sync>,
    receipt: Option<Receipt>,
    commands: mpsc::Receiver<Command>,
    last_sequence: u32,
}
pub struct Listener {
    window: isize,
    commands: mpsc::Sender<Command>,
    thread: Option<std::thread::JoinHandle<()>>,
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |v| v.as_millis() as i64)
}
unsafe extern "system" fn procedure(window: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    unsafe {
        if message == WM_NCCREATE {
            let create = &*(l.0 as *const CREATESTRUCTW);
            SetWindowLongPtrW(window, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        let pointer = GetWindowLongPtrW(window, GWLP_USERDATA) as *mut State;
        if pointer.is_null() {
            return DefWindowProcW(window, message, w, l);
        }
        let state = &mut *pointer;
        match message {
            WM_CLIPBOARDUPDATE => {
                SetTimer(Some(window), 1, 50, None);
                LRESULT(0)
            }
            WM_TIMER if w.0 == 1 => {
                let _ = KillTimer(Some(window), 1);
                let sequence = GetClipboardSequenceNumber();
                if sequence == state.last_sequence {
                    return LRESULT(0);
                }
                state.last_sequence = sequence;
                let timestamp = now();
                match (state.policy)() {
                    Ok(settings) => {
                        let result = clipboard::capture(&settings, timestamp, &mut state.receipt);
                        (state.captured)(result, timestamp, settings.revision);
                    }
                    Err(reason) => (state.captured)(Err(reason), timestamp, 0),
                }
                LRESULT(0)
            }
            WM_POWERBROADCAST if w.0 == PBT_APMRESUMEAUTOMATIC as usize => {
                (state.maintenance)();
                LRESULT(1)
            }
            WM_TIMER if w.0 == 2 => {
                (state.maintenance)();
                LRESULT(0)
            }
            WM_APP => {
                while let Ok(Command::Copy(payload, expected, reply)) = state.commands.try_recv() {
                    let result = clipboard::write_checked(window, &payload, expected).map(
                        |(receipt, missing)| {
                            let sequence = receipt.sequence;
                            state.receipt = Some(receipt);
                            (sequence, missing)
                        },
                    );
                    let _ = reply.send(result);
                }
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = RemoveClipboardFormatListener(window);
                let _ = KillTimer(Some(window), 1);
                let _ = KillTimer(Some(window), 2);
                let _ = DestroyWindow(window);
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, w, l),
        }
    }
}
impl Listener {
    pub fn start(
        policy: Policy,
        captured: Captured,
        maintenance: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<Self> {
        let (commands, rx) = mpsc::channel();
        let (ready, received) = mpsc::channel();
        let thread = std::thread::spawn(move || unsafe {
            let mut state = Box::new(State {
                policy,
                captured,
                maintenance,
                receipt: None,
                commands: rx,
                last_sequence: GetClipboardSequenceNumber(),
            });
            let instance = match GetModuleHandleW(None) {
                Ok(v) => v,
                Err(_) => {
                    let _ = ready.send(Err("accessUnavailable"));
                    return;
                }
            };
            let class = WNDCLASSW {
                hInstance: instance.into(),
                lpfnWndProc: Some(procedure),
                lpszClassName: w!("LocalClip.Listener"),
                ..Default::default()
            };
            RegisterClassW(&class);
            let window = match CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("LocalClip.Listener"),
                w!("LocalClip"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                Some(instance.into()),
                Some((&mut *state as *mut State).cast()),
            ) {
                Ok(v) => v,
                Err(_) => {
                    let _ = ready.send(Err("accessUnavailable"));
                    return;
                }
            };
            if AddClipboardFormatListener(window).is_err() {
                let _ = DestroyWindow(window);
                let _ = ready.send(Err("accessUnavailable"));
                return;
            }
            let notification = windows::Win32::System::Power::RegisterSuspendResumeNotification(
                windows::Win32::Foundation::HANDLE(window.0),
                DEVICE_NOTIFY_WINDOW_HANDLE,
            )
            .ok();
            SetTimer(Some(window), 2, 60_000, None);
            let _ = ready.send(Ok(window.0 as isize));
            let mut message = MSG::default();
            while GetMessageW(&mut message, None, 0, 0).0 > 0 {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
            if let Some(notification) = notification {
                let _ = windows::Win32::System::Power::UnregisterSuspendResumeNotification(
                    notification,
                );
            }
        });
        match received.recv().map_err(|_| "accessUnavailable")? {
            Ok(window) => Ok(Self {
                window,
                commands,
                thread: Some(thread),
            }),
            Err(error) => {
                let _ = thread.join();
                Err(error)
            }
        }
    }
    pub fn copy(&self, payload: Payload) -> Result<usize> {
        self.copy_checked(payload, None).map(|(_, missing)| missing)
    }
    /// 返回本次写入序号，转换和回退只替换本次仍持有的剪贴板。by AI.Coding
    pub fn copy_checked(&self, payload: Payload, expected: Option<u32>) -> Result<(u32, usize)> {
        let (tx, rx) = mpsc::channel();
        self.commands
            .send(Command::Copy(payload, expected, tx))
            .map_err(|_| "accessUnavailable")?;
        unsafe {
            PostMessageW(
                Some(HWND(self.window as *mut _)),
                WM_APP,
                WPARAM(0),
                LPARAM(0),
            )
            .map_err(|_| "accessUnavailable")?;
        }
        rx.recv().map_err(|_| "accessUnavailable")?
    }
}
impl Drop for Listener {
    fn drop(&mut self) {
        unsafe {
            let _ = PostMessageW(
                Some(HWND(self.window as *mut _)),
                WM_CLOSE,
                WPARAM(0),
                LPARAM(0),
            );
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
