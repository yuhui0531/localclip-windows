# Draggable editor without history backdrop

Project root: `/Users/ronny/VscodeProjects/localclip-windows`.
Change directory: `/Users/ronny/VscodeProjects/localclip-windows/changes/active/windows-editor-window`.
Baseline: `ce7acd1`. User requests title-bar dragging and removal of content behind the editor.

- [x] Switch edit mode to a compact native window, hide underlying history without unmounting its state, and restore the normal panel on close.
- [x] Use native window dragging from the editor title bar only; preserve text selection, Save/Cancel, IME and pending-save focus containment.
- [x] Verify 22 menu/editor plus 13 layout/action browser checks and full Windows/macOS source CI (39003474304).
- [x] Build 0.1.12 ARM64/x64 installers (39003479132), verify metadata hashes and pass four Defender MSI/EXE scans with enabled protection.
- [x] Verify 4 actual window drag/restore checks, 13 native editor checks and 11 native UI parity checks against the packaged ARM64 build using generated data.
- [x] Install 0.1.12 with a verified history/settings backup; all 6 existing files hash-identical before launch, installed binary/registry match, normal startup confirmed (PID 2356). Defender AV/realtime stayed enabled.

No schema, payload, history/settings, native card drag, paste, OCR, highlighting or cache changes. No new dependencies or OS permissions. The user request is the behavior/design authority for this small correction.
