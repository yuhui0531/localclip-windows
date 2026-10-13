# LocalClip for Windows

Local, private clipboard history for Windows 10/11, with a Rust core and Tauri v2 shell.
Implementation proceeds through the gated phases in the Windows PRD (2026-10-14 revision).

Behavior baseline: LocalClip macOS **0.3.29 (51)**, `macosCommit` **c097591431c00982ad34186068064b40965c95fd**.
The reference repository is read-only. No macOS history is read, imported, or migrated.

## Structure

- `vector-export/`: Swift executable depending on the reference `LocalClipCore` public library.
- `test-vectors/`: committed behavioral fixtures; CI will consume these without the reference checkout.
- `scripts/`: fixture validation and reproducibility checks.
- `reports/`: phase task lists, evidence, limitations, and PR descriptions.
- Planned subsequent phases: `core/` (Rust), `app/` (Windows/Tauri), `ui/` (React).

## Exporting the behavior baseline

Requires macOS 15+ and Swift 6. The default reference path is `../rea/macos-app`
relative to this repository. Override it with `LOCALCLIP_MAC_PATH`.

```sh
TZ=Asia/Shanghai swift run --package-path vector-export VectorExport test-vectors
python3 scripts/validate_vectors.py test-vectors
python3 scripts/check_phase0.py
```

The exporter checks the reference commit and clean status, creates synthetic databases in
its own temporary directory, removes them on exit, and never accesses the clipboard.
It writes only to the requested output directory, which must be inside this repository.
Build products and compiler caches belong to this repository. Generated UUIDs are mapped
to stable fixture aliases, retaining observed tie ordering explicitly where needed.

`check_phase0.py` runs two exports, checks the complete file bytes and fixture contracts,
and saves raw build output and SHA-256 values in ignored `reports/raw/`. It redirects SwiftPM
caches/configuration/security paths into `vector-export/.build/` for restricted environments.
Export requires access to the macOS system type registry (LaunchServices). A sandbox that
blocks this service makes even PNG/JPEG classification fail; the exporter rejects that
environment rather than publishing misleading macOS differences.

Each JSON document contains `metadata` and `cases`. Metadata identifies source functions,
commit, version, timezone, and any Windows adaptation. Platform-specific changes are explicit;
an exported macOS observation is not evidence of Windows runtime acceptance.
