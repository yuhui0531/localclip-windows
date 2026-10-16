//! 仅由用户选择的历史对象调用 Windows 文件关联；不拼接命令行。by AI.Coding
use crate::open_target::OpenTarget;
use windows::{
    Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    core::{PCWSTR, w},
};

/// 使用显式 open 动词和空参数，让系统关联程序打开单一对象。
pub fn open(target: &OpenTarget) -> Result<(), &'static str> {
    let value = if target.file {
        let path = target
            .value
            .strip_prefix('/')
            .filter(|p| p.as_bytes().get(1) == Some(&b':'))
            .unwrap_or(&target.value)
            .replace('/', "\\");
        if !std::path::Path::new(&path).is_absolute() || !std::path::Path::new(&path).exists() {
            return Err("filesMissing");
        }
        path
    } else {
        target.value.clone()
    };
    if value.contains('\0') {
        return Err("unsupported");
    }
    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    // ShellExecute 的参数保持为空，文件名中的空格和元字符不是执行参数。
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            PCWSTR(wide.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err("openFailed")
    }
}
