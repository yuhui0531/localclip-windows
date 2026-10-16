# Context menu and OCR verification

Application source: `ad5517e49a446b569ea2a1c4f251007182e63c0e` (0.1.5). Verified 2026-10-17 in the user's Parallels Windows 11 ARM64 VM. macOS comparison: 844924e, `HistoryView.swift` menu and `HistoryViewModel.swift` actions. Current macOS HEAD 509ce4e (0.3.39) retains these menu definitions. Its checkout remained read-only; final `git status --porcelain` was empty.

## Cause and correction

The previous Windows menu lacked rename, locate, preview and open commands; OCR existed in the backend but fixed menu placement could hide its entry outside the short panel. OCR was absent from the detail header. The menu now anchors to the click, stays inside the viewport and scrolls when needed. Menu and detail OCR share the existing native recognition path. Original copy overrides the configured default format; transformation actions paste, with Alt for copy.

Native acceptance found a further Open failure: the new command was missing from the Tauri command ACL. The final correction registers `open_item` and grants only `allow-open-item` to the main window. Associated-app Open passed on the rebuilt final executable. No generic shell/filesystem permission was added.

No drag, smart-paste, native OCR engine, image cache, database schema or syntax-highlighting implementation was replaced. Existing behaviors were exercised on the final binary, not inferred from unchanged source.

## Final native verification

All native checks below used capture-paused generated data at `C:\Users\Public\Documents\LocalClip-menu-20261016`. The personal instance was closed by the user. Clipboard inspection required ownership by the isolated test app; target documents/windows were identified before interaction. The generated receiver, Notepad tab, Explorer window and isolated app were closed afterwards.

| Action | Final Windows VM result |
|---|---|
| Anchored menu / screenshot actions | Passed: labels, OCR entry and viewport bounds |
| Original copy | Passed: PNG bytes equal source despite plain-text default |
| Transform paste | Passed: uppercase text reached receiver even with default action Copy |
| OCR menu / details / image-file card | Passed: actual recognition and clipboard output from all three entry paths |
| OCR cancellation / hidden content / missing language | Passed: cancellation leaves clipboard unchanged; hidden content disables OCR; unavailable English language gives guidance |
| Rename / favorite / category | Passed: persisted rename, favorite and assignment; category-plus entry opens categories |
| Locate history / category | Passed: same record retained after clearing filters and paging beyond the first page |
| Details / preview / Open | Passed: detail OCR, preview visible, actual associated Notepad file opened |
| Save As | Passed: native dialog completed and saved PNG matched source bytes |
| Delete / Undo | Passed: protected-delete confirmation and record restoration |
| Mixed filter | Passed with generated mixed record |
| Code highlighting | Passed: 39 colored tokens; privacy hides previews and disabling privacy restores them |
| Smart code-file paste | Seven cases passed: off, Copy, on, unknown target, editable target after button, read-only target, hidden content |
| Text drag into Notepad | Passed: inserted into the same document; existing before/after text preserved |
| Explorer drag | Passed: cancel creates no file, text exports correct contents, file drop copies original bytes |
| Image / code-file drag | Five cases passed: image, cache reuse, externally modified cache, smart option off/on; paths still exist after drag |
| Long-lived image cache / manual cleanup | Passed: Settings shows usage and warning; Cancel retains cache; Confirm clears exported copies; original history image remains byte-identical and can still be saved |
| Mouse wheel / scrollbar | Passed: horizontal offset 240 after vertical wheel; idle scrollbar transparent |
| Focus / pin | Passed: unpinned panel hides, pinned remains, output from hidden panel rejected |

18 menu checks, seven smart-paste cases, five image/file drop cases and three Explorer cases are recorded individually, with additional Open/Save, transform, cancellation, highlighting, Notepad and focus results. Raw PowerShell output had a nested array wrapper for the first 13 menu checks; the consolidated evidence flattens that wrapper into 18 original checks without changing results.

The native menu sequence was interrupted by a Parallels command-channel disconnect after 13 completed checks. No remaining test runner was found; the final five checks were resumed against the same generated store and executable. This interruption is not reported as an application failure or as a passed unexecuted test.

## Automated and package verification

- Frontend typecheck/build and 14 real Chrome interaction cases passed, zero page errors. These cover new-category entry/create, keyboard/menu wiring, default-format override, cancellation/progress and privacy. Browser fixtures are separate from native clipboard/OCR evidence.
- 43 local Rust tests passed, including 50 crash-recovery cycles and the new safe-open-target tests; Clippy passed.
- Final [Windows/macOS CI](https://github.com/yuhui0531/localclip-windows/actions/runs/38926066451): passed, including Windows adapters, desktop tests and application build.
- Final [ARM64/x64 installer build](https://github.com/yuhui0531/localclip-windows/actions/runs/38926128113): passed.
- Both MSIs install only localclip.exe; extracted executables match CI artifacts. No PowerShell download custom action. Embedded Microsoft bootstrapper and WiX helper signatures are valid.
- All eight Defender component scans passed with real-time protection enabled. The samples are bound to final hashes in the evidence. No AV settings or allowlist changes were made.

| Artifact | SHA-256 |
|---|---|
| ARM64 executable | `026c2d9cd78a1a0844f3b1cbb170ea883b2ca90350dc277c78fd0491cf7acf58` |
| ARM64 MSI | `78b5082598e79ccb3067bd10d184db6c05fbe85bb1117b0831b40b8b44cb60f8` |
| x64 executable | `7eb265bef5327b34ffa7063d335e63691fbcaa4ee4437f138dbf85c88b565233` |
| x64 MSI | `d4fa1e4a7dc1cc0a8415f192aa2529619c4ee7face12907a2e3ed2bca24b96ba` |

## Installed-artifact verification

The personal LocalClip data directory was copied and all six existing files hash-verified before upgrade. Backup retained at:

`C:\Users\ronny\AppData\Local\LocalClip-UpgradeBackups\before-0.1.5-20261017-000742`

The exact ARM64 MSI completed with exit 0 using history removal disabled. Before first launch, every existing data-file hash remained unchanged. The registered version and executable version are 0.1.5; the installed executable at `C:\apps\LocalClip\localclip.exe` matches the final tested SHA-256 above. It launched normally as the user, PID 4852 at verification. Defender antivirus and real-time protection remained enabled. VM idle pause was restored to its original enabled setting.

Machine-readable evidence: [`reports/menu-parity-0.1.5.json`](../../../reports/menu-parity-0.1.5.json). Detailed raw scripts/results remain locally under ignored `reports/raw/`; the report contains generated test data and installation metadata, not personal history contents.

## Limits

- The Windows OCR engine with zh-Hans-CN returned `LocalClip OCR 12345` and `图 片 文 字 识 另 刂 测 试` for the Chinese fixture `图片文字识别测试`. Recognition works, but spacing and the character 别 were inaccurate. This is not an exact-transcription pass. No English OCR language pack is installed; its missing-language guidance was verified.
- Desktop acceptance ran on Windows 11 ARM64. x64 packages built and were scanned; x64/Windows 10 desktop behavior was not rerun here.
- App/MSI remain unsigned as previously approved for manual testing. This VM has Defender, not 360; these checks do not establish that 360's earlier detection is resolved.
- This acceptance covers the requested menu/OCR additions and listed regressions; it does not close unrelated earlier full-product release gates.
