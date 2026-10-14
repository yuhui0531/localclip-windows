# Phase 4 tasks

Base: Phase 3 PR #4. Owner approved an unsigned manual-test installer with
signed-update acceptance open until signing keys are provided. Desktop gates
remain explicitly deferred; Phase 3 CI continues before delivery.
Reference start status: externally dirty; no writes by this task.

- [x] P4-01 — MSI x64/ARM64, WebView2 bootstrapper; §11/12.
- [x] P4-02 — Secret-only optional signing and release artifacts; §11/12.
- [x] P4-03 — GitHub-only daily checks, fixed-key signed download, explicit install, snapshot and receipt; §5.17/WAC24.
- [x] P4-04 — Uninstall keep-by-default, opt-in current-user history deletion; §11.
- [x] P4-05 — Build/download/hash installer, VM smoke where available, README/manual test ledger; §13.

These boxes track the owner-approved unsigned manual-test scope. Full-manifest
authentication, production signing, WAC24, required-hardware performance and the
remaining desktop acceptance matrix are open in phase-4.md and manual-testing.md.
