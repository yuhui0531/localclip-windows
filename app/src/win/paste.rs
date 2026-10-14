use super::{
    clipboard::Result,
    code_file::{Files, TextTarget},
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
    pub smart_paste: Option<&'static str>,
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
    output_with_code_files(
        listener,
        payload,
        target,
        direct,
        modifiers_released,
        hide,
        || false,
    )
}

/// 焦点恢复后才读取代码正文，最后复查设置、文件、控件和剪贴板身份。by AI.Coding
pub fn output_with_code_files(
    listener: &Listener,
    payload: Payload,
    target: Option<Target>,
    direct: bool,
    modifiers_released: bool,
    hide: impl FnOnce(),
    code_files_allowed: impl Fn() -> bool,
) -> Result<Outcome> {
    let (mut sequence, missing_files) = listener.copy_checked(payload.clone(), None)?;
    let result = |status| {
        Ok(Outcome {
            status,
            missing_files,
            smart_paste: None,
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
        loop {
            let mut focus = GUITHREADINFO {
                cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                ..Default::default()
            };
            // 前台 HWND 可先于子控件恢复；两者都吻合后才继续。by AI.Coding
            if GetForegroundWindow() == window
                && GetGUIThreadInfo(GetWindowThreadProcessId(window, None), &mut focus).is_ok()
                && focus.hwndFocus.0 as isize == target.focus
            {
                break;
            }
            if start.elapsed() >= Duration::from_millis(150) {
                return result(if GetForegroundWindow() == window {
                    "focusChanged"
                } else {
                    "focusNotRestored"
                });
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        if target.validate().is_none() {
            return result("targetUnavailable");
        }
        // 文件句柄存活到 SendInput 完成，转换不适用于复制、拖出或未知输入控件。
        let eligible = code_files_allowed()
            && missing_files == 0
            && payload.items.iter().all(|i| i.file_path().is_some());
        let mut smart_paste = None;
        let mut files = None;
        let mut text_target = None;
        if eligible {
            text_target = TextTarget::capture(target);
            if let Some(text_target) = &text_target {
                match Files::open(&payload)
                    .and_then(|mut files| files.read().map(|text| (files, text)))
                {
                    Ok((locked, text)) => {
                        if !text_target.is_current()
                            || !locked.is_current()
                            || !code_files_allowed()
                        {
                            return result("focusChanged");
                        }
                        sequence = listener
                            .copy_checked(Payload::text(&text), Some(sequence))?
                            .0;
                        files = Some(locked);
                        smart_paste = Some("converted");
                    }
                    Err(reason) => smart_paste = Some(reason),
                }
            } else {
                smart_paste = Some("codeTargetUnknown");
            }
        }
        if smart_paste == Some("converted")
            && (!code_files_allowed()
                || !text_target.as_ref().is_some_and(TextTarget::is_current)
                || !files.as_ref().is_some_and(Files::is_current))
        {
            listener.copy_checked(payload.clone(), Some(sequence))?;
            return result("focusChanged");
        }
        if windows::Win32::System::DataExchange::GetClipboardSequenceNumber() != sequence {
            return result("clipboardChanged");
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
            if smart_paste == Some("converted") {
                listener.copy_checked(payload.clone(), Some(sequence))?;
            }
            return result("focusChanged");
        }
        if SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) != inputs.len() as u32 {
            if smart_paste == Some("converted") {
                listener.copy_checked(payload, Some(sequence))?;
            }
            return result("inputRejected");
        }

        Ok(Outcome {
            status: "commandSent",
            missing_files,
            smart_paste,
        })
    }
}
