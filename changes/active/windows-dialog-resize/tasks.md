# Detail/editor window correction

Project root: `/Users/ronny/VscodeProjects/localclip-windows`.
Change directory: `/Users/ronny/VscodeProjects/localclip-windows/changes/active/windows-dialog-resize`.
Authority: current user request and two annotated screenshots. This is a targeted UI correction; no separate design/spec document is present or needed to resolve the explicit behavior.

- [x] Trace native sizing, title dragging, dialog DOM and CSS; distinguish old screenshot appearance from current source.
- [x] Reuse a shared opaque dialog surface and title bar for detail/preview and editing; hide unrelated history, retain notifications and OCR cancellation.
- [x] Enable native title dragging and eight-edge/corner resizing for both dialogs with a common default size and bounded minimum; keep history/settings behavior.
- [x] Verify minimum/large layouts, long-content scrolling, header/action visibility, pointer isolation, Save/Cancel/IME/shortcuts and history restoration.
- [x] Build/check native Windows implementation and verify actual movement/resizing on generated data; package the tested correction.

No changes to payload, storage schema, retention, history/settings, card drag, smart paste, OCR, highlighting or image cache logic. No new dependency, runtime network or OS privilege.

Local validation: production build and 35-command ACL consistency, locked Clippy, 24 menu/editor checks, 11 preview/layout checks and 13 history/settings alignment checks passed. Windows native compilation and full source CI passed (39005508887). Actual window tests passed 18 detail plus 17 editor checks; 13 native editor regression checks also passed. All 11 final native UI parity checks passed: 59 native checks in total. ARM64/x64 packages built and passed four Defender scans with AV/realtime enabled.

- [x] Install 0.1.13 in the VM at C:/apps/LocalClip after a verified backup; MSI exited 0, all 6 existing files hash-identical before launch, one registry entry/exact EXE hash verified, normal launch running as PID 4684. No antivirus settings/exclusions changed. Original idle-pause preference restored.
