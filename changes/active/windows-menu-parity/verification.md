# Context menu and OCR verification

Application source: `a37679fd0db484135e38abb3a23df1545cd9db00` (0.1.5). macOS comparison: 844924e, `HistoryView.swift` menu and `HistoryViewModel.swift` actions. The macOS checkout was read only. Its current HEAD is 509ce4e (0.3.39); the later detail-layout changes do not alter these context-menu action definitions. Final `git status --porcelain` is empty.

## Cause and correction

The previous Windows menu lacked rename, locate, preview and open commands; its OCR command existed but the fixed menu placement could put the entry away from the clicked card or outside the short panel. OCR was also absent from the detail header. The menu now anchors to the click, stays within the viewport and scrolls when needed. Both menu and detail OCR share the existing native recognition path. Original copy explicitly overrides the configured default format; transformation actions explicitly paste, with Alt for copy.

No drag, smart-paste, native OCR engine, image cache, database schema or syntax-highlighting implementation was replaced. Their final-binary regressions remain required; an unchanged source path is not runtime acceptance.

## Verification matrix

| Action | Implemented/browser verification | Final Windows VM verification |
|---|---|---|
| Anchored menu and visible screenshot actions | Passed at 1200 × 340, bounds and labels checked | Pending |
| Original copy | Passed: original format overrides plain-text preference | Actual image clipboard pending |
| Transform paste | Passed: directPaste even when default action is copy | Receiver pending |
| OCR menu and detail | Both dispatch OCR; visible progress, cancellation dispatch and hidden-content disabled entry passed | Chinese/English generated image, file-image OCR, missing-language message, cancellation and privacy checks pending |
| Rename | Selected record title updated | Persistence pending |
| Favorite/category | Assignment and favorite wiring passed; new-category remains available | Persistence/confirmations pending |
| Locate history/category | Generated record 120 found across multiple pages; filters cleared and ID retained | Real store paging pending |
| Details/preview/open | Detail entry, Space preview, Ctrl+O invocation passed; safe target parsing unit tests passed | Actual associated-app open pending |
| Save as | Invocation passed; file-reference menu correctly omits it like macOS | Native save dialog and output bytes pending |
| Delete | Invocation passed | Protected confirmation and undo pending |
| Mixed filter/category plus | Passed | Native fixture pending |
| Code highlighting | Actual syntax spans still render in browser | Installed WebView pending |
| Drag/smart paste/wheel/focus/cache | Existing paths retained; test drivers prepared | Not executed on 0.1.5 |

## Executed checks

- `npm --prefix ui run build`: passed.
- `node ui/tests/menu.mjs` with installed headless Chrome and fixture IPC: 14 passed, zero page errors. This is frontend interaction evidence, not actual clipboard/OCR evidence.
- `sh scripts/cargo-local.sh test --workspace --locked`: 43 tests passed again after the final corrections, including 50 crash-recovery cycles and the two new open-target tests.
- `sh scripts/cargo-local.sh clippy --workspace --all-targets --locked -- -D warnings`: passed again after final fixture/menu corrections.
- First implementation f1e587c: [Windows/macOS CI](https://github.com/yuhui0531/localclip-windows/actions/runs/38916906653) passed; [ARM64/x64 installers](https://github.com/yuhui0531/localclip-windows/actions/runs/38916912607) passed.
- Final a37679f: [CI](https://github.com/yuhui0531/localclip-windows/actions/runs/38919170804) and [installers](https://github.com/yuhui0531/localclip-windows/actions/runs/38919229978) dispatched; status must be refreshed before delivery.
- Preliminary f1e587c MSIs: only localclip.exe is installed; contained EXE hashes equal CI artifacts; no DownloadAndInvokeBootstrapper/PowerShell installer action. Embedded Microsoft bootstrapper and WiX helper signatures valid. Eight Defender component scans found no threats. Raw evidence: `reports/raw/menu-msi-inventory.json`, `menu-msi-extracted-v2.json`, `menu-defender-scan.json`. These results must not be attributed to later a37679f packages.
- App/MSI remain unsigned, as previously approved for manual testing. VM has Defender, not 360; no claim about 360 detection resolution.

## Installation boundary

The personal app remained running from `C:\apps\LocalClip\localclip.exe`; the single-instance lock prevents an isolated fixture instance. Remote tray activation was unreliable and the computer-use surface returned `noWindowsAvailable`. The user was asked to quit from the tray. No forced termination, personal data modification, antivirus change, allowlist entry or 0.1.5 installation was performed.

Prepared isolated root: `C:\Users\Public\Documents\LocalClip-menu-20261016`, capture paused, generated records/images/source only. Next: close personal instance, run native suite, fix any findings, then hash-verified backup and MSI upgrade with history removal disabled; verify all existing data-file hashes before launching the installed app.
