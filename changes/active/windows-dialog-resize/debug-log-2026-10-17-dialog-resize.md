# Detail/editor dragging, resizing and background exposure

- [x] Read App.tsx, TextEditor.tsx, style.css, macos.css, desktop.rs, command manifest/ACL and existing browser/native tests.
- [x] High-confidence cause: main native window is created non-resizable and never enabled for detail/editor. Only TextEditor has a pointer drag handler. Detail is an inset fixed overlay with history siblings still visible, exposing panel controls around its edges. Editor-only CSS covers editing, but does not cover details; its fixed 560×390 inner surface would stop growing if the native window were resized.
- [x] Question chain: expected behavior is explicit in the current user request. Existing layout, sizing and event wiring are incomplete, so the correction is authorized; no data model or output redesign is required.
- [ ] Verify a shared full-window opaque surface, matching title controls, native resize constraints and scrolling; preserve mounted history state.

The screenshots use the older green theme. They establish the unwanted visual result, but are not claimed as a reproduction on installed 0.1.12. Current-source defects above are independently traceable.

The added browser resize assertion failed against 0.1.12: at 800px viewport the editor began at x=120 instead of x=0 (fixed 560px inner surface). After the correction, 24 menu/editor and 11 preview/layout checks passed. Native APIs were checked against the installed Tauri 2.12.3 sources; stable Webview::window() supplies the resize dispatcher without enabling the unstable Manager::get_window API or adding dependencies.
