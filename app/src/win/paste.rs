use super::{
    clipboard::Result,
    listener::Listener,
    source::{self, Target},
};
use localclip_core::domain::Payload;
use std::time::{Duration, Instant};
use windows::Win32::{
    Foundation::HWND,
    UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
};

const MODIFIERS: [VIRTUAL_KEY; 8] = [
    VK_LCONTROL,
    VK_RCONTROL,
    VK_LSHIFT,
    VK_RSHIFT,
    VK_LMENU,
    VK_RMENU,
    VK_LWIN,
    VK_RWIN,
];
#[derive(Debug, PartialEq, Eq)]
pub struct Outcome {
    pub status: &'static str,
    pub missing_files: usize,
}
pub fn modifiers_down() -> bool {
    MODIFIERS
        .iter()
        .any(|k| unsafe { GetAsyncKeyState(k.0 as i32) } < 0)
}
pub fn wait_for_modifiers() -> bool {
    let start = Instant::now();
    while modifiers_down() {
        if start.elapsed() >= Duration::from_millis(500) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    true
}
fn key(key: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                },
                ..Default::default()
            },
        },
    }
}
pub fn output(
    listener: &Listener,
    payload: Payload,
    target: Option<Target>,
    direct: bool,
    modifiers_released: bool,
    hide: impl FnOnce(),
) -> Result<Outcome> {
    let missing_files = listener.copy(payload)?;
    let result = |status| {
        Ok(Outcome {
            status,
            missing_files,
        })
    };
    if !direct {
        return result("copied");
    }
    if !modifiers_released {
        return result("modifiersHeld");
    }
    let Some(target) = target else {
        return result("targetUnavailable");
    };
    let Some(process) = target.validate() else {
        return result("targetUnavailable");
    };
    let Some(current) = source::process(std::process::id()) else {
        return result("targetUnavailable");
    };
    if process.elevated && !current.elevated {
        return result("elevatedTarget");
    }
    hide();
    unsafe {
        let window = HWND(target.window as *mut _);
        let _ = SetForegroundWindow(window);
        let start = Instant::now();
        while GetForegroundWindow() != window {
            if start.elapsed() >= Duration::from_millis(150) {
                return result("focusNotRestored");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        if target.validate().is_none() {
            return result("targetUnavailable");
        }
        let mut focus = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if GetGUIThreadInfo(GetWindowThreadProcessId(window, None), &mut focus).is_err()
            || focus.hwndFocus.0 as isize != target.focus
        {
            return result("focusChanged");
        }
        let mut inputs: Vec<_> = MODIFIERS
            .iter()
            .filter(|k| GetAsyncKeyState(k.0 as i32) < 0)
            .map(|k| key(*k, true))
            .collect();
        inputs.extend([
            key(VK_CONTROL, false),
            key(VK_V, false),
            key(VK_V, true),
            key(VK_CONTROL, true),
        ]);
        if GetForegroundWindow() != window {
            return result("focusChanged");
        }
        if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) != inputs.len() as u32 {
            return result("inputRejected");
        }
    }
    result("commandSent")
}
