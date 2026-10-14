# Drag-out and focus-loss regression

Project: /Users/ronny/VscodeProjects/localclip-windows
Scope: file drag-out and unpinned history dismissal; preserve pinned/modal behavior.

- [x] Read card gesture, perform IPC, OLE drag implementation, foreground tracker and desktop lifecycle.
- [x] Check pinned dependency source and official Tauri/Microsoft contracts.
- [x] Root-cause challenge: required behaviors are unchanged; defects are Windows integration logic, not a product requirement change.
- [x] Local formatting, workspace Clippy and portable app tests (7) pass.
- [ ] Native Windows compile/tests and packaged build.
- [ ] Before/after native pointer drop and focus regression using generated, capture-paused data.

Hypotheses: (1) Tauri's default native drop interception suppresses frontend HTML drag initiation (high confidence; documented requirement to disable it). (2) OLE drag on a fresh worker STA does not share the window's input queue (high confidence; code starts a new thread with no source HWND). (3) WebView LostFocus alone misses top-level deactivation (medium confidence; dependency deliberately suppresses native window focus events); a drag can also suppress dismissal while the modal flag is set.

Fix: disable the WebView interception; perform native OLE drag on the window UI STA with its HWND, release prior mouse capture, and cancel if the initiating mouse button has already been released. Use the existing event-driven foreground tracker to schedule dismissal on the UI thread. Recheck current foreground PID plus pin/modal guards when handling queued events and after drag completes. No polling, clipboard writes or content logging added. Version 0.1.1 permits MSI upgrade from 0.1.0.

Verification pending below; no native success claimed yet. Test fixture tool accepts an optional generated file URL for real file-card regression.

References: https://docs.rs/tauri/2.12.3/tauri/webview/struct.WebviewWindowBuilder.html#method.disable_drag_drop_handler ; https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shdodragdrop
