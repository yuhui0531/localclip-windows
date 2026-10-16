# Draggable editor without history backdrop

Project root: `/Users/ronny/VscodeProjects/localclip-windows`.
Change directory: `/Users/ronny/VscodeProjects/localclip-windows/changes/active/windows-editor-window`.
Baseline: `ce7acd1`. User requests title-bar dragging and removal of content behind the editor.

- [x] Switch edit mode to a compact native window, hide underlying history without unmounting its state, and restore the normal panel on close.
- [x] Use native window dragging from the editor title bar only; preserve text selection, Save/Cancel, IME and pending-save focus containment.
- [x] Verify 22 menu/editor plus 13 layout/action browser checks and full Windows/macOS source CI (38960409040).
- [x] Build ARM64/x64 installers (38960413626), verify metadata hashes and pass four Defender MSI/EXE scans with enabled protection.
- [ ] Verify actual native title-bar movement and install with a verified history/settings backup after the personal instance is closed.

No schema, payload, history/settings, native card drag, paste, OCR, highlighting or cache changes. No new dependencies or OS permissions. The user request is the behavior/design authority for this small correction.
