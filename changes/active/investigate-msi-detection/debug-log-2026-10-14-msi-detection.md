# 360 对未安装 MSI 的报警排查

日期：2026-10-14。范围：本地已交付 0.1.3 ARM64 / x64 MSI，只读分析。

用户确认：接收安装包即报警，尚未安装；要求先说明原因，不关闭杀毒、不加白名单、不削减现有功能。用户实际报毒样本的版本、架构、SHA-256 和 360 检测名称尚未提供，因此不能认定本次检查样本与报毒样本完全一致。

## 已完成

- [x] 读取实际 MSI 的 File、Media、Binary、CustomAction 和安装执行序列表。
- [x] 不执行安装，使用 Windows Installer 只读数据库 API 读取 CAB 和 WiX DLL，再用 expand.exe 解包。
- [x] 核对包内 EXE 与独立 CI 交付 EXE 的 SHA-256；两个架构均一致。
- [x] 检查 Authenticode 签名和实际二进制中的更新提醒字符串。
- [x] 检查 VM 防病毒注册和相关进程：仅发现 Windows Defender，没有发现 360。
- [ ] 获取 360 检测名、引擎版本、报毒样本哈希和发生报警的电脑。
- [ ] 在原 360 环境分别扫描原 MSI 与解出的各个 EXE/DLL，记录命中对象。
- [ ] 根据命中对象决定是否制作保持功能一致的对照安装包。

## 证据与假设

| 发现 | 已验证事实 | 对 360 根因的判断 |
|---|---|---|
| MSI 的 WebView2 安装动作 | DownloadAndInvokeBootstrapper 包含隐藏 PowerShell、从微软地址下载 EXE、再执行安装；配置为 downloadBootstrapper | 中等优先级静态检测候选，未证实命中 |
| 主程序中的更新提醒 | 两种架构实际 EXE 均包含 powershell.exe、-EncodedCommand、update-pending.json 和失败提醒字符串；源码为 start_watchdog | 中等优先级静态检测候选。禁用自动更新不等于二进制中不存在相关字符串；未证明执行过或被命中 |
| 数字签名 | 两种 MSI、主程序、demo、fixture 均 NotSigned | 无签名是事实，但不能据此认定 360 的具体报毒原因；签名也不保证不报毒 |
| 多余交付工具 | File 表和实际 CAB 均包含 localclip-demo.exe 与 ui_fixture.exe | 高置信度打包范围问题；不是已证实的报毒根因 |
| WiX 辅助 DLL | 两包均含同一 WixUIWixca.dll；签名 Valid，签名主体 WiX Toolset (.NET Foundation) | 未发现该组件签名异常；不能据此代替恶意软件检测 |
| 包内主程序一致性 | 解出的 localclip.exe 与独立交付 EXE 哈希一致 | 排除了这两份本地文件之间的内容差异；不证明构建链整体无风险 |

开发工具进入 MSI 的链路：app/src/bin 下两个辅助入口被构建为额外二进制；自定义 WiX 模板的 binaries 循环将它们安装到应用目录。localclip-demo 用于隔离采集/粘贴/OCR 测试，ui_fixture 用于生成测试历史。主程序的剪贴板辅助进程通过 current_exe 启动自身，未发现依赖这两个工具的启动引用。生产功能保留与测试工具是否进入安装包是两个独立事项。

## 后续方案与边界

1. 优先用原环境、原样本和明确检测名确认命中对象。当前只有 Defender 的 VM 无法证明或复现 360 结论；未将其他杀毒结果当作替代证据。
2. 若仅 MSI 命中，可分别比较“排除开发工具”和“使用微软 WebView2 嵌入式 bootstrapper”的正规打包方案，每次只改变一个因素，保留依赖自动安装。
3. 若主程序命中，再检查更新提醒实现及依赖；必要时用原生实现保留失败提醒与更新保护，不通过隐藏字符串或改名规避检测。
4. 正式发行时签署实际交付 EXE 和 MSI；确认误报后通过 360 官方渠道复核。尚未向第三方上传样本。
5. 所有方案仍需同版本 360 复扫及安装/升级、文本和文件拖动、智能粘贴、代码高亮、滚轮和失焦隐藏回归。

本次未修改应用、打包配置或既有交付物，未安装 MSI、未运行解出的程序、未关闭杀毒或设置排除项。诊断过程中首次 COM 表读取因 FieldCount 获取方式不正确导致空行，已按显式列数修复并重新读取；首次解包脚本因 Windows PowerShell UTF-8 无 BOM 解析失败，改用 BOM 后成功。无失败结果被计为通过。

证据：evidence/msi-inventory.json、evidence/av-extracted.json、evidence/av-static-strings.json。解包文件保存在 reports/raw/av-extracted/，未加入版本控制。

只读流读取 API 参考：[Microsoft MsiRecordReadStream](https://learn.microsoft.com/en-us/windows/win32/api/msiquery/nf-msiquery-msirecordreadstream)。


## 继续自主排查：逐文件扫描与脚本来源

用户表示不清楚报警细节，要求自行排查；未再次要求用户收集资料。

- 使用现有 VM 的 Microsoft Defender 对新建隔离目录中的 10 份文件副本逐个执行 `MpCmdRun.exe -Scan -ScanType 3 -File <file>`，没有设置排除项或关闭实时防护，也未执行样本程序。
- 每个架构扫描 MSI、localclip.exe、localclip-demo.exe、ui_fixture.exe、WixUIWixca.dll。10 项均退出码 0，输出明确包含 `found no threats`，扫描后文件均仍存在。详见 evidence/av-defender-scan.json。
- 使用现有病毒库 1.459.691.0、引擎 1.1.26080.3、产品 4.18.26070.9；未宣称病毒库为最新。此结果只代表本次 Defender 扫描，不证明 360 不会命中，也不是全面安全认证。
- 从 Tauri 官方仓库的 `tauri-cli-v2.12.2` 标签获取 `crates/tauri-bundler/src/bundle/windows/msi/main.wxs`。本地 `DownloadAndInvokeBootstrapper` 的完整 CustomAction 行与官方版本逐字一致。该安装动作具有明确上游来源；不等于已证明它不会被启发式检测命中。见 evidence/av-bundler-origin.json。
- 先前尝试从 macOS Tauri CLI 二进制中搜索该模板没有命中，不能用这个结果判定来源；随后用官方 Windows MSI 模板完成核对。首次上游 URL 的目录不正确，已按官方文档提供的源码路径更正。
- 聚焦检查 app/src 的外部进程调用：clipboard worker 启动 current_exe 并通过标准输入输出读取剪贴板；更新提醒启动 PowerShell。搜索的进程注入 API 名称未在应用源码中命中。这个有限范围检索不覆盖全部第三方依赖或所有可能实现，不据此宣称不存在任何恶意行为。

当前判断：已确认多余测试工具进入交付包、交付物未签名；未确认木马感染或 360 误报根因。安装包未执行时报警与安装时行为触发必须区分，安装包仍可被扫描内含脚本及程序。

可执行整改顺序：先将开发工具排除在 MSI 交付范围外；用官方 embedBootstrapper 安装方式替代 PowerShell 下载动作，保留 WebView2 自动安装；若后续有主程序命中证据，再考虑以原生实现替换更新提醒子进程并保留提醒功能。修改后仍须同引擎复扫与功能回归，不能仅凭包哈希变化或 Defender 通过宣称 360 问题已解决。

参考：
- [Tauri 官方 MSI 模板（2.12.2）](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.12.2/crates/tauri-bundler/src/bundle/windows/msi/main.wxs)
- [Tauri 官方 WebView2 安装方式](https://v2.tauri.app/distribute/windows-installer/#embedded-bootstrapper)

应用源码、打包配置与交付安装包仍未修改；本轮新增扫描脚本、证据和排查记录。没有主动向第三方提交样本；使用的是现有 Defender 配置下的标准扫描。
