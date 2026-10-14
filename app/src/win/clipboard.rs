use super::{formats, source};
use base64::Engine;
use localclip_core::domain::{Item, MIB, Payload, Representation, Settings, Source};
use serde_json::{Value, json};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::mpsc,
    time::Duration,
};
use windows::{
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL, HWND},
        Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom},
        System::{DataExchange::*, Memory::*},
        UI::WindowsAndMessaging::*,
    },
    core::{PCWSTR, w},
};

pub type Result<T> = std::result::Result<T, &'static str>;
const LIMIT: usize = (50 * MIB) as usize;
const BASE64: base64::engine::GeneralPurpose = base64::engine::general_purpose::STANDARD;

#[derive(Clone)]
pub struct Receipt {
    pub sequence: u32,
    pub id: [u8; 16],
    pub fingerprint: String,
}
pub struct Capture {
    pub payload: Payload,
    pub source: Source,
    pub sequence: u32,
}

pub struct MessageWindow(pub HWND);
impl MessageWindow {
    pub fn new() -> Result<Self> {
        unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("LocalClip"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
            .map(Self)
            .map_err(|_| "accessUnavailable")
        }
    }
}
impl Drop for MessageWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.0);
        }
    }
}
struct Open;
impl Drop for Open {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}
fn open(window: HWND) -> Result<Open> {
    for delay in [0, 20, 40, 80, 160] {
        if delay > 0 {
            std::thread::sleep(Duration::from_millis(delay));
        }
        if unsafe { OpenClipboard(Some(window)) }.is_ok() {
            return Ok(Open);
        }
    }
    Err("busy")
}
pub fn registered(name: &str) -> Result<u32> {
    let value: Vec<u16> = name.encode_utf16().chain([0]).collect();
    let id = unsafe { RegisterClipboardFormatW(PCWSTR(value.as_ptr())) };
    if id == 0 {
        Err("accessUnavailable")
    } else {
        Ok(id)
    }
}
fn mapping() -> Result<Vec<(u32, &'static str)>> {
    Ok(vec![
        (13, "text/plain"),
        (registered("UniformResourceLocatorW")?, "text/uri"),
        (registered("HTML Format")?, "text/html"),
        (registered("Rich Text Format")?, "text/rtf"),
        (registered("PNG")?, "image/png"),
        (registered("JFIF")?, "image/jpeg"),
        (17, "image/x-win-dibv5"),
        (15, "text/uri-list"),
    ])
}
fn formats_present() -> Vec<u32> {
    let mut out = Vec::new();
    let mut id = 0;
    loop {
        id = unsafe { EnumClipboardFormats(id) };
        if id == 0 {
            break;
        }
        out.push(id);
    }
    out
}
fn bytes(format: u32, limit: usize, exact: bool) -> Result<Vec<u8>> {
    unsafe {
        let handle = GetClipboardData(format).map_err(|_| "unsupported")?;
        let global = HGLOBAL(handle.0);
        let size = GlobalSize(global);
        if exact && size > limit {
            return Err("tooLarge");
        }
        if size == 0 {
            return Err("unsupported");
        }
        let ptr = GlobalLock(global);
        if ptr.is_null() {
            return Err("accessUnavailable");
        }
        let data = std::slice::from_raw_parts(ptr.cast::<u8>(), size.min(limit)).to_vec();
        let _ = GlobalUnlock(global);
        Ok(data)
    }
}
fn settings_document(settings: &Settings, now: i64) -> Value {
    json!({"capture":settings.capture_enabled,"pauseUntil":settings.pause_until,
        "excluded":settings.excluded_app_ids,"pauseFrontmost":settings.preferences.pause_when_excluded_frontmost,"now":now})
}
fn guarded_read(request: &Value) -> Result<Value> {
    let window = MessageWindow::new()?;
    let _open = open(window.0)?;
    let expected = request["sequence"].as_u64().ok_or("accessUnavailable")? as u32;
    if unsafe { GetClipboardSequenceNumber() } != expected {
        return Err("changedDuringRead");
    }
    let formats = formats_present();
    for name in [
        "ExcludeClipboardContentFromMonitorProcessing",
        "Clipboard Viewer Ignore",
    ] {
        if formats.contains(&registered(name)?) {
            return Err("sensitive");
        }
    }
    for name in ["CanIncludeInClipboardHistory", "CanUploadToCloudClipboard"] {
        let id = registered(name)?;
        if formats.contains(&id) {
            let value = bytes(id, 4, false)?;
            if value.len() != 4
                || u32::from_le_bytes(value.try_into().map_err(|_| "sensitive")?) == 0
            {
                return Err("sensitive");
            }
        }
    }
    let owner = unsafe { GetClipboardOwner() }.ok();
    if owner
        .is_some_and(|w| source::window_pid(w) == request["parentPid"].as_u64().unwrap_or(0) as u32)
    {
        return Err("selfWrite");
    }
    let source = source::clipboard_source();
    let policy = &request["policy"];
    if policy["capture"].as_bool() != Some(true)
        || policy["pauseUntil"]
            .as_i64()
            .is_some_and(|t| t > policy["now"].as_i64().unwrap_or(0))
    {
        return Err("paused");
    }
    let excluded = |id: &str| {
        policy["excluded"]
            .as_array()
            .is_some_and(|v| v.iter().any(|s| s.as_str() == Some(id)))
    };
    if source.basis == "owner" && source.app_id.as_deref().is_some_and(excluded) {
        return Err("excluded");
    }
    if policy["pauseFrontmost"].as_bool() == Some(true)
        && let Some(front) = source::process(source::window_pid(unsafe { GetForegroundWindow() }))
        && excluded(&front.path)
    {
        return Err("paused");
    }
    let private = registered("LocalClip.SelfWrite")?;
    if formats.contains(&private)
        && let Some(receipt) = request["receipt"].as_str()
        && BASE64.encode(bytes(private, 16, false)?) == receipt
    {
        return Err("selfWrite");
    }
    let format = request["format"].as_u64().ok_or("accessUnavailable")? as u32;
    let data = if format == 0 {
        None
    } else {
        if !formats.contains(&format) {
            return Err("changedDuringRead");
        }
        Some(BASE64.encode(bytes(format, LIMIT, true)?))
    };
    if unsafe { GetClipboardSequenceNumber() } != expected {
        return Err("changedDuringRead");
    }
    Ok(
        json!({"status":"ok","formats":formats,"data":data,"source":{"appID":source.app_id,"name":source.name,"basis":source.basis}}),
    )
}

pub fn worker_main() -> Result<()> {
    let request: Value = serde_json::from_reader(std::io::stdin().take(1_048_576))
        .map_err(|_| "accessUnavailable")?;
    let response = match guarded_read(&request) {
        Ok(value) => value,
        Err(status) => json!({"status":status}),
    };
    serde_json::to_writer(std::io::stdout().lock(), &response).map_err(|_| "accessUnavailable")
}

fn worker(request: &Value) -> Result<Value> {
    // GetClipboardData has no cancellable timeout; a disposable message-window process bounds delayed rendering.
    let mut command = Command::new(std::env::current_exe().map_err(|_| "accessUnavailable")?);
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x08000000);
    let mut child = command
        .arg("--clipboard-read")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "accessUnavailable")?;
    let input = serde_json::to_vec(request).map_err(|_| "accessUnavailable")?;
    if child
        .stdin
        .take()
        .ok_or("accessUnavailable")?
        .write_all(&input)
        .is_err()
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err("accessUnavailable");
    }
    let output = child.stdout.take().ok_or("accessUnavailable")?;
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = output
            .take((LIMIT as u64 * 4 / 3) + 1_048_576)
            .read_to_end(&mut bytes);
        let _ = tx.send((result, bytes));
    });
    let response = rx.recv_timeout(Duration::from_millis(500));
    if response.is_err() {
        let _ = child.kill();
    }
    let _ = child.wait();
    let _ = reader.join();
    let (result, bytes) = response.map_err(|_| "formatTimeout")?;
    result.map_err(|_| "accessUnavailable")?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "accessUnavailable")?;
    match value["status"].as_str() {
        Some("ok") => Ok(value),
        Some("sensitive") => Err("sensitive"),
        Some("paused") => Err("paused"),
        Some("excluded") => Err("excluded"),
        Some("selfWrite") => Err("selfWrite"),
        Some("changedDuringRead") => Err("changedDuringRead"),
        Some("tooLarge") => Err("tooLarge"),
        Some("busy") => Err("busy"),
        Some("unsupported") => Err("unsupported"),
        _ => Err("accessUnavailable"),
    }
}

pub fn capture(settings: &Settings, now: i64, receipt: &mut Option<Receipt>) -> Result<Capture> {
    for attempt in 0..2 {
        let sequence = unsafe { GetClipboardSequenceNumber() };
        if receipt.as_ref().is_some_and(|r| r.sequence == sequence) {
            *receipt = None;
            return Err("selfWrite");
        }
        let mut request = json!({"sequence":sequence,"format":0,"parentPid":std::process::id(),
            "policy":settings_document(settings,now),"receipt":receipt.as_ref().map(|r| BASE64.encode(r.id))});
        let mut read = || -> Result<Capture> {
            let metadata = worker(&request)?;
            let source = Source {
                app_id: metadata["source"]["appID"].as_str().map(String::from),
                name: metadata["source"]["name"].as_str().unwrap_or("未知").into(),
                basis: metadata["source"]["basis"]
                    .as_str()
                    .unwrap_or("unknown")
                    .into(),
            };
            let ids = metadata["formats"].as_array().ok_or("accessUnavailable")?;
            let mut item = Item::default();
            let mut paths = None;
            let mut raw_bytes = 0;
            for (id, neutral) in mapping()? {
                if !ids.iter().any(|v| v.as_u64() == Some(id as u64)) {
                    continue;
                }
                request["format"] = json!(id);
                let result = match worker(&request) {
                    Err("formatTimeout" | "unsupported") => continue,
                    other => other?,
                };
                let data = BASE64
                    .decode(result["data"].as_str().ok_or("unsupported")?)
                    .map_err(|_| "unsupported")?;
                raw_bytes += data.len();
                if raw_bytes > LIMIT {
                    return Err("tooLarge");
                }
                let data = match neutral {
                    "text/plain" | "text/uri" => formats::decode_utf16(&data)
                        .ok_or("unsupported")?
                        .into_bytes(),
                    "text/html" => formats::decode_html(&data).ok_or("unsupported")?,
                    "text/uri-list" => {
                        paths = Some(
                            formats::decode_dropfiles_with(&data, |bytes| unsafe {
                                use windows::Win32::Globalization::{
                                    CP_ACP, MB_ERR_INVALID_CHARS, MultiByteToWideChar,
                                };
                                let count =
                                    MultiByteToWideChar(CP_ACP, MB_ERR_INVALID_CHARS, bytes, None);
                                if count <= 0 {
                                    return None;
                                }
                                let mut words = vec![0; count as usize];
                                if MultiByteToWideChar(
                                    CP_ACP,
                                    MB_ERR_INVALID_CHARS,
                                    bytes,
                                    Some(&mut words),
                                ) != count
                                {
                                    return None;
                                }
                                String::from_utf16(&words).ok()
                            })
                            .ok_or("unsupported")?,
                        );
                        continue;
                    }
                    _ => data,
                };
                if neutral == "text/plain" {
                    item.plain_text = String::from_utf8(data.clone()).ok();
                }
                item.declared_types.push(neutral.into());
                item.representations.push(Representation {
                    format: neutral.into(),
                    data,
                });
            }
            let payload = if let Some(paths) = paths {
                Payload {
                    items: paths
                        .into_iter()
                        .map(|path| {
                            Ok(Item {
                                representations: vec![Representation {
                                    format: "text/uri-list".into(),
                                    data: formats::path_to_url(&path)
                                        .ok_or("unsupported")?
                                        .into_bytes(),
                                }],
                                declared_types: vec!["text/uri-list".into()],
                                plain_text: Some(path),
                            })
                        })
                        .collect::<Result<_>>()?,
                }
            } else {
                Payload { items: vec![item] }
            };
            payload.validate()?;
            if unsafe { GetClipboardSequenceNumber() } != sequence {
                return Err("changedDuringRead");
            }
            Ok(Capture {
                payload,
                source,
                sequence,
            })
        };
        match read() {
            Err("formatTimeout") => return Err("accessUnavailable"),
            Err("changedDuringRead") if attempt == 0 => continue,
            Err("selfWrite") => {
                *receipt = None;
                return Err("selfWrite");
            }
            result => return result,
        }
    }
    Err("changedDuringRead")
}

struct Allocation(HGLOBAL);
impl Drop for Allocation {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = GlobalFree(Some(self.0));
            }
        }
    }
}
fn allocate(bytes: &[u8]) -> Result<Allocation> {
    unsafe {
        let memory = Allocation(
            GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes.len().max(1))
                .map_err(|_| "accessUnavailable")?,
        );
        let target = GlobalLock(memory.0);
        if target.is_null() {
            return Err("accessUnavailable");
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), target.cast(), bytes.len());
        let _ = GlobalUnlock(memory.0);
        Ok(memory)
    }
}
pub fn write(window: HWND, payload: &Payload) -> Result<(Receipt, usize)> {
    payload.validate()?;
    let mut blocks = Vec::new();
    let mut missing = 0;
    if payload
        .items
        .iter()
        .all(|i| i.representation("text/uri-list").is_some())
    {
        let mut paths = Vec::new();
        for item in &payload.items {
            let path = formats::url_to_path(
                std::str::from_utf8(item.representation("text/uri-list").ok_or("unsupported")?)
                    .map_err(|_| "unsupported")?,
            )
            .ok_or("unsupported")?;
            if std::path::Path::new(&path).exists() {
                paths.push(path);
            } else {
                missing += 1;
            }
        }
        if paths.is_empty() {
            return Err("missingFiles");
        }
        blocks.push((15, formats::encode_dropfiles(&paths).ok_or("unsupported")?));
        blocks.push((
            registered("Preferred DropEffect")?,
            1u32.to_le_bytes().to_vec(),
        ));
    } else {
        if payload.items.len() != 1 {
            return Err("unsupported");
        }
        for (id, neutral) in mapping()? {
            if let Some(bytes) = payload.items[0].representation(neutral) {
                let data = match neutral {
                    "text/plain" | "text/uri" => formats::encode_utf16(
                        std::str::from_utf8(bytes).map_err(|_| "unsupported")?,
                    ),
                    "text/html" => formats::encode_html(bytes).ok_or("unsupported")?,
                    _ => bytes.to_vec(),
                };
                if neutral == "text/uri" && payload.items[0].representation("text/plain").is_none()
                {
                    blocks.push((13, data.clone()));
                }
                blocks.push((id, data));
            }
        }
    }
    let mut id = [0u8; 16];
    unsafe {
        BCryptGenRandom(None, &mut id, BCRYPT_USE_SYSTEM_PREFERRED_RNG)
            .ok()
            .map_err(|_| "accessUnavailable")?;
    }
    blocks.push((registered("LocalClip.SelfWrite")?, id.to_vec()));
    let mut allocated = blocks
        .iter()
        .map(|(format, data)| Ok((*format, allocate(data)?)))
        .collect::<Result<Vec<_>>>()?;
    {
        let _open = open(window)?;
        unsafe {
            EmptyClipboard().map_err(|_| "accessUnavailable")?;
        }
        for (format, allocation) in &mut allocated {
            unsafe {
                SetClipboardData(*format, Some(HANDLE(allocation.0.0)))
                    .map_err(|_| "accessUnavailable")?;
            }
            allocation.0 = HGLOBAL::default();
        }
    }
    Ok((
        Receipt {
            sequence: unsafe { GetClipboardSequenceNumber() },
            id,
            fingerprint: payload.fingerprint(),
        },
        missing,
    ))
}
