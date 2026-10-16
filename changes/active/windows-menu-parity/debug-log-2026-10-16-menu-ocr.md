# Menu / OCR diagnosis

Repository: `/Users/ronny/VscodeProjects/localclip-windows`. Change directory: `changes/active/windows-menu-parity`.

- [x] Compare the supplied screenshot with macOS menu and action implementations.
- [x] Trace Windows menu layout, selection, OCR command registration and detail rendering.
- [x] Confirm missing frontend actions and fixed-position menu visibility problems; OCR backend already existed.
- [x] Add missing entry points using existing store/output/OCR/preview functionality; use one validated stored target for explicit Open.
- [x] Check short-viewport positioning, pagination, default-format override, transforms, category creation and hidden-content/cancellation behavior with 14 real-browser cases.
- [ ] Prove actual OCR, associated-app open, save and existing interaction regressions in the isolated Windows app.
- [ ] Install the exact tested package after backup and verify personal data preservation.

Root-cause confidence is high for the source/UI gaps: the actions were absent or incorrectly positioned in the previous Windows frontend. This is implementation omission, not a request to change product rules. The correction retains the native drag/smart-paste/highlighting/cache implementations and aligns the screenshot action order with macOS. File-reference Save As is omitted as on macOS; New Category remains accessible even when categories already exist.

Self-review: changes stay in menu/UI entry points and explicit Open handling; no schema migration, capture-policy change, dependency addition or antivirus change. Stored targets are reloaded and checked; hidden content or a hidden panel cannot open files/links. Native acceptance is still pending the personal app exiting; browser mocks are not treated as OCR or clipboard acceptance. Exact evidence and remaining checks are in `verification.md`.
