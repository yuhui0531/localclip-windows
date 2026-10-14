# Phase 4 — Manual-test installers and guarded updater

The owner explicitly approved **“Deliver manual-test installer; signing keys are
not ready.”** Installer signing and WAC24 remain open. No production keys were
generated, requested, printed or committed. No release tag or public release was made.

## Implementation mapping

| Task | Rules | Implementation |
|---|---|---|
| P4-01 | §11/12 | x64 and ARM64 Windows MSVC builds; MSI with downloadBootstrapper WebView2 |
| P4-02 | §11/12 | Release workflow reads optional certificate/updater secrets; explicit unsigned messages, SHA256 and architecture metadata; tag builds create draft releases |
| P4-03 | §5.17/§7 | Fixed GitHub endpoint, persisted daily failure throttle, backend verified download, signed-version binding, repo/architecture/downgrade checks, explicit install command, writer-serialized VACUUM INTO snapshot, startup acknowledgement and manual recovery notification |
| P4-04 | §11 | MSI full-UI checkbox defaults unchecked; deletion action is gated on explicit property, full uninstall and no major-upgrade removal; update snapshots remain outside history |
| P4-05 | §13 | Installation/build/update instructions and complete manual checklist; artifact verification in progress |

The committed public-key file is empty. `update_install` rejects immediately before
any snapshot, package write or installer launch, even if invoked directly over IPC.
The updater plugin has no frontend capability permission; only the guarded app
commands are allowed. Downloads cannot install on normal exit. No frontend-supplied
URL or package bytes are accepted.

The standard Tauri protocol authenticates package bytes and the signed version;
the complete latest.json document is TLS-protected, not independently signed.
Strict whole-manifest signature acceptance from §5.17 remains open with WAC24.
No signed-update acceptance is claimed for this unsigned build.

## Verification

- Local format, workspace Clippy and TypeScript/Vite build pass.
- Snapshot regression uses generated data, verifies SQLite integrity and row
  preservation, and proves existing backup files cannot be overwritten.
- Manual-build guard and repository/architecture asset policy tests pass.
- Phase 3 native VM smoke used only generated records with capture paused; it
  discovered and fixed the Windows thumbnail protocol host/path handling issue.
- Initial release run 38508151381 compiled both native targets successfully and
  explicitly skipped code signing. Bundling failed because the existing icon was
  not listed in bundle.icon; that configuration is corrected for the next run.
- Final build, hashes, sizes and installer/VM evidence pending below.

## Acceptance status

The owner deferred unavailable desktop gates. Win10 22H2 and clean Win11 full
install/use/update/restart/uninstall, password-manager/Ditto/Word scenarios,
mixed-DPI, target-app receipt matrix and WAC24 are **unexecuted**, not passed.
The available Parallels VM is Windows 11 ARM64, 4 CPUs/8 GiB; x64 emulation or a
120-record paused fixture cannot satisfy the required §10 performance conditions.

## Read-only reference

The Phase 4 start check found external modifications in rea, already recorded in
Phase 3. This task has not written or executed builds/tests there. The baseline is
still pinned to c097591431c00982ad34186068064b40965c95fd. An empty end status cannot
be claimed while external work is present; it will be rechecked at delivery.
