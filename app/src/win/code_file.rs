//! Windows 智能代码文件粘贴：锁定本地文件，并只接受可确认的可编辑控件。by AI.Coding
use super::{clipboard::Result, formats, source::Target};
use crate::code_file::{PER_FILE_LIMIT, TOTAL_LIMIT, decode, supported};
use localclip_core::domain::Payload;
use std::{
    fs::{File, OpenOptions},
    io::Read,
    os::windows::{
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawHandle,
    },
    path::PathBuf,
};
use windows::Win32::{
    Foundation::{HANDLE, HWND},
    Storage::FileSystem::*,
    System::{Com::*, Variant::*},
    UI::{Accessibility::*, Input::KeyboardAndMouse::IsWindowEnabled, WindowsAndMessaging::*},
};

struct LockedFile {
    file: File,
    path: PathBuf,
    identity: (u32, u64, u64, u64),
}
pub struct Files {
    files: Vec<LockedFile>,
}
fn identity(file: &File) -> Result<(u32, u64, u64, u64)> {
    let mut info = BY_HANDLE_FILE_INFORMATION::default();
    unsafe {
        GetFileInformationByHandle(HANDLE(file.as_raw_handle()), &mut info)
            .map_err(|_| "codeFileUnavailable")?;
    }
    Ok((
        info.dwVolumeSerialNumber,
        (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
        (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow),
        (u64::from(info.ftLastWriteTime.dwHighDateTime) << 32)
            | u64::from(info.ftLastWriteTime.dwLowDateTime),
    ))
}
fn open(path: &PathBuf) -> Result<File> {
    // 不共享写入/删除；从打开直到输出完成，其他进程无法改写或替换原文件。
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ.0)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0)
        .open(path)
        .map_err(|_| "codeFileUnavailable")?;
    let meta = file.metadata().map_err(|_| "codeFileUnavailable")?;
    if !meta.is_file() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
        return Err("codeFileUnavailable");
    }
    Ok(file)
}
impl Files {
    /// 只做有界元数据检查；UNC、设备路径、符号链接及任一不支持项目均整组回退。
    pub fn open(payload: &Payload) -> Result<Self> {
        if payload.items.is_empty() || payload.items.len() > 64 {
            return Err("codeFileUnsupported");
        }
        let mut files = Vec::new();
        let mut total = 0;
        for item in &payload.items {
            let url = std::str::from_utf8(
                item.representation("text/uri-list")
                    .ok_or("codeFileUnsupported")?,
            )
            .map_err(|_| "codeFileUnsupported")?;
            let path = formats::url_to_path(url).ok_or("codeFileUnsupported")?;
            let bytes = path.as_bytes();
            if bytes.len() < 3
                || !bytes[0].is_ascii_alphabetic()
                || &bytes[1..3] != b":\\"
                || path[2..].contains(':')
            {
                return Err("codeFileUnsupported");
            }
            let path = PathBuf::from(path);
            if !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(supported)
            {
                return Err("codeFileUnsupported");
            }
            let file = open(&path)?;
            let identity = identity(&file)?;
            total += identity.2;
            if identity.2 > PER_FILE_LIMIT as u64 || total > TOTAL_LIMIT as u64 {
                return Err("codeFileTooLarge");
            }
            files.push(LockedFile {
                file,
                path,
                identity,
            });
        }
        Ok(Self { files })
    }
    pub fn is_current(&self) -> bool {
        self.files.iter().all(|entry| {
            identity(&entry.file) == Ok(entry.identity)
                && open(&entry.path).and_then(|f| identity(&f)) == Ok(entry.identity)
        })
    }
    /// 读取过程中保持所有句柄，整组成功才提供正文；失败时不产生部分文本。
    pub fn read(&mut self) -> Result<String> {
        let mut texts = Vec::new();
        let mut total = 0;
        for entry in &mut self.files {
            let mut bytes = Vec::new();
            (&mut entry.file)
                .take((PER_FILE_LIMIT + 1) as u64)
                .read_to_end(&mut bytes)
                .map_err(|_| "codeFileUnavailable")?;
            total += bytes.len();
            if bytes.len() > PER_FILE_LIMIT || total > TOTAL_LIMIT {
                return Err("codeFileTooLarge");
            }
            if bytes.len() as u64 != entry.identity.2 {
                return Err("codeFileChanged");
            }
            texts.push(decode(&bytes)?);
        }
        if !self.is_current() {
            return Err("codeFileChanged");
        }
        Ok(texts.join("\n"))
    }
}

struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
pub struct TextTarget {
    automation: Option<(IUIAutomation2, IUIAutomationElement)>,
    target: Target,
    _apartment: Option<Apartment>,
}
fn class_name(window: HWND) -> String {
    let mut class = [0u16; 256];
    let len = unsafe { GetClassNameW(window, &mut class) };
    String::from_utf16_lossy(&class[..len.max(0) as usize]).to_lowercase()
}
/// 原生 Edit/RichEdit 的窗口样式可确认可写性，不依赖 UIA 代理是否可用。
fn native_edit(target: Target) -> Option<bool> {
    unsafe {
        let window = HWND(target.focus as *mut _);
        let class = class_name(window);
        if !(class == "edit"
            || class.starts_with("richedit")
            || class.starts_with("windowsforms10.edit."))
        {
            return None;
        }
        let style = GetWindowLongPtrW(window, GWL_STYLE);
        Some(
            GetAncestor(window, GA_ROOT).0 as isize == target.window
                && IsWindowEnabled(window).as_bool()
                && style & (0x0800 | 0x0020) == 0,
        ) // ES_READONLY | ES_PASSWORD
    }
}
/// 多行控件通常只提供 TextPattern；仅取只读属性，不读取目标正文。
unsafe fn writable(element: &IUIAutomationElement) -> Option<bool> {
    unsafe {
        if let Ok(value) =
            element.GetCurrentPatternAs::<IUIAutomationValuePattern>(UIA_ValuePatternId)
        {
            return Some(!value.CurrentIsReadOnly().ok()?.as_bool());
        }
        let pattern: IUIAutomationTextPattern =
            element.GetCurrentPatternAs(UIA_TextPatternId).ok()?;
        let mut attribute = pattern
            .DocumentRange()
            .ok()?
            .GetAttributeValue(UIA_IsReadOnlyAttributeId)
            .ok()?;
        let result = (attribute.Anonymous.Anonymous.vt == VT_BOOL)
            .then(|| attribute.Anonymous.Anonymous.Anonymous.boolVal.0 == 0);
        let _ = VariantClear(&mut attribute);
        result
    }
}
impl TextTarget {
    /// 不读取目标正文；只查询角色、焦点、密码及只读标记。未知目标保留文件。
    pub fn capture(target: Target) -> Option<Self> {
        let process = target.validate()?;
        if [
            "explorer.exe",
            "totalcmd.exe",
            "totalcmd64.exe",
            "doublecmd.exe",
            "dopus.exe",
        ]
        .iter()
        .any(|name| process.path.rsplit('\\').next() == Some(name))
        {
            return None;
        }
        unsafe {
            if class_name(HWND(target.window as *mut _)) == "#32770" {
                return None;
            }
            if let Some(editable) = native_edit(target) {
                let result = Self {
                    automation: None,
                    target,
                    _apartment: None,
                };
                return (editable && result.is_current()).then_some(result);
            }
            CoInitializeEx(None, COINIT_MULTITHREADED).ok().ok()?;
            let apartment = Apartment;
            let automation: IUIAutomation2 =
                CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER).ok()?;
            automation.SetConnectionTimeout(200).ok()?;
            automation.SetTransactionTimeout(200).ok()?;
            let element = automation.GetFocusedElement().ok()?;
            let result = Self {
                automation: Some((automation, element)),
                target,
                _apartment: Some(apartment),
            };
            result.is_current().then_some(result)
        }
    }
    pub fn is_current(&self) -> bool {
        (|| -> Option<bool> {
            unsafe {
                if Target::capture(GetForegroundWindow())? != self.target {
                    return None;
                }
                let Some((automation, element)) = &self.automation else {
                    return native_edit(self.target);
                };
                let current = automation.GetFocusedElement().ok()?;
                if !automation
                    .CompareElements(&current, element)
                    .ok()?
                    .as_bool()
                {
                    return None;
                }
                let kind = current.CurrentControlType().ok()?;
                Some(
                    [UIA_EditControlTypeId, UIA_DocumentControlTypeId].contains(&kind)
                        && current.CurrentProcessId().ok()? as u32 == self.target.pid
                        && current.CurrentHasKeyboardFocus().ok()?.as_bool()
                        && current.CurrentIsEnabled().ok()?.as_bool()
                        && !current.CurrentIsPassword().ok()?.as_bool()
                        && writable(&current) == Some(true),
                )
            }
        })()
        .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::w;
    #[test]
    fn native_text_controls_reject_readonly_password_and_nontext() {
        unsafe {
            let root = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("Generated test"),
                WINDOW_STYLE(0),
                0,
                0,
                300,
                100,
                None,
                None,
                None,
                None,
            )
            .unwrap();
            let edit = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("EDIT"),
                w!(""),
                WS_CHILD,
                0,
                0,
                200,
                80,
                Some(root),
                None,
                None,
                None,
            )
            .unwrap();
            let target = Target {
                window: root.0 as isize,
                focus: edit.0 as isize,
                pid: 0,
                created: 0,
            };
            assert_eq!(native_edit(target), Some(true));
            for style in [0x0800, 0x0020, WS_DISABLED.0] {
                SetWindowLongPtrW(edit, GWL_STYLE, (WS_CHILD.0 | style) as isize);
                assert_eq!(native_edit(target), Some(false));
            }
            assert_eq!(
                native_edit(Target {
                    focus: root.0 as isize,
                    ..target
                }),
                None
            );
            DestroyWindow(root).unwrap();
        }
    }
}
