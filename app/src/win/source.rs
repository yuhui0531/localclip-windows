use localclip_core::domain::Source;
use windows::{
    Win32::{
        Foundation::{CloseHandle, FILETIME, HANDLE, HWND, LPARAM},
        Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation},
        Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW},
        System::{DataExchange::GetClipboardOwner, Threading::*},
        UI::WindowsAndMessaging::*,
    },
    core::{BOOL, PCWSTR, PWSTR},
};

pub struct Process {
    pub path: String,
    pub name: String,
    pub created: u64,
    pub elevated: bool,
}

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain([0]).collect()
}

pub fn process(pid: u32) -> Option<Process> {
    unsafe {
        let handle = Handle(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?);
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        QueryFullProcessImageNameW(
            handle.0,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
        .ok()?;
        let path = String::from_utf16(&buffer[..length as usize]).ok()?;
        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        GetProcessTimes(handle.0, &mut created, &mut exited, &mut kernel, &mut user).ok()?;
        let mut token = HANDLE::default();
        OpenProcessToken(handle.0, TOKEN_QUERY, &mut token).ok()?;
        let token = Handle(token);
        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0;
        GetTokenInformation(
            token.0,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .ok()?;
        let name =
            description(&path).unwrap_or_else(|| path.rsplit('\\').next().unwrap_or("未知").into());
        Some(Process {
            path: path.to_lowercase(),
            name,
            created: ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64,
            elevated: elevation.TokenIsElevated != 0,
        })
    }
}

fn description(path: &str) -> Option<String> {
    unsafe {
        let filename = wide(path);
        let size = GetFileVersionInfoSizeW(PCWSTR(filename.as_ptr()), None);
        if size == 0 || size > 4 * 1024 * 1024 {
            return None;
        }
        let mut data = vec![0u8; size as usize];
        GetFileVersionInfoW(
            PCWSTR(filename.as_ptr()),
            None,
            size,
            data.as_mut_ptr().cast(),
        )
        .ok()?;
        let mut pointer = std::ptr::null_mut();
        let mut length = 0;
        let key = wide(r"\VarFileInfo\Translation");
        if !VerQueryValueW(
            data.as_ptr().cast(),
            PCWSTR(key.as_ptr()),
            &mut pointer,
            &mut length,
        )
        .as_bool()
            || length < 4
            || pointer.is_null()
        {
            return None;
        }
        let translation = std::slice::from_raw_parts(pointer.cast::<u16>(), (length / 2) as usize);
        for pair in translation.chunks_exact(2) {
            let key = wide(&format!(
                r"\StringFileInfo\{:04x}{:04x}\FileDescription",
                pair[0], pair[1]
            ));
            let mut text = std::ptr::null_mut();
            let mut count = 0;
            if VerQueryValueW(
                data.as_ptr().cast(),
                PCWSTR(key.as_ptr()),
                &mut text,
                &mut count,
            )
            .as_bool()
                && !text.is_null()
                && count > 1
            {
                let words = std::slice::from_raw_parts(text.cast::<u16>(), count as usize - 1);
                if let Ok(value) = String::from_utf16(words)
                    && !value.trim().is_empty()
                {
                    return Some(value);
                }
            }
        }
        None
    }
}

pub fn window_pid(window: HWND) -> u32 {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(window, Some(&mut pid));
    }
    pid
}

pub fn clipboard_source() -> Source {
    unsafe {
        let (window, mut basis) = match GetClipboardOwner() {
            Ok(window) if !window.is_invalid() => (window, "owner"),
            _ => (GetForegroundWindow(), "foreground"),
        };
        let Some(mut owner) = process(window_pid(window)) else {
            return Source::default();
        };
        if owner.path.ends_with("\\applicationframehost.exe") {
            struct Children {
                owner: u32,
                child: Option<Process>,
            }
            unsafe extern "system" fn child(window: HWND, context: LPARAM) -> BOOL {
                unsafe {
                    let context = &mut *(context.0 as *mut Children);
                    let pid = window_pid(window);
                    if pid != context.owner
                        && let Some(candidate) = process(pid)
                        && !candidate.path.ends_with("\\applicationframehost.exe")
                    {
                        context.child = Some(candidate);
                        return BOOL(0);
                    }
                    BOOL(1)
                }
            }
            let mut children = Children {
                owner: window_pid(window),
                child: None,
            };
            let _ = EnumChildWindows(
                Some(window),
                Some(child),
                LPARAM((&mut children as *mut Children) as isize),
            );
            if let Some(child) = children.child {
                owner = child;
            } else {
                basis = "foreground";
            }
        }
        Source {
            app_id: Some(owner.path),
            name: owner.name,
            basis: basis.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target {
    pub window: isize,
    pub pid: u32,
    pub created: u64,
    pub focus: isize,
}
impl Target {
    pub fn capture(window: HWND) -> Option<Self> {
        unsafe {
            if window.is_invalid() || !IsWindow(Some(window)).as_bool() {
                return None;
            }
            let pid = window_pid(window);
            if pid == std::process::id() {
                return None;
            }
            let process = process(pid)?;
            let mut info = GUITHREADINFO {
                cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
                ..Default::default()
            };
            // 初始焦点不可读仍保存应用身份，实际发送前重新固定焦点。by AI.Coding
            let _ = GetGUIThreadInfo(GetWindowThreadProcessId(window, None), &mut info);
            Some(Self {
                window: window.0 as isize,
                pid,
                created: process.created,
                focus: info.hwndFocus.0 as isize,
            })
        }
    }
    pub fn validate(&self) -> Option<Process> {
        unsafe {
            let window = HWND(self.window as *mut _);
            if !IsWindow(Some(window)).as_bool() || window_pid(window) != self.pid {
                return None;
            }
            let process = process(self.pid)?;
            (process.created == self.created).then_some(process)
        }
    }
}

pub fn running_applications() -> Vec<(String, String)> {
    unsafe extern "system" fn collect(window: HWND, parameter: LPARAM) -> BOOL {
        unsafe {
            if IsWindowVisible(window).as_bool() {
                let mut pid = 0;
                GetWindowThreadProcessId(window, Some(&mut pid));
                if pid != std::process::id() {
                    (&mut *(parameter.0 as *mut std::collections::BTreeSet<u32>)).insert(pid);
                }
            }
        }
        BOOL(1)
    }
    let mut ids = std::collections::BTreeSet::new();
    unsafe {
        let _ = EnumWindows(Some(collect), LPARAM(&mut ids as *mut _ as isize));
    }
    let applications: std::collections::BTreeMap<String, String> = ids
        .into_iter()
        .filter_map(process)
        .map(|p| (p.path, p.name))
        .collect();
    applications.into_iter().collect()
}
