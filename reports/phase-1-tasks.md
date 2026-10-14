# Phase 1 tasks and exit gate

Baseline: Phase 0 `cafbe8d`, macOS `c097591`, Windows PRD revised 2026-10-14.
Reference stage-start status: `git -C /Users/ronny/VscodeProjects/rea status --porcelain` empty.
Phase 0 PR #1 is open; this branch builds on its verified commit. No merge was performed.

- [x] P1-01 — Rust ≥1.90 workspace and allowed dependency lockfile (§2–3).
- [x] P1-02 — Vector tests first; all 15 files and every case actually executed (§9.2). 440 cases, two explicitly approved filename overrides.
- [x] P1-03 — Neutral formats, unchanged image bytes, canonical identity, kind/webLink/image-file rules (§4.1–4.3; WAC02/17/19 core only).
- [x] P1-04 — NFC/ASCII normalization, literal rank/context, prefix parser and date bounds (§5.5; AC05, WAC18 core only).
- [x] P1-05 — Transforms, joins, inspection, export formats/names, version comparison (§5.13/5.17; WAC20/25).
- [x] P1-06 — Windows v1 schema, SQL functions, WAL/FULL/secure_delete, single serialized writer and separate reader (§4.4–4.5).
- [x] P1-07 — Ingest, dedup, quota rollback, disk-space gate, byte-preserving round trips (§4.5/5.6; AC03/04/13/17/25/26).
- [x] P1-08 — Expiry, protection, categories, batch touch, settings revisions, preview/confirm and undo (§5.6/5.12; AC12/19/20/27).
- [x] P1-09 — Deterministic dataset generator, seed 0xC11F, 10,000 texts + 100 images (§10).
- [x] P1-10 — Fifty forced child-process kills, integrity and atomic-record checks; Windows WAC16 (§8/13).
- [x] P1-11 — Search/ingest P95 with ≥60 samples and first-search-after-commit versus warm gap (§10).
- [x] P1-12 — fmt, clippy -D warnings, all core tests; green macOS and Windows CI (§12–13).
- [x] P1-13 — Phase report, clean reference end check, commit and separate PR (§13).

Exit requires every item above, 100% vector pass, both CI platforms green and WAC16 passed.
Do not begin Phase 2 while any exit condition is unfulfilled.

Approved: build-time downloads, Windows-safe filenames, macOS-compatible bare-link exclusions,
release defaults. Rust 1.90 is now installed. Local Parallels Windows 11 ARM64 guest command
access is verified. No Windows desktop acceptance checks have run yet.

Final executable revision: `1072de2`; dual-platform CI run 38498258081 green.
Windows WAC16: 50/50 forced terminations, integrity, atomicity and foreign-key checks.
Windows CRT issue fixed; utility entry point verified on the Parallels guest.
CI performance misses remain explicit in the phase report.

PR: https://github.com/yuhui0531/localclip-windows/pull/2 (stacked on Phase 0; not merged).
