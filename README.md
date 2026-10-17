# LocalClip for Windows

[简体中文](#简体中文) | [English](#english)

## 简体中文

面向 Windows 的本地剪贴板历史管理工具，采用接近 macOS 的界面，由 Rust 核心和 Tauri 2 / React 桌面界面组成。复制过的文本、链接、图片和文件可以随时搜索，再复制、粘贴、编辑或拖到其他应用中。

**当前应用版本：0.1.14。** 目前提供未签名的手动测试安装包。目标平台为 Windows 10 22H2 和 Windows 11，提供 x64、ARM64 两种 MSI。近期原生验证环境为 Parallels 中的 Windows 11 ARM64；完整平台验收和签名更新验收尚未完成。

### 功能

| 功能 | 使用方式 |
| --- | --- |
| 剪贴板历史 | 采集文本、富文本、链接、图片和文件引用；按类型、来源、日期、收藏或分类搜索和筛选。 |
| 复制与粘贴 | 双击卡片正文，粘贴到先前活跃的应用；支持原格式、纯文本、文本转换、多项合并和粘贴队列。 |
| 拖出内容 | 将文本拖入编辑器，或向文件管理器导出为文件；将图片和已有文件拖到兼容应用。接收应用决定采用哪种格式。 |
| 代码文件智能粘贴 | 可选地向可识别的文本输入控件粘贴受支持代码文件的源码；复制操作和文件管理目标仍保留原文件行为。 |
| 代码预览 | 历史卡片与详情支持语法高亮，预览不改变源码正文。 |
| 查看与编辑 | 详情、预览窗口支持拖动和缩放；支持原位编辑的文本记录可保留标题、收藏和分类。 |
| 本地 OCR | 使用 Windows OCR 和已安装的简体中文、繁体中文或英文语言包，复制图片中的文字。 |
| 图片长期缓存 | 拖出的图片文件可长期供文档引用；支持查看占用和手动清理导出副本。 |
| 设置 | 配置快捷键、外观、面板大小、采集排除、保留期限、容量上限和登录启动。 |

### 安装

1. 打开 [Windows installers 工作流](https://github.com/yuhui0531/localclip-windows/actions/workflows/release.yml)，选择所需版本的成功构建。
2. Windows on ARM（包括 Apple Silicon 上的 Parallels 虚拟机）下载 `LocalClip-manual-arm64`；Intel/AMD Windows 下载 `LocalClip-manual-x64`。解压下载的 ZIP。
3. 对照附带的 `SHA256SUMS-<arch>.txt` 校验 MSI 的 SHA-256，然后运行 MSI。
4. 从开始菜单启动 LocalClip。应用在托盘运行，按 **Ctrl+Shift+V** 打开历史面板。

PowerShell 校验示例：

```powershell
Get-FileHash .\LocalClip_0.1.14_arm64.msi -Algorithm SHA256
Get-Content .\SHA256SUMS-arm64.txt
```

如果系统没有 WebView2 运行时，MSI 会使用内嵌的微软签名引导安装程序；安装运行时可能需要从微软下载。单独下载的 `localclip-<arch>.exe` 要求系统已安装 WebView2。

签名密钥尚未配置，自动更新安装暂不可用。工作流产物属于手动测试包，不代表已经完成正式发布验收。请保持杀毒软件开启；未签名或通过 Defender 扫描，都不能代替其他杀毒软件的实际检测结论。

### 日常使用

- 复制内容后打开历史，通过搜索框或筛选条件查找。鼠标滚轮可以滚动横向卡片列表，滚动条空闲时隐藏。
- **双击卡片正文**，将该条记录粘贴到先前的应用；**双击标题栏**可重命名。Enter 和主操作按钮遵循设置中的默认动作。
- 右键菜单提供 **原格式复制**、**转换后粘贴…**、**编辑内容**、收藏、分类、**详情**、**预览**、**打开**、导出和删除；实际项目取决于记录类型与所选数量。
- 识别图片文字时，右键图片选择 **复制图中文字**，或点击图片详情/预览中的同名按钮。识别在本机完成；缺失的 Windows 语言包需另行安装。
- 在 **设置 → 通用 → 代码文件智能粘贴** 中开启源码粘贴。默认关闭，单文件上限为 4 MiB，合计上限为 8 MiB；仅对可识别的文本输入目标生效。其他目标保留文件行为；拖出时可同时提供源码正文和原文件。
- 选择多条卡片，可合并内容或启动粘贴队列，使用队列快捷键依次粘贴。
- 未置顶的历史面板失焦后隐藏；置顶后可切换应用继续浏览。完成输出后，面板仍会隐藏。

编辑会更新所选记录，而不是新增一条。保存后持久化正文并刷新搜索与输出；取消会放弃草稿。详情与编辑窗口统一支持标题拖动、边缘/角落缩放、最小尺寸限制和内容滚动。

#### 快捷键

全局快捷键可在设置中修改，其余快捷键在历史面板或编辑窗口内生效。

| 快捷键 | 操作 |
| --- | --- |
| Ctrl+Shift+V | 打开历史面板。 |
| Ctrl+Shift+Alt+V | 粘贴队列下一项；仅在队列进行时注册。 |
| Enter / Shift+Enter | 执行默认复制/粘贴动作 / 另一种动作。 |
| Ctrl+F | 聚焦搜索框。 |
| 左 / 右方向键；Shift+左 / 右 | 移动选择；扩展选择。 |
| Ctrl+1 … Ctrl+9 | 选择当前显示的前九条记录之一。 |
| Ctrl+I / Space | 查看详情 / 切换预览。 |
| Ctrl+E | 编辑所选且支持编辑的文本记录。 |
| Ctrl+O / Ctrl+G | 打开记录对应的目标 / 在历史中定位。 |
| Ctrl+T | 打开文本转换；选择转换项时按住 Alt 仅复制。 |
| Ctrl+S / Ctrl+Enter | 在编辑窗口保存。 |
| Delete / Esc | 删除所选记录 / 关闭当前视图或隐藏历史面板。 |

目标应用或 Windows 权限边界可能拒绝粘贴。`commandSent` 仅表示粘贴命令已发送，不表示接收应用已确认内容写入。

### 数据、隐私与图片缓存清理

历史和设置保存在本机：

```text
%LOCALAPPDATA%\LocalClip\History
```

不提供云同步或 macOS 历史导入。可按需在设置中暂停采集、排除来源应用或隐藏预览。读取内容前会检查剪贴板敏感标记；没有提供标记的应用，无法被可靠识别为敏感来源。应用更新器使用本仓库的 GitHub Releases；开发依赖和 WebView2 安装程序有各自的下载需求。

拖出图片的导出副本独立保存在 `History\DragImages`，供文档持续引用。在 **设置 → 存储** 中可查看占用并清理。清理会保留剪贴板历史中的原图，但使用导出路径的文档可能无法继续显示图片；确认不再需要这些引用后再清理。

升级、备份或恢复数据前请先退出 LocalClip。卸载默认保留历史与设置；完整卸载界面中的删除当前 Windows 用户数据选项默认不勾选。静默卸载也会保留数据，除非明确传入 `REMOVELOCALCLIPHISTORY=1`。更新器快照另外保存在 `%LOCALAPPDATA%\LocalClip-UpdateBackups`，不会自动删除。

### 从源码构建

#### 环境要求

- Windows、MSVC C++ 构建工具和 Windows SDK；ARM64 构建还需要 MSVC ARM64 工具。
- Rust **1.90.0**，由 [rust-toolchain.toml](rust-toolchain.toml) 固定版本。
- Node.js **22** 和 npm。
- 运行桌面应用需要 Microsoft Edge WebView2 Runtime。

在 PowerShell 中执行：

```powershell
git clone https://github.com/yuhui0531/localclip-windows.git
cd localclip-windows
npm ci --prefix ui --no-audit --no-fund
Push-Location app
& ..\ui\node_modules\.bin\tauri.cmd build --ci --features desktop --bundles msi --no-sign
Pop-Location
```

构建 ARM64 时，使用 `rustup target add` 安装 `aarch64-pc-windows-msvc`，选择匹配的 MSVC 工具，并在构建命令中加入 `--target aarch64-pc-windows-msvc`。默认 MSI 输出目录为 `target/release/bundle/msi`；指定目标后为 `target/<target>/release/bundle/msi`。完整的 x64/ARM64 构建及可选签名流程见 [安装包工作流](.github/workflows/release.yml)。

Windows 桌面程序不能在 macOS 上运行；平台无关的 Rust 核心和前端检查可以运行。使用仓库内的 Rust 工具链时，以 `sh scripts/cargo-local.sh` 代替 `cargo`。

#### 检查与生成测试数据

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test -p localclip-core --locked
npm --prefix ui run build
node ui/tests/preview.mjs
```

Windows 适配层测试请在可丢弃的测试会话中串行执行：

```powershell
cargo test -p localclip-app --locked -- --test-threads=1
cargo test -p localclip-app --features desktop --lib --locked -- --test-threads=1
```

UI 测试请使用**全新**的生成数据目录：

```powershell
cargo run -p localclip-app --bin ui_fixture -- C:\LocalClip-Test-Generated
.\target\release\localclip.exe --data-dir C:\LocalClip-Test-Generated
```

测试数据包含生成的记录，默认暂停采集。应用有单实例锁，测试前应先退出正常运行的实例。不要将测试指向个人历史目录。浏览器交互测试位于 [ui/tests](ui/tests)，它们与原生目标应用验收属于不同层面的验证。

合成数据性能测试命令为 `cargo run -p localclip-core --release --locked --bin core_benchmark -- reports/my-benchmark`，输出目录必须尚不存在。每项指标记录 60 个样本，完成后删除临时历史数据。

### 仓库结构

| 目录 | 内容 |
| --- | --- |
| [core/](core) | 领域行为、SQLite 存储、搜索、文本转换和合成数据性能测试。 |
| [app/](app) | Windows 剪贴板、粘贴、拖放、OCR、更新器、安装器和 Tauri 集成。 |
| [ui/](ui) | 本地打包的 React 界面、预览、语法高亮和浏览器测试。 |
| [test-vectors/](test-vectors) | 已提交的 macOS 行为测试向量，无需参考仓库即可使用。 |
| [vector-export/](vector-export) | 可选的 Swift 参考行为导出工具。 |
| [scripts/](scripts) | 校验、工具链、隐私扫描和签名辅助脚本。 |
| [changes/](changes) / [reports/](reports) | 分项实现说明、验收记录和已知限制。 |

#### macOS 行为参考

初始基线为 LocalClip macOS **0.3.29 (51)**，提交 `c097591431c00982ad34186068064b40965c95fd`。后续 Windows 对齐工作包含 [macOS 0.3.38 行为映射](changes/active/windows-macos-038/parity.md)，有意调整的向量预期记录在 [macos-038-overrides.json](core/tests/macos-038-overrides.json)。

重新生成测试向量需要 macOS 15+、Swift 6 和只读参考仓库；默认路径为 `../rea/macos-app`，也可通过 `LOCALCLIP_MAC_PATH` 指定：

```sh
TZ=Asia/Shanghai swift run --package-path vector-export VectorExport test-vectors
python3 scripts/validate_vectors.py test-vectors
python3 scripts/check_phase0.py
```

导出工具只使用合成数据库，不读取或迁移个人剪贴板及历史。其构建产物与缓存应保留在本仓库内。正常构建和 CI 使用已提交的测试向量，不依赖参考仓库。

### 签名、更新与验收

#### Code signing policy / 代码签名政策

[中英文签名政策](CODE_SIGNING_POLICY.md) · [隐私政策](PRIVACY.md) · [SignPath 申请材料与条件核查](docs/signpath-application.md)。正在准备 SignPath Foundation 免费开源签名申请，尚未提交、获批或接入；当前安装包仍未签名。未来服务实际启用后，鸣谢措辞为“Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org)”；目前不代表已获赞助。

正式更新安装需要在 `app/updater-public-key.txt` 提交 minisign 公钥，并配置以下 CI secrets：

- `WINDOWS_UPDATER_PRIVATE_KEY`、`WINDOWS_UPDATER_KEY_PASSWORD`。
- 当前 PFX 签名路径：`WINDOWS_CODESIGN_PFX_BASE64`、`WINDOWS_CODESIGN_PFX_PASSWORD`；SignPath 获批后需要另行接入云签名，不能仅填入这些 PFX secrets。

不要将私钥提交到仓库。没有签名凭据时，工作流跳过相应签名并标记未签名产物。与应用版本一致的标签会创建**草稿** GitHub Release。下载更新不会自动安装；安装必须由用户明确触发，并经过签名、架构、版本校验和数据库快照。缺少公钥时，安装请求会被拒绝。

近期验证记录包括 [0.1.14 双击粘贴与安装](reports/double-click-0.1.14.json) 和 [0.1.13 详情/编辑窗口](reports/dialog-0.1.13.json)。记录仅证明对应安装包和环境中的已执行检查，不代表完整的外部应用兼容矩阵已经通过。

尚待完成的验收包括：干净 Windows 10/11 环境中的安装卸载全流程，完整的 Word/浏览器/终端/聊天应用粘贴矩阵和密码管理器/Ditto 共存，混合 DPI，完整 OCR/语言包验收，签名更新与回滚验收，以及全部 PRD 性能目标。详见 [手动验收清单](reports/manual-testing.md) 和 [阶段 2](reports/phase-2.md)、[阶段 3](reports/phase-3.md)、[阶段 4](reports/phase-4.md) 报告。图片与文本等异构内容合并会被拒绝，以免静默丢失格式；仅含 HTML/RTF 的混合合并会为另一种表示使用纯文本。

### 许可证

[MIT](LICENSE)。随应用打包的依赖保留各自声明，包括 [highlight.js](ui/licenses/highlight.js.txt)。

---

## English

A local clipboard history manager for Windows, with a macOS-inspired interface, a Rust core, and a Tauri 2 / React desktop shell. Keep copied text, links, images, and files searchable, then copy, paste, edit, or drag them into another application.

**Current app version: 0.1.14.** Available packages are unsigned manual-test builds. Windows 10 22H2 and Windows 11 are the intended platforms; MSI builds cover x64 and ARM64. Recent native verification used Windows 11 ARM64 in Parallels. Full platform and signed-update acceptance remain open.

### Features

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

### Install

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

### Everyday use

- Copy something, open history, and use the search field or filters to find it. The mouse wheel scrolls the horizontal card list; scrollbars hide when idle.
- **Double-click card content** to paste that record into the previous app. **Double-click the title bar** to rename it. Enter and the main action button follow your configured default action.
- Right-click a record for **原格式复制** (copy original), **转换后粘贴…** (transform and paste), **编辑内容** (edit), favorites, categories, **详情** (details), **预览** (preview), **打开** (open), export, and deletion. Entries depend on the record type and selection.
- For OCR, right-click an image and choose **复制图中文字**, or use that button in the image detail/preview window. Recognition runs locally; missing Windows language packs must be installed separately.
- Enable **设置 → 通用 → 代码文件智能粘贴** to paste source from supported files into recognized text inputs. It is off by default, with limits of 4 MiB per file and 8 MiB total. Unrecognized targets retain file behavior; drag-out can offer both source text and the original file.
- Select several cards to combine their contents or start a paste queue. The queue shortcut advances through the selected records.
- The unpinned history panel hides on focus loss. Pin it to keep browsing while switching apps; completing output still hides the panel.

Editing updates the selected record rather than creating a new one. Save persists the new body and refreshes search/output; Cancel discards the draft. Detail and editor windows share title dragging, edge/corner resizing, minimum dimensions, and scrolling content.

#### Keyboard shortcuts

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

### Data, privacy, and image cleanup

History and settings are stored locally under:

```text
%LOCALAPPDATA%\LocalClip\History
```

There is no cloud sync or macOS history import. Pause capture, exclude source applications, or hide previews in Settings when needed. Sensitive clipboard markers are checked before reading content; apps that provide no marker cannot be identified reliably as sensitive sources. The shipped app's updater uses this repository's GitHub Releases; development dependencies and the WebView2 installer have separate download requirements.

Dragged image exports live independently under `History\DragImages`, so documents can continue referencing them. In **设置 → 存储**, inspect their size and request cleanup. Cleanup preserves the images in clipboard history, but exported paths used by documents may stop working. Clear them only after those references are no longer needed.

Close LocalClip before upgrading, backing up, or restoring data. Uninstall preserves history/settings by default. The full uninstall UI has an initially unchecked option to delete this Windows user's data; quiet uninstall preserves it unless `REMOVELOCALCLIPHISTORY=1` is explicitly passed. Updater snapshots are retained separately under `%LOCALAPPDATA%\LocalClip-UpdateBackups` and are not automatically deleted.

### Build from source

#### Requirements

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

#### Checks and generated test data

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

### Repository layout

| Directory | Contents |
| --- | --- |
| [core/](core) | Domain behavior, SQLite storage, search, transforms, and synthetic benchmarks. |
| [app/](app) | Windows clipboard, paste, drag/drop, OCR, updater, installer, and Tauri integration. |
| [ui/](ui) | Bundled React interface, previews, highlighting, and browser checks. |
| [test-vectors/](test-vectors) | Committed macOS behavior fixtures consumed without the reference checkout. |
| [vector-export/](vector-export) | Optional Swift exporter for regenerating reference fixtures. |
| [scripts/](scripts) | Validation, toolchain, privacy-scan, and signing helpers. |
| [changes/](changes) / [reports/](reports) | Scoped implementation notes, acceptance records, and known limitations. |

#### macOS behavior reference

The original baseline is LocalClip macOS **0.3.29 (51)** at `c097591431c00982ad34186068064b40965c95fd`. Later Windows parity work includes the [macOS 0.3.38 behavior mapping](changes/active/windows-macos-038/parity.md); intentional fixture changes are recorded in [macos-038-overrides.json](core/tests/macos-038-overrides.json).

Regenerating fixtures requires macOS 15+, Swift 6, and the read-only reference repository, defaulting to `../rea/macos-app` or `LOCALCLIP_MAC_PATH`:

```sh
TZ=Asia/Shanghai swift run --package-path vector-export VectorExport test-vectors
python3 scripts/validate_vectors.py test-vectors
python3 scripts/check_phase0.py
```

The exporter uses synthetic databases and never reads or migrates personal clipboard/history data. Keep its build products and caches inside this repository. Normal builds and CI use the committed fixtures and do not need the reference checkout.

### Signing, updates, and acceptance

#### Code signing policy

[Bilingual signing policy](CODE_SIGNING_POLICY.md) · [Privacy policy](PRIVACY.md) · [SignPath application and eligibility review](docs/signpath-application.md). We are preparing a SignPath Foundation free OSS signing application; it has not been submitted, approved or integrated. Current packages remain unsigned. Once the service is approved and active, the planned attribution is “Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org)”; this is not a claim of current sponsorship.

Production update installation requires a committed public minisign key in `app/updater-public-key.txt`, plus these CI secrets:

- `WINDOWS_UPDATER_PRIVATE_KEY` and `WINDOWS_UPDATER_KEY_PASSWORD`.
- Current PFX signing path: `WINDOWS_CODESIGN_PFX_BASE64` and `WINDOWS_CODESIGN_PFX_PASSWORD`. SignPath requires a separate cloud-signing integration after approval, not merely values for these PFX secrets.

Never commit private keys. The workflow skips unavailable signing credentials and labels unsigned artifacts. A version-matching tag creates a **draft** GitHub release. Update downloads never install automatically: installation requires an explicit user action, signature/architecture/version validation, and a database snapshot. Without the public key, installation requests are rejected.

Recent evidence includes [0.1.14 double-click paste and installation](reports/double-click-0.1.14.json) and [0.1.13 detail/editor windows](reports/dialog-0.1.13.json). Those checks have specific package and environment scope; they do not establish the complete external-app matrix.

Outstanding gates include clean Windows 10/11 lifecycle coverage, the full Word/browser/terminal/chat paste and password-manager/Ditto matrix, mixed-DPI behavior, complete OCR/language-pack acceptance, signed-update/rollback acceptance, and all PRD performance targets. See the [manual acceptance checklist](reports/manual-testing.md) and [Phase 2](reports/phase-2.md), [Phase 3](reports/phase-3.md), and [Phase 4](reports/phase-4.md) reports. Heterogeneous image/text merges are rejected rather than silently dropping formats; HTML-only/RTF-only mixed merges use plain text for the other representation.

### License

[MIT](LICENSE). Bundled dependencies retain their own notices, including [highlight.js](ui/licenses/highlight.js.txt).
