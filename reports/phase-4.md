# Phase 4 — Manual-test installers and guarded updater

**Latest manual build: 0.1.3.** See the [text-drag fix and current acceptance status](../changes/active/fix-text-drag/debug-log-2026-10-14-text-drag.md). The [0.1.2 history wheel/code-feature evidence](../changes/active/fix-history-wheel-code/debug-log-2026-10-14-history-wheel-code.md), [0.1.1 drag/focus native regression](../changes/active/fix-drag-focus/debug-log-2026-10-14-drag-focus.md) and 0.1.0 evidence below remain historical records; they are not substituted for current-version runtime acceptance.

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
| P4-05 | §13 | Installation/build/update instructions and complete manual checklist; verified two MSI hashes/architectures and installed ARM64 smoke |

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
- Final CI [38509029498](https://github.com/yuhui0531/localclip-windows/actions/runs/38509029498)
  passed Windows and macOS for cef3ad2: core vectors/storage/crash suite, formatting,
  Clippy, Windows serial adapters and desktop unit tests, frontend typecheck/build,
  Release app and service benchmarks. [Concise log](phase-4-evidence/ci-checks.txt).
  Benchmark execution success is not a claim that all PRD thresholds passed.
- Snapshot regression uses generated data, verifies SQLite integrity and row
  preservation, and proves existing backup files cannot be overwritten.
- Manual-build guard and repository/architecture asset policy tests pass.
- Phase 3 native VM smoke used only generated records with capture paused; it
  discovered and fixed the Windows thumbnail protocol host/path handling issue.
- Initial release run 38508151381 compiled both native targets successfully and
  explicitly skipped code signing. Bundling failed because the existing icon was
  not listed in bundle.icon; that configuration is corrected for the next run.
- Release build [38509027331](https://github.com/yuhui0531/localclip-windows/actions/runs/38509027331)
  passed both x64 and ARM64 for exact code commit `cef3ad210963df03c04ecbfdfc4c3594402cdeaf`.
  [Build excerpt](phase-4-evidence/installer-build.txt) retains explicit signing skips.
- Both downloaded MSIs match their workflow SHA256 metadata. The standalone PE
  machines are ARM64 `0xaa64` and x64 `0x8664`; MSI summary architectures are
  `Arm64;0` and `x64;0`. [Artifact verification](phase-4-evidence/artifacts.json),
  [MSI metadata](phase-4-evidence/msi-metadata.json).
- ARM64 MSI installation in the existing Parallels Windows 11 VM returned exit 0.
  The installed executable hash matches the downloaded ARM64 executable.
  [Installation evidence](phase-4-evidence/vm-install.json).
- The installed native app used only a generated 120-record, capture-paused database.
  Real WebView2 IPC passed search (3 requested results), detail, usage, delete/undo,
  thumbnail retrieval and 8 rendered cards. Update status reported installation
  disabled and no download; direct installation returned `signedUpdatesDisabled`.
  [Smoke metadata](phase-4-evidence/vm-smoke-final.json),
  [native screenshot](phase-4-evidence/vm-native-final.png).
- Removing the test-installed ARM64 MSI returned exit 0 and removed its executable.
  All three generated custom-directory files retained identical SHA256 values.
  [Uninstall evidence](phase-4-evidence/vm-uninstall.json). This does not prove
  default-path history retention or opt-in deletion; personal history was not read.
  The test instance and frontend server were stopped, and the VM's original
  idle-pause setting was restored.
- Installer metadata for both architectures has no default history-deletion value;
  the action requires full uninstall, explicit opt-in and no upgrade removal.
  This proves packaging configuration, not execution of opt-in data deletion.

## Manual-test packages

| Package | Bytes | MiB | SHA256 |
|---|---:|---:|---|
| LocalClip_0.1.0_arm64.msi | 9,904,128 | 9.445 | `347cd2f987cdada97c4f555766105ab99bea7495caa040073b20c2b267f02b56` |
| LocalClip_0.1.0_x64.msi | 10,915,840 | 10.410 | `cd60708a14583d09a1eacd449a6682e24c9c515155e302408c3c5c2f40287ef7` |

Both are below 15 MB excluding WebView2, unsigned, version 0.1.0. Local delivery:
`dist/manual-test/arm64/` and `dist/manual-test/x64/`, including SHA256SUMS and
standalone executables. Use ARM64 for the owner's Apple Silicon Parallels VM.
Public distribution has not been performed.

## Resource diagnostic — performance gate remains open

The same generated 120-record fixture stayed capture-paused; the panel was hidden.
The native ARM64 process tree (including WebView2, eight processes) after five
minutes had **562,118,656 summed working-set bytes (536.08 MiB)** and
**195,493,888 private committed bytes (186.44 MiB)**. Five-minute CPU average was
0.1029% of four CPUs. [Aggregate samples](phase-4-evidence/vm-resource-native.json).
Working-set sums double-count shared pages; private committed bytes are not
resident memory. Neither measurement satisfies the memory target, and the native
CPU window is shorter than the required ten minutes.

The earlier x64-emulation ten-minute sample averaged 0.0391% CPU, with
596,590,592 summed working-set bytes and 237,223,936 private committed bytes after
five minutes. [Earlier samples](phase-3-evidence/vm-resource-smoke.json).
These are diagnostics on an 8 GiB VM with a small paused fixture. They do not
substitute for the specified 16 GiB/NVMe dataset, GUI latency samples or normal
capture workload. No performance acceptance is claimed.

## Acceptance status

The owner deferred unavailable desktop gates. Win10 22H2 and clean Win11 full
install/use/update/restart/uninstall, password-manager/Ditto/Word scenarios,
mixed-DPI, target-app receipt matrix and WAC24 are **unexecuted**, not passed.
The available Parallels VM is Windows 11 ARM64, 4 CPUs/8 GiB; x64 emulation or a
120-record paused fixture cannot satisfy the required §10 performance conditions.

## Read-only reference

The Phase 4 start check found external modifications in rea, already recorded in
Phase 3. This task has not written or executed builds/tests there. The baseline is
still pinned to c097591431c00982ad34186068064b40965c95fd. The delivery recheck still found external source/document/test changes. An empty
end status cannot be claimed. These changes were neither modified nor reverted.
