Implement the platform-neutral Rust core required by Phase 1: byte-preserving clipboard representations, canonical fingerprints, offline URL rules, search and prefix parsing, transformations, inspection and Windows exports. History writes use serialized WAL/FULL transactions with deduplication, quota rollback, protection, categories, previews, settings revisions and bounded undo.

Validation: all 440 reference cases in 15 files, original-byte round trips, privacy rejection, concurrent WAL reads, export boundaries and 50 forced process kills. Windows and macOS CI run formatting, Clippy, tests and Release service benchmarks. All measured samples, including the initial failed cache-gap run, are retained.

[Phase report](https://github.com/yuhui0531/localclip-windows/blob/feat/phase-1-core/reports/phase-1.md) · [Task and exit checklist](https://github.com/yuhui0531/localclip-windows/blob/feat/phase-1-core/reports/phase-1-tasks.md)

This PR is stacked on Phase 0 (#1), so its diff contains only Phase 1. Neither PR is merged. Windows filenames and bare-domain exclusions follow the explicitly approved compatibility decisions. The reference macOS checkout remains unchanged.

The core utilities are not the Windows app. Real desktop paste, UI, installer and update acceptance belong to Phases 2–4; no such acceptance is claimed. CI service search latency exceeds the future full-UI budget and remains recorded for Phase 3 measurement.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
