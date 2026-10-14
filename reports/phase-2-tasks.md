# Phase 2 tasks and exit gate

Baseline: Phase 1 PR #2 (`182b11f` report closure; executable code `1072de2`).
Phase 1 vectors, Windows/macOS CI and WAC16 passed. Phase 2 stage-start read-only
reference `git status --porcelain`: empty. No PR has been merged.

- [ ] P2-01 — Native format codecs and tests: UTF-16, CF_HTML, file URLs/CF_HDROP, original image bytes (§4.1, §5.11; WAC02/04).
- [ ] P2-02 — Message-window listener, debounce, bounded reads, sequence checks and policy-before-body filtering (§5.1 W-CAP-1–9, §5.4; WAC01/11).
- [ ] P2-03 — Receipt IDs, eager clipboard writes and exit persistence (§5.2/5.10; WAC10/12).
- [ ] P2-04 — Owner/foreground source resolution, process identity and ApplicationFrameHost child resolution (§5.3).
- [ ] P2-05 — Foreground WinEvent tracking, target/focus checks, elevation fallback and SendInput flow (§5.8/5.15; WAC05–07).
- [ ] P2-06 — Main/queue shortcuts, queue registration lifetime, tray/autostart/single-instance (§5.9/5.10/5.14; WAC13).
- [ ] P2-07 — Local WinRT OCR, language failures, cancellation and no result persistence (§5.16; WAC23).
- [ ] P2-08 — Headless `localclip-demo capture|paste <id>|ocr <id>` and metadata-only logs (§13).
- [ ] P2-09 — Serial Windows integration tests, data-directory privacy scanner and actual WAC01/02/04–07/10–13/23 evidence (§8/12/13).
- [ ] P2-10 — Workspace Clippy, clean reference end check, report and separate PR (§13).

The user offered to test an app build manually if VM use is unavailable. The VM
supports guest commands but pauses between uses; required third-party acceptance
apps were not found in the checked locations. A clarification about deferring
unavailable desktop gates is pending. No test has been waived or marked passed.
