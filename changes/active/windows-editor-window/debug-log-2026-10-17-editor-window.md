# Draggable editor and redundant background — 0.1.11

Baseline: `ce7acd1`. Project root and change directory are declared in tasks.md.

- [x] Read TextEditor.tsx, App.tsx, style.css/macos.css and desktop.rs/commands.rs.
- [x] Trace edit opening through set_detail_view to the 1200×800 detail layout; verify the modal has a translucent backdrop and no title drag handler. Confidence: high.
- [x] Question chain: correction is within the explicit user request. The missing editor layout and title drag wiring are the cause; data storage and output paths are unrelated.
- [x] Add a bounded 560×390 editor mode to the existing native window, centered in its current monitor work area. Temporarily relax history's minimum width; restore the original layout/constraint on exit.
- [x] Hide underlying history with visibility rather than unmounting/collapsing its layout, preserving late-page selection and scroll. Use an opaque editor surface.
- [x] Start native window dragging from primary title-bar pointer presses only. Text and buttons keep normal pointer behavior. No new dependencies or OS permissions.
- [x] Production frontend build, local Clippy and 22 menu/editor plus 13 responsive/action checks passed. Generated editor screenshot inspected; no history shows behind it.
- [x] Windows shell compilation/Clippy and core/adapter/desktop tests passed. ARM64/x64 installers built (run 38960413626), MSI hashes match metadata, and four Defender scans passed with AV/realtime enabled and no setting changes. This is not a 360 rescan.
- [x] Full source CI run 38960409040 passed on Windows and macOS, including application build and benchmark.
- [ ] Native drag/upgrade gate; actual movement has not been marked passed.

Root chain: edit selected → shared expanded detail layout → enlarged history window remains rendered → translucent edit overlay exposes cards/toolbar/footer. Header has no window-movement handler. Correction gives editing its own layout and native title drag path, retaining existing edit transactions and Save/Cancel/IME/focus safeguards.

Self-check: both requested symptoms addressed; no data/schema/output/cache/OCR/highlight logic changed; no adjacent refactor or duplicate window lifecycle; Chinese file/method/fix comments present. Browser checks assert hidden history, compact bounds, title-only drag dispatch, restoration of late selection/scroll, cancellation, failed saves, IME, pending-save focus and hidden-content guards. Actual desktop movement is a separate native gate, not proven by mocked IPC.

Native verification remains pending: process-only inspection still shows the personal app at C:\apps\LocalClip\localclip.exe, PID 10964 (last verified installed version 0.1.8). No personal history or clipboard was read; no personal process was stopped. Prepared native test scripts were syntax-validated without execution. No 0.1.11 installation is claimed.
