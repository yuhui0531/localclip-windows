# Phase 1 — core

Phase 1 implementation and executable exit checks are complete. See
[phase-1-tasks.md](phase-1-tasks.md) for the rule mapping and exit checklist.

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
  Windows execution uses `taskkill /F`; WAC16 passed on the Windows CI runner in [run 38497014488](https://github.com/yuhui0531/localclip-windows/actions/runs/38497014488).
  Both macOS and Windows jobs passed on `f7ab704` and core code revision `6c34a4e` (run 38497753643). Static-runtime revision `1072de2` is also green in run 38498258081.
- The local Parallels guest `Windows 11` is running, architecture ARM64.
  PowerShell guest commands work. Cargo/Rust/Node/npm were not found in PATH.
  No clipboard or personal data was read. No application build exists yet.

Commands use `rtk proxy sh scripts/cargo-local.sh ...` after installing the local
toolchain. `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
and `cargo test -p localclip-core -- --nocapture` passed after final code review:

```text
Finished `dev` profile (Clippy, -D warnings)
forced-termination iterations=50 integrity=50 atomic-records=50 foreign-keys=50
test result: ok. 1 passed; 0 failed (crash)
test result: ok. 2 passed; 0 failed (export)
test result: ok. 7 passed; 0 failed (storage)
test result: ok. 16 passed; 0 failed (15 vector files plus byte-literal rank)
```

All 440 cases ran. Storage tests additionally prove a WAL reader can search while
the writer holds an uncommitted transaction, disk-space refusal, preserved undo
metadata, invisible expired records and unchanged-title cursor validity.

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

The second local run uses borrowed SQLite blob arguments, optimized literal search,
and a persistent 32 MiB reader page cache. The fixture checksum is unchanged.

| Boundary | Second-run P95 |
|---|---:|
| SQLite search | 73.026 ms |
| Text ingest | 11.612 ms |
| Image ingest (encoding excluded) | 91.564 ms |
| First search after commit | 80.686 ms |
| Warm search | 72.701 ms |

First-search P95 gap: **10.98%, passed <20%**. Raw samples and fixture manifest:
[second run](phase-1-benchmark-macos-02/service-results.json).
This does not establish the Windows input-to-render target. No failed sample has
been discarded or replaced. Shared CI runner metrics are retained separately in
`phase-1-ci-38497014488/`; that environment does not match §10 desktop hardware.

## CI measurements

Run [38497014488](https://github.com/yuhui0531/localclip-windows/actions/runs/38497014488)
is green on both platforms. Windows uses Windows Server 2025 on the hosted runner;
this establishes core Windows compatibility and WAC16, not Win10/11 desktop UX.
[Raw Windows test summary](phase-1-ci-38497014488/windows-test-summary.txt).

| Host | Search P95 | Text ingest P95 | Image ingest P95 | First/warm P95 | Gap |
|---|---:|---:|---:|---:|---:|
| CI macOS ARM64 | 146.237 ms | 30.752 ms | 187.099 ms | 147.515 / 151.275 ms | -2.49% |
| CI Windows x64 | 102.107 ms | 30.803 ms | 74.254 ms | 126.251 / 105.641 ms | 19.51% |

Each series has 60 retained samples. The Windows search result already exceeds the
eventual 100 ms input-to-render budget before UI overhead; full desktop performance
remains a Phase 3 gate. CI hardware differs from §10. The available Parallels VM has
4 processors and 8 GiB RAM, also below the 16 GiB reference condition.

Run [38497753643](https://github.com/yuhui0531/localclip-windows/actions/runs/38497753643)
tested `6c34a4e`; both CI jobs passed. All raw samples are retained in
[its artifact directory](phase-1-ci-38497753643/).

| Host | Search P95 | Text ingest P95 | Image ingest P95 | First/warm P95 | Gap |
|---|---:|---:|---:|---:|---:|
| CI macOS ARM64 | 101.143 ms | 22.142 ms | 121.497 ms | 99.344 / 99.231 ms | 0.11% |
| CI Windows x64 | 131.572 ms | 21.956 ms | 56.983 ms | 158.814 / 128.682 ms | **23.42%, failed** |

This failed measurement is not discarded. CI success reflects build/test execution,
not the first-search performance target; the benchmark reports its pass/fail result.
The §13 Phase 1 exit is vectors, both CI jobs, and WAC16. §10 full desktop performance
and product acceptance of unresolved misses remain open for later phases.

Final build run [38498258081](https://github.com/yuhui0531/localclip-windows/actions/runs/38498258081)
tested `1072de2` with static Windows CRT and is green on both OSes.
[Windows WAC16/test summary](phase-1-ci-38498258081/windows-test-summary.txt).

| Host | Search P95 | Text ingest P95 | Image ingest P95 | First/warm P95 | Gap |
|---|---:|---:|---:|---:|---:|
| CI macOS ARM64 | 118.870 ms | 21.986 ms | 146.515 ms | 147.036 / 116.907 ms | **25.77%, failed** |
| CI Windows x64 | 102.028 ms | 29.310 ms | 178.628 ms | 123.998 / 106.061 ms | 16.91% |

Both new 60-sample series are retained. These variations are not a stable performance
pass across environments; §10 remains open as described above.

## Rule mapping

| Task | Implementation and evidence |
|---|---|
| P1-01–04; §4.1–4.3, §5.5, §9.2 | Rust workspace, neutral formats, canonical identity, offline link classification, literal search/parser; all vectors |
| P1-05; §5.13, §5.17 | Transforms/joins/inspector, Windows exports, grapheme-safe filenames and versions; vector/export tests |
| P1-06–08; §4.4–4.5, §5.6, §5.12 | WAL/FULL schema, dedicated writer/reader, quotas, settings revisions, categories, preview/confirm, 30-second undo; store/vector tests |
| P1-09–11; §8 WAC16, §10 | Synthetic generator, 50 forced kills per platform, retained nearest-rank measurements |
| P1-12–13; §12–13 | Dual-platform CI, report, read-only reference checks and separate PR |

## Approved decisions and implementation differences

- Launching the first CI utility in the Parallels guest failed with `0xC0000135`.
  PE inspection found a `VCRUNTIME140.dll` import and the guest has no VC runtime.
  Windows MSVC targets now statically link the C runtime via `.cargo/config.toml`,
  following the [Rust linkage reference](https://doc.rust-lang.org/stable/reference/linkage.html#static-and-dynamic-c-runtimes).
  This adds no dependency or VM installation. The rebuilt utility launched in the Windows 11 ARM64 guest and reached its expected
  missing-argument validation (exit 1), replacing the loader failure. PE imports
  contain only Windows system DLLs. [Launch evidence](phase-1-ci-38498258081/vm-launch.txt).
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
Intermediate repeats: empty. Stage-end check: empty (2026-10-14).

No Phase 1 implementation or executable exit checks remain. PR delivery is recorded
in the task checklist. All subsequent report-only commits preserve tested code `1072de2`.
Phases 2–4 have not started. No paste receipt, installer, update or release acceptance
is claimed.
