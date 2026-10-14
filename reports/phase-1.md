# Phase 1 — core (in progress)

Phase 1 is **not complete**. Do not advance to Phase 2 until the exit checklist
in [phase-1-tasks.md](phase-1-tasks.md) is satisfied.

Baseline: macOS 0.3.29 (51), `c097591431c00982ad34186068064b40965c95fd`.
Branch: `feat/phase-1-core`, based on Phase 0 `cafbe8d` (PR #1 remains unmerged).

## Current evidence

- Rust 1.90.0 installed inside `.tooling/`; Cargo.lock pins approved dependencies.
- All 15 reference vector files are executed: 440 cases. Two filename cases
  have explicitly approved Windows expectations in `core/tests/windows-overrides.json`.
- The first complete vector run passed, including SQLite rank and seven store scenarios.
- Separate tests cover unchanged bytes of every stored image representation,
  deduplication, transaction rollback preserving old history, stale previews/cursors,
  policy rejection before persistence, schema refusal and WAL/auto-vacuum configuration.
- Fifty real macOS child-process kills during open write transactions passed:
  `forced-termination iterations=50 integrity=50 atomic-records=50 foreign-keys=50`.
  Windows execution uses `taskkill /F`; **WAC16 not yet executed on Windows**.
- The local Parallels guest `Windows 11` is running, architecture ARM64.
  PowerShell guest commands work. Cargo/Rust/Node/npm were not found in PATH.
  No clipboard or personal data was read. No application build exists yet.

Commands use `rtk proxy sh scripts/cargo-local.sh ...` after installing the local
toolchain. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
and `cargo test -p localclip-core` are the relevant checks. Final rerun pending.

## Performance — every measured run retained

First macOS ARM64 Release run, 10,000 texts + 100 generated 1920×1080 images,
60 samples per metric, monotonic timer and nearest-rank P95:

| Boundary | P95 |
|---|---:|
| SQLite search | 117.435 ms |
| Text ingest | 37.626 ms |
| Image ingest (encoding excluded) | 95.341 ms |
| First search after commit | 149.797 ms |
| Warm search | 116.554 ms |

First-search P95 gap: **28.52%, failed <20% target**. Search service alone also
exceeds the eventual 100 ms input-to-render budget. These are macOS service
measurements, not Windows UI acceptance.

Raw samples: [service-results.json](phase-1-benchmark-macos-01/service-results.json).
Dataset: [fixture-manifest.json](phase-1-benchmark-macos-01/fixture-manifest.json).
The first run's database was 332,365,824 bytes (316.97 MiB), versus the approximate
226 MiB macOS reference. Text content is identical by generator rules; TIFF slots
become DIBV5 under §4.1, and Rust PNG/JPEG encoders produce different fixture bytes.
Captured images are never re-encoded. The synthetic database was removed after
measurement; manifests and all samples remain.

Search optimization and a second measurement are in progress. No failed sample
has been discarded or replaced.

## Approved decisions and implementation differences

- Product approved build-time downloads from official Rust, crates.io/npm, and IANA.
  Toolchain/cache directories remain inside this repository. No app network was added.
- Product approved replacing Windows-invalid filename characters with `-`, removing
  trailing dots/spaces, and prefixing reserved device names with `_`.
- Product approved preserving macOS bare-link recognition via documented exclusions;
  see [offline TLD notes](../core/data/README.md). Explicit HTTP(S) still supports
  those domains; reference vectors were not changed.
- Name, main/queue shortcuts, Win10 22H2 and ARM64 choices were confirmed. Actual
  signing identity, public key and secret availability still need release-stage evidence.
- CI is introduced in Phase 1 because its exit explicitly requires Windows/macOS
  CI green. Frontend/app checks will be added only in their relevant phases.
- macOS quota errors map to Windows `skipped(quotaFull)`, preserving the atomic result.
- The macOS source already uses in-place DELETE-journal transactions; Windows follows
  the explicit WAL/FULL requirement despite the PRD's outdated APFS description.

## Read-only reference checks and remaining work

Stage-start `git -C /Users/ronny/VscodeProjects/rea status --porcelain`: empty.
Intermediate repeat: empty. Final stage-end check pending.

Outstanding: finish storage API/boundary review, complete final checks, improved
performance measurements, green Windows/macOS CI, Windows WAC16, final report and PR.
Phases 2–4 have not started. No paste receipt, installer, update or release acceptance
is claimed.
