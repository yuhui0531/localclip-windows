# Code signing policy / 代码签名政策

[简体中文](#简体中文) | [English](#english)

## 简体中文

### 当前状态与范围

LocalClip for Windows 的 SignPath Foundation 免费开源签名申请于 2026-10-17 **已提交，等待审核**。尚未获批、尚未接入 SignPath；**获批后再接入 CI**。已发布的 `v0.1.14` LocalClip EXE/MSI 仍未签名；本政策不会改变这些文件，也不会启用自动更新。

申请范围为从[本仓库](https://github.com/yuhui0531/localclip-windows)公开源码构建的 Windows x64、ARM64 LocalClip 主程序及 MSI。只为本项目申请签名，不提供代签服务。第三方文件不使用本项目签名覆盖；微软 WebView2 引导程序保留其原有微软签名，其打包资格须由 SignPath 确认。

### 团队职责

| 职责 | 负责人 | 责任 |
| --- | --- | --- |
| 代码提交者 | [yuhui0531](https://github.com/yuhui0531) | 维护源码、依赖及构建脚本。 |
| 代码审查者 | [yuhui0531](https://github.com/yuhui0531) | 审查外部贡献，特别关注安装器、权限、依赖和 CI 变更。 |
| 签名批准者 | [yuhui0531](https://github.com/yuhui0531) | 核对来源、版本、产物和检查结果，逐次人工批准签名。 |

维护者于 2026-10-17 确认上述职责及 GitHub 双重验证已启用。当前为单维护者项目，不宣称存在独立第二人审查。未来启用 SignPath 前，所有获得访问权的成员必须启用 SignPath MFA；新增角色成员应先更新此政策。

### 获批后的签名流程

1. 只接受公开源码和受审查构建配置产生的 GitHub Actions 产物，构建至提交签名之间使用 GitHub 托管 runner，不上传个人历史或本机任意二进制。
2. 在 SignPath 配置中限制仓库、发布来源、产物路径、产品名 `LocalClip` 和一致版本；仅签指定主程序与安装包，核验第三方原有签名。
3. 每次发布由批准者在 SignPath 人工批准。签名私钥由服务方保管，不导出、共享或提交到仓库；CI 提交凭据仅放在受保护的 GitHub Actions secret 中，不向外部 PR 提供。
4. 验证两个架构的主程序、MSI、签名及时间戳后，才计算最终 SHA-256 和发布。Windows Authenticode 与 Tauri 更新签名分别验证；缺少更新密钥时继续保持自动安装禁用。

### 隐私与服务说明

请阅读[隐私政策](PRIVACY.md)，其中说明本地剪贴板数据、更新检查、WebView2 及用户主动输出的边界。签名服务只接收公开构建产物及来源信息，不接收用户剪贴板数据库。

申请获批、服务实际启用后，主页和下载页将使用以下鸣谢；**目前仅为计划使用的措辞，不表示已获赞助**：

> Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

申请条件及撤销机制以 [SignPath Foundation 官方条件](https://signpath.org/terms)为准。签名证明发布来源和文件完整性，不保证应用无缺陷或所有安全软件都不报警。问题可通过[项目 Issues](https://github.com/yuhui0531/localclip-windows/issues)反馈；请勿上传个人历史、凭据或私钥。

## English

### Status and scope

LocalClip for Windows **submitted its application for free open-source signing through SignPath Foundation on 2026-10-17 and is awaiting review**. The application has not been approved and SignPath is not integrated; **CI integration will begin only after approval**. Published `v0.1.14` LocalClip EXEs/MSIs remain unsigned. This policy changes neither those files nor automatic update availability.

The requested scope is the Windows x64 and ARM64 LocalClip executables and MSI installers built from this [public repository](https://github.com/yuhui0531/localclip-windows). We do not sign unrelated software. Third-party signatures must not be replaced with the project's signature. Microsoft's WebView2 bootstrapper retains its Microsoft signature; its inclusion requires SignPath's eligibility review.

### Responsibilities

| Role | Member | Responsibility |
| --- | --- | --- |
| Committer | [yuhui0531](https://github.com/yuhui0531) | Maintain source, dependencies and build scripts. |
| Reviewer | [yuhui0531](https://github.com/yuhui0531) | Review external contributions, especially installer, permission, dependency and CI changes. |
| Signing approver | [yuhui0531](https://github.com/yuhui0531) | Check provenance, version, artifacts and validation results; manually approve each signing request. |

On 2026-10-17, the maintainer confirmed these roles and that GitHub two-factor authentication is enabled. This is a single-maintainer project; independent second-person review is not claimed. Before SignPath activation, everyone granted access must enable SignPath MFA. Update this policy before adding role members.

### Signing process after approval

1. Accept only GitHub Actions artifacts built from public source and reviewed build configuration, using GitHub-hosted runners through signing submission. Never upload personal history or arbitrary local binaries.
2. Restrict the repository, release sources, artifact paths, product name `LocalClip` and matching versions in SignPath. Sign only designated project binaries/installers; verify existing third-party signatures.
3. Require manual SignPath approval for every release. Signing private keys stay with the service. Keep the CI submission token in a protected GitHub Actions secret, unavailable to external pull requests.
4. Verify both architectures' executable/MSI signatures and timestamps before calculating final SHA-256 hashes and publishing. Validate Authenticode and Tauri update signatures independently; keep automatic installation disabled without update keys.

### Privacy and service attribution

The [privacy policy](PRIVACY.md) describes local data, update checks, WebView2 and user-directed output. Signing receives public build artifacts and provenance, never users' clipboard databases.

After approval and activation, the homepage and download pages will display the following attribution. **It is planned wording, not a claim of current sponsorship:**

> Free code signing provided by [SignPath.io](https://signpath.io), certificate by [SignPath Foundation](https://signpath.org).

Eligibility and revocation remain subject to the [Foundation's conditions](https://signpath.org/terms). Signatures establish origin and integrity, not absence of defects or universal antivirus acceptance. Report problems through [project Issues](https://github.com/yuhui0531/localclip-windows/issues), without personal history, credentials or private keys.
