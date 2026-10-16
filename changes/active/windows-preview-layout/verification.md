# Preview layout correction — 0.1.6

Application source: `a5f44eab214e7b3a390f9066974db9a50735998a`. Verified and installed on 2026-10-17 in the user's Parallels Windows 11 ARM64 VM.

## Changes

- Custom/detected card titles now precede content, with favorite/category indicators retained.
- Details and preview temporarily use a centered workspace bounded by the current monitor. Closing or reopening restores the saved history-panel geometry without changing preferences.
- Image preview requests original pixels, initially fits the whole image without stretching, and provides 适应窗口, 原始大小, zoom, scroll and pointer pan. The existing record/path/privacy and 40-million-pixel limits remain.
- Details allow full text and metadata scrolling. Text beyond the highlighter's 16,384-character budget displays completely as plain text; normal code highlighting remains.

## Verification

| Check | Result |
|---|---|
| Browser layout | 8 passed: title order, tall/wide image fit, original size/zoom/pan, close/restore dispatch, full long text, short and narrow viewports; 2× device scale |
| Existing browser menu/OCR | 14 passed; no page errors |
| Local Rust / Clippy / UI build | 43 tests passed, including 50 crash-recovery cycles; Clippy and build passed |
| Native layout | 11 checks passed; original 1000×240 pixels, fit/zoom/pan, visible close/OCR controls, usable detail body/inspector, close and reopen restoration |
| Native menu/OCR | 18 checks passed; actual clipboard/OCR, privacy, rename, favorite, category, locate, preview, protected delete/undo and mixed filter |
| OCR cancellation | Passed; clipboard unchanged |
| Native Open / Save As | Passed; associated Notepad opened; saved PNG bytes match source |
| Smart code-file paste | All 7 cases passed: off, Copy, on, unknown target, editable target after button, read-only target, hidden content |
| Highlighting / wheel | 34 colored tokens; hide/restore passed; wheel offset 240 and idle scrollbar transparent |
| Image/file drag and cache | 5 drop cases passed, including reuse and externally modified export; Settings cleanup Cancel/Confirm passed; original image still saves byte-identically afterwards |
| Notepad text drag | Inserted into the same generated document, preserving existing text |
| Explorer drops | Cancel leaves no file; text contents and original file copy match |
| Focus / pin | Unpinned hides, pinned stays, hidden output rejected |
| Final MSI components | Extracted EXEs match CI; Microsoft bootstrapper and WiX helper signatures valid; no installer PowerShell download custom action; 8 Defender scans passed |

Final [Windows/macOS CI](https://github.com/yuhui0531/localclip-windows/actions/runs/38934937386) and [ARM64/x64 installer builds](https://github.com/yuhui0531/localclip-windows/actions/runs/38934963127) passed. Native tests used generated history with capture paused. Test windows were closed and only clipboard content still owned by the test app was cleared before the personal app launched.

Native geometry was 4122×695 physical pixels for history, 2426×1615 for inspection, then restored exactly. Browser checks include a wide image matching the user's 3966×670 screenshot proportions. Local screenshots: `reports/raw/layout-native-preview.png`, `layout-preview-wide.png`, `layout-preview-2x.png`, and `layout-preview-narrow.png`. Browser images are generated fixtures; the native screenshot is from the final Windows binary.

## Installed artifact and preserved data

- Installed at `C:\apps\LocalClip\localclip.exe`; registered/file version **0.1.6**.
- ARM64 EXE SHA-256: `9c5462a7f5042454e582e917ec3b22d045ef5b1a9ba959986ecea2464b22585a`.
- ARM64 MSI SHA-256: `e74cc85020ffc2bc184ee9101a576c26ceeca3857a4a4d33f4e47a85862f5430`.
- MSI exit **0**. All **six** existing data files matched their pre-upgrade hashes before first launch.
- Verified backup: `C:\Users\ronny\AppData\Local\LocalClip-UpgradeBackups\before-0.1.6-20261017-011357`.
- Started normally as the user, PID 8344 at verification. Defender antivirus and real-time protection stayed enabled; no allowlisting. Original VM idle-pause setting restored.

[Hash-bound machine-readable evidence](../../../reports/preview-layout-0.1.6.json) includes both architectures, each native result, package checks and installed verification. No personal history contents are included.

## Limits

Desktop verification ran on Windows 11 ARM64; x64 packages built and were scanned. The unchanged Windows Chinese OCR engine still introduces spacing/character errors in the test image. Packages remain unsigned manual-test builds as previously approved; Defender results do not establish that the earlier 360 detection is resolved. This report does not close unrelated full-product release gates.
