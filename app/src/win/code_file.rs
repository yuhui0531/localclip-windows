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
    System::Com::*,
    UI::{Accessibility::*, WindowsAndMessaging::*},
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
    automation: IUIAutomation2,
    element: IUIAutomationElement,
    target: Target,
    _apartment: Apartment,
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
            let mut class = [0u16; 256];
            let len = GetClassNameW(HWND(target.window as *mut _), &mut class);
            if String::from_utf16_lossy(&class[..len.max(0) as usize]) == "#32770" {
                return None;
            }
            CoInitializeEx(None, COINIT_MULTITHREADED).ok().ok()?;
            let apartment = Apartment;
            let automation: IUIAutomation2 =
                CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER).ok()?;
            automation.SetConnectionTimeout(200).ok()?;
            automation.SetTransactionTimeout(200).ok()?;
            let element = automation.GetFocusedElement().ok()?;
            let result = Self {
                automation,
                element,
                target,
                _apartment: apartment,
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
                let current = self.automation.GetFocusedElement().ok()?;
                if !self
                    .automation
                    .CompareElements(&current, &self.element)
                    .ok()?
                    .as_bool()
                {
                    return None;
                }
                let kind = current.CurrentControlType().ok()?;
                let value: IUIAutomationValuePattern =
                    current.GetCurrentPatternAs(UIA_ValuePatternId).ok()?;
                Some(
                    [UIA_EditControlTypeId, UIA_DocumentControlTypeId].contains(&kind)
                        && current.CurrentProcessId().ok()? as u32 == self.target.pid
                        && current.CurrentHasKeyboardFocus().ok()?.as_bool()
                        && current.CurrentIsEnabled().ok()?.as_bool()
                        && !current.CurrentIsPassword().ok()?.as_bool()
                        && !value.CurrentIsReadOnly().ok()?.as_bool(),
                )
            }
        })()
        .unwrap_or(false)
    }
}
