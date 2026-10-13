# Phase 0 — repository and behavioral vectors

Date: 2026-10-14 (Asia/Singapore).
Status: Phase 0 complete. Implementation commit: `8be5ab8`.
Pull request: [#1](https://github.com/yuhui0531/localclip-windows/pull/1), open against `main`.
Branch: `feat/phase-0-vectors`; base: remote `main` at `c59862d` (existing license preserved).

## Scope and baseline

Windows PRD: `PRD-Windows版-Rust-Tauri-2026-10-12.md`, revision 2026-10-14, read in full.
Product baseline: `PRD-macOS剪贴板管理器-2026-10-09.md`, read in full.
Reference: `/Users/ronny/VscodeProjects/rea/macos-app`, LocalClip 0.3.29 (51),
`c097591431c00982ad34186068064b40965c95fd`.
Export host: macOS 15.7.5 (24G624), arm64; Apple Swift 6.2.4; Python 3.12.3.

Only Phase 0 was implemented. No macOS history, clipboard, user data directory, or application
settings were accessed. All database inputs are generated; all temporary databases are removed.
No source was copied from `rea`, no submodule was created, and no build command ran inside it.
The Swift package links the local public `LocalClipCore` library. No new third-party dependencies.

## Task and rule evidence

| Task | Rules | Delivered evidence |
|---|---|---|
| P0-01–02 | §0, §3, §13 | Exact reference baseline checked; README, ignore rules, phase branch; remote license retained |
| P0-03 | §9.1 | Independent Python contract checker written and run before exporter; initial expected failure: missing 15 files |
| P0-04–05 | §4.1–4.3, §5.5, §5.12–5.13, §5.17, §9.1 | Public-API exporter and all 15 JSON files; metadata first in each file |
| P0-06 | §4.5, §5.6; AC03/12/13/17/19/20/26/27 | Seven ordered storage scenarios, every step carrying observed state, plus independent logical assertions |
| P0-07 | §9.1, §13 | Two final exports compared in full, 15/15 byte-identical; clean reference before/after |
| P0-08 | §13 | This report, task list, SHA-256 manifest, committed fixtures, and open PR #1 |

| Fixture | Cases | Coverage |
|---|---:|---|
| normalize | 15 | Chinese, full-width, combining characters, ASCII, emoji, NUL |
| rank | 17 | Public search ordering; exact/prefix/body/source/title matches, non-match, empty query, NFC, multi-item newline |
| context | 22 | Literal matching, grapheme windows, truncation, empty/no match, NUL |
| kind | 15 | Supported UTI-to-neutral mappings, rich/mixed/file/image/text/link |
| fingerprint | 22 | All eight neutral IDs, empty data/items, representation ordering, item ordering, synthetic multi-format image bytes |
| html | 14 | Tags, scripts/styles/head, comments, entities, line endings, malformed markup |
| policy | 18 | Sensitivity precedence, disabled capture, reliable/foreground/unknown source exclusion |
| store-scenarios | 7 | Dedup, protection, expiry, confirmation/cancellation, rollback, quotas, undo boundary, category batch touch |
| weblink | 50 | Above the 40-case minimum; schemes, bare domains, filenames, versions, whitespace, IP, ports, lengths |
| imagefile | 20 | Required extension whitelist, mixed case, invalid extensions, explicit macOS differences |
| query | 53 | All prefix types, aliases, Chinese, quoted literals, ambiguous/missing sources/categories, local dates, spacing |
| transform | 103 | All 25 transformations, at least four cases each, CRLF and invalid inputs |
| join | 35 | All seven styles × five input sets |
| inspect | 37 | Badges, full details, export format precedence, sanitized names and default export names |
| version | 12 | Prefixes, build suffixes, numeric comparison, equal/malformed inputs |
| **Total** | **440** | These are exported observations, not Rust or Windows acceptance results |

Storage scenarios exercise these observable results:

- AC03: duplicate ingestion preserves identity and increases the count; favorites/categories remain.
- AC12/19: ordinary entries become invisible at the exact expiry boundary; protected entries survive;
  re-copy before expiry refreshes the original entry, whose first-copy time is then over 30 days old.
- AC20/27: cancellation preserves data/settings; confirmation removes the affected ordinary history.
- AC13/17: insufficient quota rejects the new payload while preserving both protected and ordinary history.
  AC17 is a scaled logical fixture with an injected 512 KiB quota, not the exact 500 MiB acceptance setup.
- AC26: a fault after candidate writes/eviction but before commit retains the old item; retry without the
  fault admits the replacement and evicts the old item.
- AC27: lowering to 100 MiB evicts only ordinary items; if protected data remains over quota, capture stops.
- §5.6: undo succeeds at 29,999 ms and fails at 30,000 ms; batch touch keeps category order `b,a,c`.

## Commands and raw output summaries

Commands below were invoked through `rtk` as required by the workspace instructions.

1. `rtk proxy python3 scripts/validate_vectors.py test-vectors` before implementation:

   ```text
   AssertionError: Expected exactly 15 fixture files
   exit 1
   ```

2. Final reproducibility check: `rtk proxy python3 scripts/check_phase0.py`.
   The runner executes two real `TZ=Asia/Shanghai swift run --package-path vector-export ... VectorExport <output>`
   invocations. Extra options are `--cache-path`, `--config-path`, `--security-path`, `--manifest-cache local`,
   and `--disable-sandbox`; paths are all under `vector-export/.build/`. Clang/Swift module caches are redirected there too.
   Normal host service access was required for LaunchServices (see below).

   ```text
   Export 1: exit 0
   PASS: 15 files; provenance, minimum coverage, neutral fingerprints, required storage invariants
   Export 2: exit 0
   PASS: 15 files; provenance, minimum coverage, neutral fingerprints, required storage invariants
   PASS: 15/15 complete files byte-identical; reference clean before/after; temporary databases removed
   ```

   Final first build reported `Build of product 'VectorExport' complete! (1.27s)`.
   This is a build duration, not an application performance measurement.
   Full local logs: ignored `reports/raw/export-1.log`, `export-2.log`; reproducible hashes are committed
   in [phase-0-vectors.sha256](phase-0-vectors.sha256).

3. `rtk git diff --check`: exit 0, no output.
4. `rtk git -C /Users/ronny/VscodeProjects/rea status --porcelain`:
   empty at the initial stage check, after resuming, before and after both final exports, and at the final stage check.
   Saved before/after output files are both **0 bytes**. Reference HEAD remained `c097591431c00982ad34186068064b40965c95fd`.

The initial sandboxed export was rejected during review because `UTType` lookups reported PNG/JPEG as
unsupported. Running the same compiled exporter with normal host service access changed only
`imagefile.json` and restored correct PNG/JPEG observations. The exporter and validator now fail fast
if those baseline types are unavailable. Both final exports used working system services; only HEIC
differs from the Windows whitelist (`macosIsImage=true`, `isImage=false`, `macosDiffers=true`).

## Fixture semantics and deviations

- JSON envelopes use `metadata` and `cases`. Source function names, commit, version, timezone, fixed time,
  and offset are recorded. Rust consumers must dispatch `inspect` cases by `operation`.
- Random database IDs are mapped consistently to fixture aliases. Global equal-time results use
  `orderGroups` with the explicit rule `actual-id-ascending`; category ordering is preserved exactly.
  This prevents random UUIDs from making the export nondeterministic without changing reference behavior.
- Fingerprints are recomputed by the public canonical/fingerprint API **using neutral IDs**. Python
  independently reconstructs the prescribed big-endian canonical bytes and verifies every SHA-256.
- No TIFF-to-DIB conversion is performed. DIBV5 fingerprint cases use synthetic neutral-ID bytes.
  Image classification on the macOS side only covers actual mapped macOS formats.
- Policy observations map macOS `declared` to Windows `owner`. Sensitive-marker inputs retain their
  macOS names; the four Windows marker predicates require Win32 adapter tests in Phase 2.
- macOS quota failures are exported as `quotaExceeded`; the prescribed Windows result is
  `skipped(quotaFull)`. This mapping must be explicit in the future Rust vector consumer.
- Reference `CommitStrategy.swift` already uses in-place transactions with DELETE journaling. The
  PRD's APFS-clone description is stale; the explicit Windows WAL requirements still take precedence.
- macOS `sanitizedFileName` preserves backslashes, Windows-invalid characters and reserved names.
  The fixture records the actual result. Windows-safe filename behavior needs a product decision before
  the corresponding implementation; no alternate sanitization policy was invented in Phase 0.
- Timestamp badge presentation uses the reference API's actual-current-year comparison; full details
  use fixed `nowMs`. The selected timestamp cases are from 2023. Date and natural-sort behavior depend
  on Foundation/system services, so host/tool versions are recorded above.
- Pure-text newline joins are covered; rich-text composition, `.url` export and DIBV5/BMP export remain
  Windows-specific implementation tests. Synthetic image bytes here are identity/quota inputs, not decoder fixtures.

## Unexecuted work and next gate

Performance measurements are not applicable to Phase 0. Rust tests, crash kills/WAC16, Windows runtime
tests, dual-platform CI, frontend, installers, signing and updates were **not executed**. No Windows
acceptance result is claimed. The exact AC17 500 MiB fixture remains a later acceptance check.

Next is Phase 1, only after this Phase 0 delivery is complete. This task does not begin it.
No human decision is required to finish Phase 0. PRD §15 decisions and the Windows filename issue
remain recorded for subsequent phases.
