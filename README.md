# LocalClip for Windows

Local, private clipboard history for Windows 10/11, with a Rust core and Tauri v2 shell.
Implementation proceeds through the gated phases in the Windows PRD (2026-10-14 revision).

Behavior baseline: LocalClip macOS **0.3.29 (51)**, `macosCommit` **c097591431c00982ad34186068064b40965c95fd**.
The reference repository is read-only. No macOS history is read, imported, or migrated.

## Structure

- `vector-export/`: Swift executable depending on the reference `LocalClipCore` public library.
- `test-vectors/`: committed behavioral fixtures; CI will consume these without the reference checkout.
- `scripts/`: fixture validation and reproducibility checks.
- `reports/`: phase task lists, evidence, limitations, and PR descriptions.
- `core/`: platform-neutral domain, search, transforms, SQLite store and synthetic benchmarks.
- `app/`: Windows adapters, native IPC, Tauri shell and guarded updater.
- `ui/`: locally bundled React history panel and settings.

## Exporting the behavior baseline

Requires macOS 15+ and Swift 6. The default reference path is `../rea/macos-app`
relative to this repository. Override it with `LOCALCLIP_MAC_PATH`.

```sh
TZ=Asia/Shanghai swift run --package-path vector-export VectorExport test-vectors
python3 scripts/validate_vectors.py test-vectors
python3 scripts/check_phase0.py
```

The exporter checks the reference commit and clean status, creates synthetic databases in
its own temporary directory, removes them on exit, and never accesses the clipboard.
It writes only to the requested output directory, which must be inside this repository.
Build products and compiler caches belong to this repository. Generated UUIDs are mapped
to stable fixture aliases, retaining observed tie ordering explicitly where needed.

`check_phase0.py` runs two exports, checks the complete file bytes and fixture contracts,
and saves raw build output and SHA-256 values in ignored `reports/raw/`. It redirects SwiftPM
caches/configuration/security paths into `vector-export/.build/` for restricted environments.
Export requires access to the macOS system type registry (LaunchServices). A sandbox that
blocks this service makes even PNG/JPEG classification fail; the exporter rejects that
environment rather than publishing misleading macOS differences.

Each JSON document contains `metadata` and `cases`. Metadata identifies source functions,
commit, version, timezone, and any Windows adaptation. Platform-specific changes are explicit;
an exported macOS observation is not evidence of Windows runtime acceptance.

## Core checks

Rust 1.90.0 is pinned by `rust-toolchain.toml`. With a normal Rust installation:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p localclip-core --locked
cargo run -p localclip-core --release --locked --bin core_benchmark -- reports/my-benchmark
```

The benchmark output directory must be new. It creates only synthetic history,
records all 60 samples per metric, and removes its temporary database on completion.
The crash test force-kills 50 child processes using generated data. Run Windows integration tests serially (`cargo test -p localclip-app -- --test-threads=1`).
Core utility executables are separate from the clipboard application.

For the repository-local toolchain, use `sh scripts/cargo-local.sh` in place of
`cargo`. See [Phase 1 evidence](reports/phase-1.md) for measured limits and gates.

## Installing the manual-test build

Use the ARM64 MSI on Windows ARM (including an Apple Silicon Parallels VM), or the
x64 MSI on Intel/AMD Windows. Builds are provided as GitHub Actions artifacts by
**Windows installers**. Compare the downloaded file against its SHA256SUMS file.
The current build is unsigned and has no updater public key; automatic installation
is disabled. It is a manual-test build, not a fully accepted production release.

The MSI uses Microsoft's WebView2 bootstrapper if the runtime is absent. Launch
LocalClip from the Start menu. It starts in the tray; press **Ctrl+Shift+V** to open
history. Copy test text, images or files, then use the panel to search or restore.
The queue's **Ctrl+Shift+Alt+V** shortcut is registered only while a queue is active.
Default output sends a paste command to the saved target; an application can reject
that command. The UI reports `commandSent`, not confirmed receipt.

Data lives in `%LOCALAPPDATA%\LocalClip\History`. There is no macOS import or cloud
sync. Exclude applications or pause capture in settings when needed. Sensitive
clipboard markers are checked before content reads, but applications that supply no
marker cannot be recognized as holding sensitive data.

Uninstall preserves history/settings by default. The full MSI uninstall UI offers
**Also delete this Windows user's history and settings**, initially unchecked.
Quiet uninstall preserves data unless `REMOVELOCALCLIPHISTORY=1` is explicitly
passed. Close LocalClip first. Update snapshots are always retained separately in
`%LOCALAPPDATA%\LocalClip-UpdateBackups`.

## Building on Windows

Install Rust 1.90, the MSVC C++ tools/Windows SDK, Node.js 22 and WebView2, then:

```powershell
npm ci --prefix ui --no-audit --no-fund
cargo test -p localclip-core --locked
cargo test -p localclip-app --locked -- --test-threads=1
cd app
../ui/node_modules/.bin/tauri build --ci --features desktop --bundles msi --no-sign
```

For ARM64, install the MSVC ARM64 tools and Rust target, and add
`--target aarch64-pc-windows-msvc`. Output is under `target/<target>/release/bundle/msi`
(or `target/release/bundle/msi` for the native default target). The CI release job
also provides the standalone executable; it needs WebView2 already installed.

To test without capturing personal clipboard data, generate a new fixture directory
with `cargo run -p localclip-app --bin ui_fixture -- <new-test-directory>`, then launch
`localclip.exe --data-dir <new-test-directory>`. The fixture contains generated
records and starts with capture paused. Do not point tests at your normal history.

## Updates and signing

Only GitHub Releases is used by the app's updater. Automatic checks are enabled by
default and throttled to once per day, including failed attempts. Settings allow a
manual retry. Download never installs; the **Install and restart** button is the
only installation path. Without a committed public key, that path rejects requests.

Before enabling production updates, the owner must commit the public minisign key
to `app/updater-public-key.txt` and configure these CI secrets (names chosen for the
workflow; none are currently supplied): `WINDOWS_UPDATER_PRIVATE_KEY`,
`WINDOWS_UPDATER_KEY_PASSWORD`, `WINDOWS_CODESIGN_PFX_BASE64`,
`WINDOWS_CODESIGN_PFX_PASSWORD`. Never put private keys in the repository.
The workflow skips absent signing credentials and labels unsigned artifacts.
Tags must match the application version; tag runs create a **draft** release.

Verified package bytes remain backend-only. The updater binds signatures to the
announced version, restricts asset URLs to this repository, checks the MSI target
architecture, rejects downgrades, and creates a serialized `VACUUM INTO` snapshot
before install. Startup acknowledgement requires storage to open within 45 seconds;
failed acknowledgement offers manual rollback. Reinstall the old MSI and restore
an appropriate snapshot only with LocalClip stopped; preserve the current data first.
Backups are never automatically deleted.

## Acceptance and known limits

See [Phase 2](reports/phase-2.md), [Phase 3](reports/phase-3.md),
[Phase 4](reports/phase-4.md), and the [manual checklist](reports/manual-testing.md).
The owner explicitly deferred unavailable desktop gates and signed-update acceptance
until environments/keys are available. Build success and synthetic checks do not
prove full target-app receipt, mixed-DPI behavior or clean-install compatibility.

Windows 10 22H2 and clean Win11 lifecycle acceptance, Word/browser/terminal/chat paste
matrix, real password-manager/Ditto coexistence, OCR/language-pack acceptance and
WAC24 remain open. Full PRD performance thresholds remain unaccepted; earlier failed
cold-cache measurements are retained. The available ARM64 VM also exceeded the
hidden-memory target; its generated-data diagnostic is recorded in Phase 4. Heterogeneous image/text merges are rejected
instead of silently losing formats. HTML-only/RTF-only mixed merges use plain text
for the other representation. A pinned panel hides after output per §5.15 and the
pinned macOS behavior, despite WAC22's conflicting visibility wording.
