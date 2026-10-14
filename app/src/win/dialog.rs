use localclip_core::export::{PreparedFile, available_name};
use std::{io::Write, os::windows::ffi::OsStrExt, path::PathBuf};
use tauri::Manager;
use windows::{
    Win32::UI::Controls::Dialogs::*,
    core::{PCWSTR, PWSTR},
};
fn choose(
    app: &tauri::AppHandle,
    name: &str,
    extension: &str,
    save: bool,
) -> Result<Option<PathBuf>, &'static str> {
    let mut file = vec![0u16; 32768];
    let initial: Vec<u16> = name.encode_utf16().collect();
    file[..initial.len().min(32767)].copy_from_slice(&initial[..initial.len().min(32767)]);
    let ext: Vec<u16> = extension.encode_utf16().chain([0]).collect();
    let filter: Vec<u16> = format!("{} 文件\0*.{}\0\0", extension.to_uppercase(), extension)
        .encode_utf16()
        .collect();
    let owner = app
        .get_webview_window("main")
        .and_then(|w| w.hwnd().ok())
        .map(|h| windows::Win32::Foundation::HWND(h.0))
        .unwrap_or_default();
    let mut dialog = OPENFILENAMEW {
        lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
        hwndOwner: owner,
        lpstrFile: PWSTR(file.as_mut_ptr()),
        nMaxFile: file.len() as u32,
        lpstrFilter: PCWSTR(filter.as_ptr()),
        lpstrDefExt: PCWSTR(ext.as_ptr()),
        Flags: OFN_EXPLORER
            | OFN_NOCHANGEDIR
            | OFN_PATHMUSTEXIST
            | if save {
                OFN_HIDEREADONLY
            } else {
                OFN_FILEMUSTEXIST
            },
        ..Default::default()
    };
    let success = unsafe {
        if save {
            GetSaveFileNameW(&mut dialog)
        } else {
            GetOpenFileNameW(&mut dialog)
        }
    };
    if !success.as_bool() {
        return if unsafe { CommDlgExtendedError().0 } == 0 {
            Ok(None)
        } else {
            Err("dialogUnavailable")
        };
    }
    let end = file
        .iter()
        .position(|v| *v == 0)
        .ok_or("dialogUnavailable")?;
    Ok(Some(PathBuf::from(
        String::from_utf16(&file[..end]).map_err(|_| "invalidName")?,
    )))
}
pub fn save(app: &tauri::AppHandle, base: &str, file: &PreparedFile) -> Result<bool, &'static str> {
    let suggested = format!("{base}.{}", file.extension);
    let Some(mut path) = choose(app, &suggested, file.extension, true)? else {
        return Ok(false);
    };
    path.set_extension(file.extension);
    let folder = path.parent().ok_or("invalidName")?;
    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .ok_or("invalidName")?;
    let unique =
        available_name(file_name, |name| folder.join(name).exists()).ok_or("invalidName")?;
    let path = folder.join(unique);
    let mut output = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|_| "saveFailed")?;
    if output
        .write_all(&file.bytes)
        .and_then(|_| output.sync_all())
        .is_err()
    {
        drop(output);
        let _ = std::fs::remove_file(path);
        return Err("saveFailed");
    }
    Ok(true)
}
pub fn pick_exe(app: &tauri::AppHandle) -> Result<Option<String>, &'static str> {
    let path = choose(app, "", "exe", false)?;
    path.map(|p| {
        if !p.extension().is_some_and(|v| v.eq_ignore_ascii_case("exe")) {
            return Err("invalidApplication");
        }
        let words: Vec<u16> = p.as_os_str().encode_wide().collect();
        String::from_utf16(&words).map_err(|_| "invalidApplication")
    })
    .transpose()
}
