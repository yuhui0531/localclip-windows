# Windows 0.1.4 verification

Application source: `e12e640` (later tooling/report-only changes do not alter the packaged application).
Installer workflow: https://github.com/yuhui0531/localclip-windows/actions/runs/38908993559
CI for application source: https://github.com/yuhui0531/localclip-windows/actions/runs/38908959257

## Boundaries

- Tests use generated records/files in the repository or the explicitly isolated Windows VM fixture. Capture is paused during desktop tests. No personal history is read.
- The read-only macOS reference is pinned to `844924e` (0.3.38). It was clean at the initial checks. At final delivery, concurrent uncommitted edits exist in `HistoryView.swift` and `HistoryViewModel.swift`; these were left untouched and are outside this pinned comparison.
- No antivirus setting, exclusion, or allowlist was changed. No sample was manually uploaded to an external scanner.
- The MSI is inspected without installing it over the user's application. Target-app results come from the extracted/CI-matching executable and the generated history, not a clean installed-artifact lifecycle.
- 360 is not installed in this VM. Its original detection name is unavailable. Independent Defender checks cannot prove the original 360 cause or its acceptance of the rebuilt package.

## Completed local checks

Rust 1.90.0: formatting and Clippy pass; 41 workspace tests pass (including 50 forced-crash recoveries). Frontend typecheck/build and 10 highlighting/size cases pass. Tests involving Windows APIs are not included in the macOS count.

The local launcher now selects the pinned Cargo, Clippy and rustfmt together, avoiding a Homebrew Rust 1.99 PATH mismatch observed during verification.

## Windows CI and packaged executable checks

Both macOS/Windows CI jobs and both architecture installer jobs completed successfully at the source SHA above. Windows native tests cover original image-file paths, Unicode plus original-file dragging, intervening clipboard changes, extensionless image content/version invalidation, and native PDF first-page rendering. The Windows adapter library passed 8 default-feature tests and 9 desktop-feature tests; these overlap and are not added together as distinct cases.

The ARM64 executable matching the MSI payload was exercised against generated records in a fresh Windows 11 fixture:

| Check | Result / evidence |
|---|---|
| Smart paste | Seven scenarios: enabled/disabled, copy, unknown/read-only target, restored focus after a button, hidden content. Supported targets received the exact source text; fallbacks retained the original file. [Results](evidence/parity-text-drag-code-native.json). |
| Text into Notepad | Inserted into the same document, preserving pre-existing generated text. [Results](evidence/parity-text-notepad-result.json). |
| Explorer drag | Cancel creates nothing; text exports a usable file; original code file copies with matching content. [Results](evidence/parity-text-explorer-result.json). |
| Persistent image cache and code drag | Five cases passed: persistent image, reuse, external edit preservation/new copy, original file with smart mode off, original file plus Unicode with smart mode on. Cleanup required confirmation and reduced owned cache bytes to zero; source remained. [Results](evidence/parity-native-cache-drag.json). |
| Focus and scrolling | Unpinned panel hides, pinned panel remains, hidden panel rejects output; wheel changes horizontal offset and idle scrollbar is transparent. [Focus](evidence/parity-focus.json), smart-paste evidence above. |
| Highlighting | Five code previews with 34 colored tokens; hiding content removes it and restoring content restores highlighting. [ARM64](evidence/parity-text-highlight-result.json), [x64](evidence/parity-text-highlight-result-x64.json). |
| x64 under ARM emulation | Real code preview, file/image sizes, detail consistency, hidden-content protection and deletion confirmation passed; confirmation did not delete records. [Results](evidence/parity-preview-guards-x64.json). |
| Cleanup UI | Storage displays cache usage and warns that referenced documents may lose images. Opened confirmation and cancelled. [Screenshot](evidence/parity-cache-confirmation.png). |

[Generated history screenshot](evidence/parity-history.png) and [local check summary](evidence/parity-local-summary.json) are retained with the report. Desktop tests are targeted regressions, not the full third-party application matrix.

## Installer and scanner results

Both MSI File tables contain only `localclip.exe`. The extracted executable hashes match the corresponding CI executables. Neither development executable is installed. MSI CustomAction tables contain direct `InvokeBootstrapper`, with no PowerShell download action. The embedded WebView2 bootstrapper has a valid Microsoft Corporation signature; WiX's support DLL also has a valid signature. The app and MSI remain unsigned. See [MSI inventory](evidence/parity-msi-inventory.json) and [extracted payload signatures](evidence/parity-msi-extracted-v2.json).

Defender scanned each architecture's MSI, app executable, Microsoft bootstrapper and WiX DLL: **8/8 returned exit 0 and “found no threats”**, with all scanned files still present. Real-time protection remained enabled. Engine `1.1.26090.9`, signatures `1.461.55.0`. [Full scanner evidence](evidence/parity-defender-scan.json). This is not a 360 result or proof of its original detection cause.

| Installer | Bytes | SHA-256 |
|---|---:|---|
| LocalClip_0.1.4_arm64.msi | 8,716,288 | `73efcb29b0ec498ed5f548434eb6b12ac984490ded180e05c0d226ad66543b85` |
| LocalClip_0.1.4_x64.msi | 9,207,808 | `a8a88af54195f87dfd4108e32d9378be2848ab3ca9eac90de71329fc15175406` |

Local delivery folder: `dist/manual-test-0.1.4/{arm64,x64}`. Build artifacts are also available in the installer workflow above.

## Test cleanup

The isolated app and generated Notepad tab are closed, and generated Explorer windows were closed. The VM's original idle-pause setting was restored to `on`. The personal installation was not replaced. [Cleanup evidence](evidence/cleanup.json).

## Previously approved open acceptance gates

Signed automatic update/install/recovery (keys unavailable), clean Windows 10 22H2 installation lifecycle, the full third-party Word/browser/Typora matrix and full DPI/multi-display performance acceptance remain open. Office previews depend on locally installed Windows Shell providers. These are not counted as passed by a successful build or unit test.
