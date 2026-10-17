# SignPath application package / SignPath 申请材料

Reviewed / 核查日期：2026-10-17. Source baseline / 源码基线：[`6c86bee`](https://github.com/yuhui0531/localclip-windows/commit/6c86beea8ddf1adad16e21e6366a6545d637039d), version `0.1.14`.

**Prepared, not submitted or approved. / 材料已准备，申请尚未提交或获批。** This document contains public project information only; contact email belongs in the private application, not the repository. / 本文件仅含公开项目资料，联系人邮箱仅用于私下申请，不入库。

## Form answers / 表单填写内容

Fields and choices below were checked on the live [application form](https://signpath.org/apply). / 以下字段与选项已按实际申请表核对。

| Form field / 字段 | Answer / 填写值 |
| --- | --- |
| Project Name | LocalClip for Windows |
| Repository URL | https://github.com/yuhui0531/localclip-windows |
| Homepage URL | https://github.com/yuhui0531/localclip-windows |
| Download URL | https://github.com/yuhui0531/localclip-windows/releases/tag/v0.1.14 |
| Privacy Policy URL | https://github.com/yuhui0531/localclip-windows/blob/main/PRIVACY.md |
| Wikipedia URL | Leave blank; no article is claimed. / 留空，不宣称存在词条。 |
| Category | Productivity |
| Maintainer Type | Individual maintainer(s) |
| Build System | GitHub Actions |
| First Name / Last Name | Enter the maintainer-confirmed contact details privately. / 使用维护者确认的联系人信息，私下填写。 |
| Email | Enter privately; do not add it to this document. / 私下填写，不添加至本文件。 |
| Company Name | Leave blank; no company affiliation is claimed. / 留空，不宣称公司背景。 |
| Primary Discovery Channel | AI / LLM tools |
| Please specify the exact source | Codex-assisted review of Microsoft Learn's Windows code-signing options and the SignPath Foundation website. |

### Tagline / 一句话介绍

English, ready to paste:

> A local clipboard history manager for Windows that helps users find and reuse copied text, images, links and files.

中文对照：面向 Windows 的本地剪贴板历史工具，帮助用户查找并复用复制过的文本、图片、链接和文件。

### Description / 项目描述

English, ready to paste:

> LocalClip for Windows helps users keep, search and organize clipboard history on their own device. Users can reuse entries through copy, paste or drag-and-drop, edit supported text records, and recognize text in images locally. The project is MIT-licensed and maintained in a public repository. It provides Windows x64 and ARM64 installers. LocalClip has no account system or cloud history synchronization. Its privacy policy separately explains update checks, user-directed output and the Microsoft WebView2 runtime.

中文对照：LocalClip for Windows 在用户设备上保存、检索和整理剪贴板历史，支持复制、粘贴、拖放、文本编辑和本地图片文字识别。项目采用 MIT 许可证、公开维护，提供 x64/ARM64 安装包，无账户和历史云同步；隐私政策单独说明更新检查、主动输出及 WebView2 行为。

### Reputation / 信誉材料

English, ready to paste:

> This is an early-stage Windows project. The repository was created on 2026-10-13, and v0.1.14 is a public prerelease. We do not yet claim widespread adoption, independent media coverage or an established download reputation. Verifiable evidence includes public source/history, successful source CI, hosted x64/ARM64 installer builds, and published package-verification records. These demonstrate provenance and maintenance, not independent reputation. Please assess eligibility on that basis; we understand acceptance is discretionary.
>
> Source and history: https://github.com/yuhui0531/localclip-windows
>
> Source CI: https://github.com/yuhui0531/localclip-windows/actions/runs/39030566002
>
> Installer build: https://github.com/yuhui0531/localclip-windows/actions/runs/39034321070
>
> Release: https://github.com/yuhui0531/localclip-windows/releases/tag/v0.1.14
>
> Verification record: https://github.com/yuhui0531/localclip-windows/releases/download/v0.1.14/release-verification.json

中文对照：Windows 仓库刚建立，公开预发布和 CI 可证明来源与维护，不能等同于用户规模或独立信誉。没有编造媒体报道、下载量、推荐或第三方背书；接受与否由基金会审核。

## Eligibility review / 申请条件核查

The Foundation's [conditions](https://signpath.org/terms) remain authoritative. “Ready” means repository evidence exists, not Foundation approval. / 官方条件为准；“具备材料”不代表审核已通过。

| Item / 条件 | Result / 结果 | Evidence or remaining work / 依据或待办 |
| --- | --- | --- |
| Public source and project license / 公开源码与项目许可 | Ready / 具备材料 | GitHub API confirms public MIT repository; [LICENSE](../LICENSE). No project commercial dual-license found. / API 确认公开 MIT；未发现项目商业双许可。 |
| Maintainer controls repository / 仓库控制权 | Ready / 具备材料 | API lists `yuhui0531` as the sole collaborator with admin role. / API 列出唯一管理员。 |
| Maintained, released, documented / 维护、发行与功能说明 | Ready / 具备材料 | Public commit history, successful CI, v0.1.14 prerelease and bilingual README. / 提交记录、CI、预发布和双语 README。 |
| Dependency licensing / 依赖许可 | Preliminary check only / 初步核查 | Offline locked desktop dependency metadata for Windows x64 covers 316 resolved Rust packages including build/dev dependencies; all declare licenses. npm lockfile has 85 dependency entries, all with license fields. This is not a complete SBOM, ARM64 audit or license-compliance certification. / 已核查元数据，不等同完整制品审计。 |
| WebView2 proprietary component / WebView2 非开源组件 | Foundation decision required / 待基金会确认 | `embedBootstrapper` packages a Microsoft-signed bootstrapper. Request confirmation under the system-library exception; preserve its signature. Do not remove or replace functionality to claim eligibility. / 请求确认系统组件例外，保留微软签名及功能。 |
| Privacy and third-party notices / 隐私及第三方告知 | Public notice prepared; review open / 已备公开说明，审核待定 | [PRIVACY.md](../PRIVACY.md) discloses WebView2 diagnostics/SmartScreen and update checks. Current installer does not display this policy. Ask whether installation-time notice/options are required for this architecture. No installer change is included. / 安装器未展示该政策；须确认是否要求安装时告知或选项。 |
| System changes and uninstall / 系统变更与卸载 | Documented; review open / 已说明，待审核 | MSI program/shortcut registration, optional login startup, WebView2 installation and opt-in data removal are disclosed in the privacy policy. / 安装与卸载行为已公开说明。 |
| GitHub MFA / GitHub 双重验证 | Maintainer-confirmed / 维护者确认 | Owner confirmed enabled on 2026-10-17; no recovery codes or authenticator secrets collected. / 不收集恢复码或验证器密钥。 |
| Roles and per-release approval / 角色与逐次批准 | Designated / 已指定 | [Code signing policy](../CODE_SIGNING_POLICY.md); owner confirmed all three roles. Single-maintainer arrangement is disclosed for review. / 如实披露单维护者安排。 |
| SignPath account/MFA / SignPath 账户与 MFA | Not configured / 尚未配置 | Maintainer must complete account activation and MFA after invitation. / 维护者在邀请后完成激活和 MFA。 |
| Verifiable hosted build / 可验证托管构建 | Existing foundation / 已有基础 | Installer workflow uses GitHub-hosted Windows runners for both architectures. / 双架构均使用 GitHub 托管 Windows runner。 |
| SignPath origin/configuration enforcement / 来源与产物约束 | Not integrated / 尚未接入 | Configure repository, source refs, artifact paths, product/version restrictions, submitter and manual approver after approval. / 获批后配置并验证。 |
| Website/download policy links / 主页和下载页政策入口 | Ready / 具备材料 | README and the v0.1.14 download page link the policy; unsigned status stays explicit. / README 与 v0.1.14 下载页提供政策入口，未签名状态保持明确。 |
| Verifiable project reputation / 项目信誉 | Uncertain / 不确定 | Young repository; no independent usage/reputation evidence claimed. CI and antivirus scans are not reputation substitutes. / 新仓库，CI 与杀毒扫描不能替代信誉。 |
| Malware/PUA and security-tool exclusions / 恶意软件与工具限制 | Reviewer decision / 由审核方判定 | Clipboard utility with user-directed output; no vulnerability-exploitation purpose identified in reviewed paths. Existing verification is not a universal clean bill of health. / 说明真实用途，不宣称普遍安全认证。 |

### Questions for the reviewer / 给审核方的问题

1. Can the existing Microsoft-signed WebView2 bootstrapper qualify as a system component? It is never re-signed using the project certificate. / 内嵌 WebView2 引导程序是否符合系统组件例外？
2. Given local UI content and the runtime's own diagnostic/security behavior, are the public privacy notice and Windows controls sufficient, or is installer notice/another control required? / 隐私告知与控制是否需要补充安装器展示？
3. Is the disclosed single-maintainer committer/reviewer/approver arrangement acceptable? / 单维护者三角色安排是否可接受？
4. Is the current public provenance sufficient for an early-stage project, or should we reapply with further independent reputation evidence? / 现阶段材料是否足够，或需后续积累信誉再申请？

## Owner actions / 维护者需操作

1. Open the [application form](https://signpath.org/apply), paste the answers above and enter the private contact details. / 打开申请表，粘贴材料并填写私下联系人。
2. Personally review the required Code of Conduct acceptance and personal-data processing consent before submission; optional marketing consent is not required for our request. Complete any CAPTCHA personally. / 本人核对行为准则与个人数据处理同意，并处理验证码；无需勾选可选营销订阅。
3. Submit and retain the receipt; no submission or approval is claimed in this package. / 提交后保留回执；本材料未宣称已提交或获批。
4. On invitation, activate SignPath, enable MFA and review the GitHub connector's requested repository access before granting it. Scope access to this repository. / 收到邀请后激活、开启 MFA，审阅并限制 GitHub 授权范围。
5. Supply only non-secret organization ID/project/policy identifiers for integration. Enter `SIGNPATH_API_TOKEN` directly into GitHub Actions secrets; do not send passwords, OTPs, recovery codes or tokens in chat. / 仅提供非敏感项目标识；令牌直接填 GitHub secret，不在聊天发送。

## CI integration handoff, not implemented / CI 接入交接，尚未实施

Keep the current application, installer behavior, version, history and settings unchanged during application preparation. / 本次准备期间不改变应用、安装器行为、版本、历史或设置。

After approval, replace the PFX-only signing calls in [.github/workflows/release.yml](../.github/workflows/release.yml) with the [official SignPath Action](https://docs.signpath.io/trusted-build-systems/github). Upload unsigned hosted-build artifacts first; SignPath validates their origin. Its [artifact configuration](https://docs.signpath.io/artifact-configuration/) can sign the MSI and nested LocalClip executable while verifying, rather than overwriting, Microsoft's bootstrapper signature. Include standalone EXEs in the explicitly approved artifact configuration. Maintain per-release manual approval; do not sign local binaries or bypass origin checks.

获批后再替换现有 PFX 调用，先上传托管构建产物，再由 SignPath 验证来源并签名。可利用嵌套签名同时处理 MSI 和其中的 LocalClip 主程序；独立 EXE 也需明确纳入产物配置。第三方微软组件仅验签，不覆盖签名；保留每次发布的人工批准。

Verify signed artifacts before generating Tauri update signatures, hashes and release metadata. Updater keys are separate and remain unconfigured by this documentation change. Never relabel old unsigned release assets as signed or replace published binaries silently. / 先验证 Windows 签名，再生成更新签名、哈希和发布元数据；更新密钥独立配置，本次不启用。不得将旧未签名文件改称已签名或静默替换发行文件。
