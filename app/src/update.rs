use crate::{desktop::AppState, update_policy as policy};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tauri::{Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

struct Pending {
    update: Update,
    verified_bytes: Vec<u8>,
}
#[derive(Default)]
pub struct State {
    busy: AtomicBool,
    pending: Mutex<Option<Pending>>,
    latest: Mutex<Option<String>>,
    pub menu: Mutex<Option<tauri::menu::MenuItem<tauri::Wry>>>,
}
fn root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .local_data_dir()
        .map_err(|_| "updateUnavailable")?
        .join("LocalClip");
    std::fs::create_dir_all(&root).map_err(|_| "updateUnavailable")?;
    Ok(root)
}
fn document(state: &State) -> Value {
    json!({"current":env!("CARGO_PKG_VERSION"),"latest":state.latest.lock().ok().and_then(|v| v.clone()),
        "downloaded":state.pending.lock().is_ok_and(|v| v.is_some()),"installationEnabled":policy::installation_enabled()})
}
struct Busy<'a>(&'a AtomicBool);
impl Drop for Busy<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

pub async fn check(app: tauri::AppHandle, manual: bool) -> Result<Value, String> {
    let state = app.state::<State>();
    if state.busy.swap(true, Ordering::SeqCst) {
        return Ok(document(&state));
    }
    let _busy = Busy(&state.busy);
    let metadata = root(&app)?.join("last-update-check");
    let now = crate::win::listener::now();
    if !manual {
        if !app
            .state::<AppState>()
            .runtime
            .store
            .settings()
            .map_err(|_| "storageUnavailable")?
            .preferences
            .auto_check_updates
        {
            return Ok(document(&state));
        }
        let last = std::fs::read_to_string(&metadata)
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);
        if now.saturating_sub(last) < 86_400_000 {
            return Ok(document(&state));
        }
    }
    // Write before the request: failures are throttled across process restarts too.
    std::fs::write(metadata, now.to_string()).map_err(|_| "updateUnavailable")?;
    let updater = app
        .updater_builder()
        .endpoints(vec![
            policy::ENDPOINT.parse().map_err(|_| "updateUnavailable")?,
        ])
        .map_err(|_| "updateUnavailable")?
        .pubkey(policy::PUBLIC_KEY.trim())
        .timeout(Duration::from_secs(30))
        .configure_client(|client| client.https_only(true))
        .version_comparator(|current, release| {
            localclip_core::version::is_newer(&release.version.to_string(), &current.to_string())
        })
        .build()
        .map_err(|_| "updateUnavailable")?;
    let Some(update) = updater.check().await.map_err(|_| "updateUnavailable")? else {
        return Ok(document(&state));
    };
    let architecture = if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x64"
    };
    if !policy::allowed_asset(update.download_url.as_str(), architecture)
        || !localclip_core::version::is_newer(&update.version, env!("CARGO_PKG_VERSION"))
    {
        return Err("updateRejected".into());
    }
    *state.latest.lock().map_err(|_| "updateUnavailable")? = Some(update.version.clone());
    if let Ok(menu) = state.menu.lock()
        && let Some(menu) = menu.as_ref()
    {
        let _ = menu.set_text(format!("发现更新 {} · 打开设置", update.version));
    }
    let _ = app.emit("update-available", json!({"version":update.version}));
    if policy::installation_enabled() {
        // Only this verified download can populate pending; frontend supplies no bytes or URL.
        let verified_bytes = update
            .download(|_, _| {}, || {})
            .await
            .map_err(|_| "updateRejected")?;
        validate_msi_architecture(&verified_bytes, architecture)?;
        *state.pending.lock().map_err(|_| "updateUnavailable")? = Some(Pending {
            update,
            verified_bytes,
        });
    }
    if let Ok(latest) = state.latest.lock()
        && let Some(version) = latest.as_ref()
    {
        let _ = app.emit("update-available", json!({"version":version}));
    }
    Ok(document(&state))
}

pub fn install(app: &tauri::AppHandle) -> Result<(), String> {
    if !policy::installation_enabled() {
        return Err("signedUpdatesDisabled".into());
    }
    let state = app.state::<State>();
    if state.busy.swap(true, Ordering::SeqCst) {
        return Err("updateBusy".into());
    }
    let _busy = Busy(&state.busy);
    let mut pending = state.pending.lock().map_err(|_| "updateUnavailable")?;
    let package = pending.as_ref().ok_or("updateNotReady")?;
    if !localclip_core::version::is_newer(&package.update.version, env!("CARGO_PKG_VERSION")) {
        return Err("updateRejected".into());
    }
    let backups = app
        .path()
        .local_data_dir()
        .map_err(|_| "updateUnavailable")?
        .join("LocalClip-UpdateBackups");
    std::fs::create_dir_all(&backups).map_err(|_| "snapshotFailed")?;
    let backup = backups.join(format!(
        "history-{}-{}.sqlite",
        crate::win::listener::now(),
        std::process::id()
    ));
    app.state::<AppState>()
        .runtime
        .store
        .snapshot(backup)
        .map_err(|_| "snapshotFailed")?;
    let marker = root(app)?.join("update-pending.json");
    std::fs::write(
        &marker,
        json!({"version":package.update.version}).to_string(),
    )
    .map_err(|_| "updateUnavailable")?;
    start_watchdog()?;
    // Invoked only by the explicit install command, never by check or normal exit.
    if package.update.install(&package.verified_bytes).is_err() {
        let _ = std::fs::remove_file(marker);
        return Err("updateInstallFailed".into());
    }
    pending.take();
    Ok(())
}
fn start_watchdog() -> Result<(), String> {
    use base64::Engine;
    use std::os::windows::process::CommandExt;
    let script = r#"$ErrorActionPreference='Stop'; Start-Sleep -Seconds 5; $limit=(Get-Date).AddMinutes(15); while ((Get-Process msiexec -ErrorAction SilentlyContinue) -and (Get-Date) -lt $limit) { Start-Sleep -Seconds 2 }; Start-Sleep -Seconds 45; if (Test-Path (Join-Path $env:LOCALAPPDATA 'LocalClip\update-pending.json')) { Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.MessageBox]::Show('LocalClip could not confirm the new version opened its storage. You can reinstall the previous MSI and restore a snapshot from LocalClip-UpdateBackups. Existing snapshots are retained.','LocalClip update') | Out-Null }"#;
    let bytes: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    std::process::Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-EncodedCommand",
            &base64::engine::general_purpose::STANDARD.encode(bytes),
        ])
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|_| "updateUnavailable")?;
    Ok(())
}
pub fn startup(app: &tauri::AppHandle, began: Instant) {
    if let Ok(root) = root(app) {
        let pending = root.join("update-pending.json");
        let expected = std::fs::read(&pending)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok());
        if expected
            .as_ref()
            .is_some_and(|v| v["version"].as_str() == Some(env!("CARGO_PKG_VERSION")))
            && began.elapsed() <= Duration::from_secs(45)
        {
            let _ = std::fs::remove_file(pending);
        }
    }
    let app = app.clone();
    std::thread::spawn(move || {
        loop {
            let _ = tauri::async_runtime::block_on(check(app.clone(), false));
            std::thread::park_timeout(Duration::from_secs(3600));
        }
    });
}

#[tauri::command]
pub async fn update_check(app: tauri::AppHandle, request: Value) -> Result<Value, String> {
    if request["statusOnly"].as_bool() == Some(true) {
        return Ok(document(&app.state::<State>()));
    }
    check(app, true).await
}
#[tauri::command]
pub async fn update_install(app: tauri::AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || install(&app))
        .await
        .map_err(|_| "updateUnavailable")?
}

fn validate_msi_architecture(bytes: &[u8], architecture: &str) -> Result<(), String> {
    use std::{io::Write, os::windows::ffi::OsStrExt};
    use windows::{
        Win32::System::ApplicationInstallationAndServicing::*,
        core::{PCWSTR, PWSTR},
    };
    let path = std::env::temp_dir().join(format!(
        "LocalClip-verify-{}-{}.msi",
        std::process::id(),
        crate::win::listener::now()
    ));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|_| "updateRejected")?;
    struct Temporary(PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
    let _temporary = Temporary(path.clone());
    file.write_all(bytes).map_err(|_| "updateRejected")?;
    drop(file);
    let path: Vec<u16> = path.as_os_str().encode_wide().chain([0]).collect();
    unsafe {
        let mut summary = MSIHANDLE(0);
        if MsiGetSummaryInformationW(MSIHANDLE(0), PCWSTR(path.as_ptr()), 0, &mut summary) != 0 {
            return Err("updateRejected".into());
        }
        let mut text = [0u16; 128];
        let mut length = text.len() as u32;
        let mut kind = 0;
        let mut number = 0;
        let result = MsiSummaryInfoGetPropertyW(
            summary,
            7,
            &mut kind,
            &mut number,
            None,
            Some(PWSTR(text.as_mut_ptr())),
            Some(&mut length),
        );
        MsiCloseHandle(summary);
        let value = String::from_utf16_lossy(&text[..(length as usize).min(text.len())]);
        let expected = if architecture == "arm64" {
            "Arm64"
        } else {
            "x64"
        };
        if result != 0
            || !value
                .split(';')
                .next()
                .is_some_and(|v| v.eq_ignore_ascii_case(expected))
        {
            return Err("updateRejected".into());
        }
    }
    Ok(())
}
