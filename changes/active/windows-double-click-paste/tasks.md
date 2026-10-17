# Restore double-click direct paste

Project root: `/Users/ronny/VscodeProjects/localclip-windows`.
Change directory: `/Users/ronny/VscodeProjects/localclip-windows/changes/active/windows-double-click-paste`.
Authority: user requests restoring automatic paste into the previously active app when double-clicking a history record. No separate design/spec exists; this is a bounded event-wiring correction.

- [x] Trace card activation, default action, selection references and existing native target/clipboard/focus validation.
- [x] Reproduce copy-default failure in generated browser tests; cover clicked item identity, multi-selection and retained default/rename behavior.
- [x] Route card-content double-click to explicit directPaste for the clicked ID, reusing the existing native output path.
- [x] Verify browser regressions and actual double-click receipt in a generated native text target, including panel dismissal and exact-once content.
- [x] Package and install the verified correction with a hash-verified backup, preserving history/settings and enabled antivirus.

No native input/focus security, storage, retention, formatting, card drag, smart-code-file paste, OCR, highlighting, image cache or dialog changes.

Local validation: production frontend/35-command ACL consistency, format/locked Clippy, 27 menu/editor and 13 history/settings browser checks passed. Full Windows/macOS CI and ARM64/x64 installer build passed for packaged source 96863f4c4285ec52e506db9f4bfa6b5790be80e2.

The final packaged ARM64 executable passed 15 native checks in a generated Notepad document: physical double-click with both default actions, exact once-only receipt at the existing caret, target focus/panel dismissal, unchanged preferences, retained Enter-copy and text drag. Both architectures passed four Defender MSI/EXE scans with protection enabled. Installed 0.1.14 in the VM; MSI exit 0, exact installed EXE hash/version and normal startup verified. All six existing data files were backed up and hash-identical before launch. No security settings or exclusions changed. Evidence: `reports/double-click-0.1.14.json`.
