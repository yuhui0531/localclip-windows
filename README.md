# LocalClip for Windows

A local clipboard history manager for Windows, with a macOS-inspired interface, a Rust core, and a Tauri 2 / React desktop shell. Keep copied text, links, images, and files searchable, then copy, paste, edit, or drag them into another application.

**Current app version: 0.1.14.** Available packages are unsigned manual-test builds. Windows 10 22H2 and Windows 11 are the intended platforms; MSI builds cover x64 and ARM64. Recent native verification used Windows 11 ARM64 in Parallels. Full platform and signed-update acceptance remain open.

## Features

| Feature | What you can do |
| --- | --- |
| Clipboard history | Capture text, rich text, links, images, and file references; search and filter by type, source, date, favorites, or category. |
| Copy and paste | Double-click a card's content to paste into the previously active app. Choose original-format or plain-text output, transform text, combine selections, or use a paste queue. |
| Drag out | Drag text into text editors or export it as a file to file managers; drag images and existing files to compatible targets. The receiving app chooses the offered format. |
| Smart code-file paste | Optionally paste supported code-file contents into recognized editable controls while preserving original file behavior for Copy and file-management targets. |
| Code previews | Syntax highlighting in history cards and detail views, with source text preserved. |
| View and edit | Inspect details and previews in draggable, resizable windows. Edit supported text records in place while retaining their title, favorites, and categories. |
| Local OCR | Copy text from images using Windows OCR and installed Simplified Chinese, Traditional Chinese, or English language packs. |
| Persistent image exports | Keep dragged image files available for document references; inspect storage usage and clear exported images manually. |
| Preferences | Configure hotkeys, appearance, panel size, capture exclusions, retention, storage quota, and login startup. |

## Install

1. Open the [Windows installers workflow](https://github.com/yuhui0531/localclip-windows/actions/workflows/release.yml) and select a successful run for the desired version.
2. Download `LocalClip-manual-arm64` for Windows on ARM, including Apple Silicon Parallels VMs, or `LocalClip-manual-x64` for Intel/AMD Windows. Extract the artifact ZIP.
3. Compare the MSI's SHA-256 with the included `SHA256SUMS-<arch>.txt`, then run the MSI.
4. Launch LocalClip from the Start menu. It starts in the tray; press **Ctrl+Shift+V** to open history.

For example, in PowerShell:

```powershell
Get-FileHash .\LocalClip_0.1.14_arm64.msi -Algorithm SHA256
Get-Content .\SHA256SUMS-arm64.txt
```

The MSI embeds Microsoft's signed WebView2 bootstrapper for machines without the runtime. Installing the runtime may require a Microsoft download. The standalone `localclip-<arch>.exe` artifact requires WebView2 to be installed already.

Automatic update installation is disabled until signing keys are configured. A successful workflow artifact is a manual-test package, not a fully accepted production release. Keep antivirus enabled; an unsigned build or a passed Defender scan does not establish another scanner's verdict.

## Everyday use

- Copy something, open history, and use the search field or filters to find it. The mouse wheel scrolls the horizontal card list; scrollbars hide when idle.
- **Double-click card content** to paste that record into the previous app. **Double-click the title bar** to rename it. Enter and the main action button follow your configured default action.
- Right-click a record for **原格式复制** (copy original), **转换后粘贴…** (transform and paste), **编辑内容** (edit), favorites, categories, **详情** (details), **预览** (preview), **打开** (open), export, and deletion. Entries depend on the record type and selection.
- For OCR, right-click an image and choose **复制图中文字**, or use that button in the image detail/preview window. Recognition runs locally; missing Windows language packs must be installed separately.
- Enable **设置 → 通用 → 代码文件智能粘贴** to paste source from supported files into recognized text inputs. It is off by default, with limits of 4 MiB per file and 8 MiB total. Unrecognized targets retain file behavior; drag-out can offer both source text and the original file.
- Select several cards to combine their contents or start a paste queue. The queue shortcut advances through the selected records.
- The unpinned history panel hides on focus loss. Pin it to keep browsing while switching apps; completing output still hides the panel.

Editing updates the selected record rather than creating a new one. Save persists the new body and refreshes search/output; Cancel discards the draft. Detail and editor windows share title dragging, edge/corner resizing, minimum dimensions, and scrolling content.

### Keyboard shortcuts

Global shortcuts can be changed in Settings. Other shortcuts apply within the history panel or editor.

| Shortcut | Action |
| --- | --- |
| Ctrl+Shift+V | Open history. |
| Ctrl+Shift+Alt+V | Paste the next queue item; registered only while a queue is active. |
| Enter / Shift+Enter | Perform the configured default Copy/Paste action / its alternative. |
| Ctrl+F | Focus search. |
| Left / Right; Shift+Left / Right | Move selection; extend selection. |
| Ctrl+1 … Ctrl+9 | Select one of the first nine displayed records. |
| Ctrl+I / Space | Show details / toggle preview. |
| Ctrl+E | Edit the selected supported text record. |
| Ctrl+O / Ctrl+G | Open the record's target / locate it in history. |
| Ctrl+T | Open text transformations; hold Alt when choosing one to copy only. |
| Ctrl+S / Ctrl+Enter | Save in the editor. |
| Delete / Esc | Delete the selection / close the current view or hide history. |

A paste command can be rejected by the target application or Windows privilege boundaries. `commandSent` means the command was sent, not that the receiving app confirmed receipt.

## Data, privacy, and image cleanup

History and settings are stored locally under:

```text
%LOCALAPPDATA%\LocalClip\History
```

There is no cloud sync or macOS history import. Pause capture, exclude source applications, or hide previews in Settings when needed. Sensitive clipboard markers are checked before reading content; apps that provide no marker cannot be identified reliably as sensitive sources. The shipped app's updater uses this repository's GitHub Releases; development dependencies and the WebView2 installer have separate download requirements.

Dragged image exports live independently under `History\DragImages`, so documents can continue referencing them. In **设置 → 存储**, inspect their size and request cleanup. Cleanup preserves the images in clipboard history, but exported paths used by documents may stop working. Clear them only after those references are no longer needed.

Close LocalClip before upgrading, backing up, or restoring data. Uninstall preserves history/settings by default. The full uninstall UI has an initially unchecked option to delete this Windows user's data; quiet uninstall preserves it unless `REMOVELOCALCLIPHISTORY=1` is explicitly passed. Updater snapshots are retained separately under `%LOCALAPPDATA%\LocalClip-UpdateBackups` and are not automatically deleted.

## Build from source

### Requirements

- Windows with the MSVC C++ build tools and Windows SDK; ARM64 builds also require the MSVC ARM64 tools.
- Rust **1.90.0**, pinned in [rust-toolchain.toml](rust-toolchain.toml).
- Node.js **22** with npm.
- Microsoft Edge WebView2 Runtime for running the desktop app.

From the repository root, in PowerShell:

```powershell
git clone https://github.com/yuhui0531/localclip-windows.git
cd localclip-windows
npm ci --prefix ui --no-audit --no-fund
Push-Location app
& ..\ui\node_modules\.bin\tauri.cmd build --ci --features desktop --bundles msi --no-sign
Pop-Location
```

For an ARM64 build, install `aarch64-pc-windows-msvc` with `rustup target add`, use the matching MSVC tools, and add `--target aarch64-pc-windows-msvc` to the build command. MSI output is under `target/release/bundle/msi`, or `target/<target>/release/bundle/msi` when a target is specified. The [installer workflow](.github/workflows/release.yml) shows the complete x64/ARM64 build and optional signing sequence.

The Windows desktop shell cannot run on macOS. The platform-neutral Rust core and frontend checks can run there. For the repository-local Rust installation, use `sh scripts/cargo-local.sh` instead of `cargo`.

### Checks and generated test data

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p localclip-core --locked
npm --prefix ui run build
node ui/tests/preview.mjs
```

Run Windows adapter tests serially in a disposable test session:

```powershell
cargo test -p localclip-app --locked -- --test-threads=1
cargo test -p localclip-app --features desktop --lib --locked -- --test-threads=1
```

Use a **new** generated-data directory for UI testing:

```powershell
cargo run -p localclip-app --bin ui_fixture -- C:\LocalClip-Test-Generated
.\target\release\localclip.exe --data-dir C:\LocalClip-Test-Generated
```

The fixture contains generated records and starts with capture paused. Close the normal instance first because the application has a single-instance lock. Do not point tests at personal history. Browser interaction harnesses are in [ui/tests](ui/tests); they are separate from native target-app acceptance.

The synthetic service benchmark is available through `cargo run -p localclip-core --release --locked --bin core_benchmark -- reports/my-benchmark`. Its output directory must be new. It records 60 samples per metric and removes its temporary history afterward.

## Repository layout

| Directory | Contents |
| --- | --- |
| [core/](core) | Domain behavior, SQLite storage, search, transforms, and synthetic benchmarks. |
| [app/](app) | Windows clipboard, paste, drag/drop, OCR, updater, installer, and Tauri integration. |
| [ui/](ui) | Bundled React interface, previews, highlighting, and browser checks. |
| [test-vectors/](test-vectors) | Committed macOS behavior fixtures consumed without the reference checkout. |
| [vector-export/](vector-export) | Optional Swift exporter for regenerating reference fixtures. |
| [scripts/](scripts) | Validation, toolchain, privacy-scan, and signing helpers. |
| [changes/](changes) / [reports/](reports) | Scoped implementation notes, acceptance records, and known limitations. |

### macOS behavior reference

The original baseline is LocalClip macOS **0.3.29 (51)** at `c097591431c00982ad34186068064b40965c95fd`. Later Windows parity work includes the [macOS 0.3.38 behavior mapping](changes/active/windows-macos-038/parity.md); intentional fixture changes are recorded in [macos-038-overrides.json](core/tests/macos-038-overrides.json).

Regenerating fixtures requires macOS 15+, Swift 6, and the read-only reference repository, defaulting to `../rea/macos-app` or `LOCALCLIP_MAC_PATH`:

```sh
TZ=Asia/Shanghai swift run --package-path vector-export VectorExport test-vectors
python3 scripts/validate_vectors.py test-vectors
python3 scripts/check_phase0.py
```

The exporter uses synthetic databases and never reads or migrates personal clipboard/history data. Keep its build products and caches inside this repository. Normal builds and CI use the committed fixtures and do not need the reference checkout.

## Signing, updates, and acceptance

Production update installation requires a committed public minisign key in `app/updater-public-key.txt`, plus these CI secrets:

- `WINDOWS_UPDATER_PRIVATE_KEY` and `WINDOWS_UPDATER_KEY_PASSWORD`.
- `WINDOWS_CODESIGN_PFX_BASE64` and `WINDOWS_CODESIGN_PFX_PASSWORD`.

Never commit private keys. The workflow skips unavailable signing credentials and labels unsigned artifacts. A version-matching tag creates a **draft** GitHub release. Update downloads never install automatically: installation requires an explicit user action, signature/architecture/version validation, and a database snapshot. Without the public key, installation requests are rejected.

Recent evidence includes [0.1.14 double-click paste and installation](reports/double-click-0.1.14.json) and [0.1.13 detail/editor windows](reports/dialog-0.1.13.json). Those checks have specific package and environment scope; they do not establish the complete external-app matrix.

Outstanding gates include clean Windows 10/11 lifecycle coverage, the full Word/browser/terminal/chat paste and password-manager/Ditto matrix, mixed-DPI behavior, complete OCR/language-pack acceptance, signed-update/rollback acceptance, and all PRD performance targets. See the [manual acceptance checklist](reports/manual-testing.md) and [Phase 2](reports/phase-2.md), [Phase 3](reports/phase-3.md), and [Phase 4](reports/phase-4.md) reports. Heterogeneous image/text merges are rejected rather than silently dropping formats; HTML-only/RTF-only mixed merges use plain text for the other representation.

## License

[MIT](LICENSE). Bundled dependencies retain their own notices, including [highlight.js](ui/licenses/highlight.js.txt).
