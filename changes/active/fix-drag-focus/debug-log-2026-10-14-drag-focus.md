# Drag-out and focus-loss regression

Project: /Users/ronny/VscodeProjects/localclip-windows
Scope: file drag-out and unpinned history dismissal; preserve pinned/modal behavior.

- [x] Read card gesture, perform IPC, OLE drag implementation, foreground tracker and desktop lifecycle.
- [x] Check pinned dependency source and official Tauri/Microsoft contracts.
- [x] Root-cause challenge: required behaviors are unchanged; defects are Windows integration logic, not a product requirement change.
- [x] Local formatting, workspace Clippy and portable app tests (7) pass.
- [x] Native Windows compile/tests and packaged build.
- [x] Before/after native pointer drop and focus regression using generated, capture-paused data.

Hypotheses: (1) Tauri's default native drop interception suppresses frontend HTML drag initiation (high confidence; documented requirement to disable it). (2) OLE drag on a fresh worker STA does not share the window's input queue (high confidence; code starts a new thread with no source HWND). (3) WebView LostFocus alone misses top-level deactivation (medium confidence; dependency deliberately suppresses native window focus events); a drag can also suppress dismissal while the modal flag is set.

Fix: disable the WebView interception; perform native OLE drag on the window UI STA with its HWND, release prior mouse capture, and cancel if the initiating mouse button has already been released. Use the existing event-driven foreground tracker to schedule dismissal on the UI thread. Recheck current foreground PID plus pin/modal guards when handling queued events and after drag completes. No polling, clipboard writes or content logging added. Version 0.1.1 permits MSI upgrade from 0.1.0.

Test fixture tool accepts an optional generated file URL for real file-card regression.

## Verified results

Exact app code: `81d960ce836fc952713eee854714389ddbbc2b04`, version 0.1.1.
[Windows/macOS CI](https://github.com/yuhui0531/localclip-windows/actions/runs/38517365818)
and [x64/ARM64 installers](https://github.com/yuhui0531/localclip-windows/actions/runs/38517365661)
passed. [CI excerpt](evidence/ci.txt) and [package hashes](evidence/artifacts.json).

Native pointer tests ran in Windows 11 ARM64 Parallels with a generated 120-record
history, capture paused, and a generated text file with a space in its filename.
A blank WinForms OLE receiver copied the supplied file; source and receiver HWNDs
were verified before mouse input. No personal clipboard or history was read.

| Check | 0.1.0 | 0.1.1 |
|---|---|---|
| Native file drop received | **Fail**, no file | **Pass**, matching SHA256 |
| Original source remains | Pass | Pass |
| Outside click hides unpinned panel | Pass in this scenario | Pass |
| Outside click preserves pinned panel | Pass | Pass |
| Escape cancels drag without a copy | Not executed | Pass |
| Outside click after cancellation hides panel | Not executed | Pass |

Evidence: [before](evidence/native-before.json), [after](evidence/native-after.json),
[cancellation](evidence/native-cancel.json). The initial launcher exited its receiver,
so that first comparison was discarded. The valid comparison used a live receiver.
Long encoded VM commands also exceeded the launcher limit; invoking the saved
script through a short command resolved that test-driver issue.

The reported focus-loss failure was **not reproduced in the simple before-build
outside-click case**. The change adds an independent native foreground event and
post-drag check; only the specific after-build behaviors above are claimed verified.
Explorer/other destination matrices and the prior release/performance gates remain open.

Delivery: `dist/manual-test-0.1.1/arm64/LocalClip_0.1.1_arm64.msi` and
`dist/manual-test-0.1.1/x64/LocalClip_0.1.1_x64.msi`. Both downloaded MSI SHA256 values
match CI metadata; PE architectures match their labels. The tested ARM64 executable
matches the executable supplied with the ARM64 MSI artifact. Installer upgrade was
not run against the user's existing installation. Signing remains disabled as approved.

Self-review: fixes address Windows input/activation integration; no polling, unrelated
refactoring, new dependencies or private-data logging. Main-thread queued dismissal
rechecks the live foreground PID, and pinned/modal guards remain in place. Tests
confirm source retention and cancellation. Generated test processes are stopped
and the VM's original pause-idle setting is restored after testing.

References: https://docs.rs/tauri/2.12.3/tauri/webview/struct.WebviewWindowBuilder.html#method.disable_drag_drop_handler ; https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shdodragdrop
