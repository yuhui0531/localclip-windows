# Privacy policy / 隐私政策

[简体中文](#简体中文) | [English](#english)

## 简体中文

适用于 LocalClip for Windows；依据 2026-10-17 的 `0.1.14` 源码行为编写。本文件是公开说明，不代表安装器已展示此政策或已完成 SignPath 隐私合规审核。

### 本地数据及用户控制

LocalClip 在本机处理剪贴板中的文本、富文本、链接、图片及文件引用，并保存来源应用、时间、使用记录、收藏、分类和设置。代码预览、文本转换及 Windows OCR 在本机执行。LocalClip 没有账户、云同步、广告分析或自建的遥测上传服务；应用代码不向项目维护者或签名服务上传剪贴板正文、OCR 结果或历史数据库。

默认数据目录为 `%LOCALAPPDATA%\LocalClip\History`；拖出图片副本保存在其中的 `DragImages`。可在设置中暂停采集、排除来源、隐藏内容、设置保留期限及容量、删除记录和清理图片导出缓存。敏感标记的保护依赖来源应用提供标记，不能保证识别所有敏感内容。

正常升级与默认卸载保留历史和设置。完整卸载时，用户可明确选择删除当前用户历史；更新快照 `%LOCALAPPDATA%\LocalClip-UpdateBackups` 单独保留。手动清理导出图片不会删除历史原图，但可能使引用导出路径的文档失去图片。

复制、粘贴、拖出、另存为和打开链接/文件是用户主动触发的输出。接收应用、浏览器、邮件程序、网络共享或用户选择的同步文件夹有其自身的数据处理行为，不由 LocalClip 控制。

### 联网与第三方组件

| 场景 | 对方及控制方式 |
| --- | --- |
| 更新检查 | 更新器设计为访问本仓库 GitHub Releases；“设置 → 关于 → 自动检查更新”默认开启，可关闭，另可手动检查。请求会产生 IP、请求时间和客户端信息等通常的网络元数据，不上传历史。当前发行版未配置更新公钥，自动安装不可用；不能把该限制当作永久无网络保证。 |
| WebView2 | 桌面界面使用 Microsoft Edge WebView2。缺少运行时时，MSI 内嵌的微软引导程序可能联网安装运行时。运行时自身还可能执行更新、诊断、崩溃报告和 SmartScreen 安全通信；部分诊断受 Windows 设置控制，必需数据并非均可关闭。 |
| 主动打开或输出 | 用户打开历史中的链接或文件、粘贴和拖出时，由目标程序或系统处理，可能涉及网络。 |

LocalClip 包含 WebView2 所提供的 Microsoft Defender SmartScreen 安全功能；该功能可能向微软发送用户信息，详见[微软隐私声明](https://www.microsoft.com/en-us/privacy/privacystatement)及 [WebView2 数据与隐私说明](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy)。GitHub 请求适用 [GitHub 隐私声明](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)。因此，本项目不作“整个应用及全部第三方组件绝不联网”的承诺。

开发依赖下载和将公开二进制提交给 SignPath 属于构建/发布流程，不是向用户应用添加签名服务连接。[SignPath 隐私说明](https://signpath.io/privacy-policy)适用于维护者使用该服务时的数据处理。

### 安装与系统变更

MSI 安装程序和快捷方式、注册卸载信息，并可能安装 WebView2；安装完成后的首次启动由用户选择。登录启动由应用设置控制，默认关闭。卸载程序支持保留数据，或在明确选择后清理当前用户历史。无需关闭杀毒软件或添加白名单。

隐私问题可通过[项目 Issues](https://github.com/yuhui0531/localclip-windows/issues)反馈，勿附带个人剪贴板内容、历史数据库或凭据。

## English

Applies to LocalClip for Windows, based on `0.1.14` source behavior reviewed on 2026-10-17. This public notice does not establish that the installer displays it or that SignPath privacy eligibility has been approved.

### Local data and controls

LocalClip processes clipboard text, rich text, links, images and file references locally, storing source-app information, timestamps, usage, favorites, categories and preferences. Code previews, transformations and Windows OCR run locally. LocalClip has no accounts, cloud synchronization, advertising analytics or project-operated telemetry upload service. Application code does not upload clipboard bodies, OCR results or history databases to maintainers or signing services.

The default directory is `%LOCALAPPDATA%\LocalClip\History`; exported drag images live under `DragImages`. Settings support pausing capture, excluding sources, hiding content, retention/quota limits, record deletion and exported-image cleanup. Sensitive-marker protection depends on source applications and cannot identify all sensitive data.

Normal upgrades and default uninstall preserve history/settings. Full uninstall offers explicit current-user history deletion. Update snapshots under `%LOCALAPPDATA%\LocalClip-UpdateBackups` remain separate. Export cleanup preserves history originals but can break documents referencing exported paths.

Copy, paste, drag, Save As and opening a link/file are user-directed outputs. Recipient apps, browsers, mail clients, network shares or selected synchronized folders have their own data practices outside LocalClip's control.

### Networking and third parties

| Scenario | Destination and controls |
| --- | --- |
| Update checks | The updater targets this repository's GitHub Releases. Settings → About → automatic update checks is on by default and can be disabled; manual checks are also available. Requests expose ordinary network metadata such as IP address, time and client information, not clipboard history. The current release lacks an update public key and cannot install updates automatically; this is not a permanent guarantee of no network activity. |
| WebView2 | The desktop UI uses Microsoft Edge WebView2. The embedded Microsoft bootstrapper may download the runtime if missing. The runtime itself may perform updates, diagnostics, crash reporting and SmartScreen communications. Windows settings govern some diagnostic collection; required data is not entirely optional. |
| Explicit opening/output | User-selected links/files and clipboard output are handled by destination applications or Windows and may involve networking. |

LocalClip includes Microsoft Defender SmartScreen through WebView2. This feature may send user information to Microsoft as described in the [Microsoft Privacy Statement](https://www.microsoft.com/en-us/privacy/privacystatement) and [WebView2 data/privacy documentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy). GitHub requests are subject to the [GitHub Privacy Statement](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement). We do not claim that the complete app and all third-party components never communicate over a network.

Development downloads and submitting public binaries to SignPath belong to build/release operations, not an added signing-service connection in the shipped app. The [SignPath privacy policy](https://signpath.io/privacy-policy) applies when maintainers use that service.

### Installation and system changes

MSI installs program files/shortcuts, registers uninstall information and may install WebView2. First launch after installation is a user choice. Login startup is controlled in Settings and defaults to off. Uninstall supports data preservation or explicit current-user history cleanup. Disabling antivirus or adding exclusions is unnecessary.

Raise privacy questions through [project Issues](https://github.com/yuhui0531/localclip-windows/issues). Do not attach personal clipboard content, history databases or credentials.
