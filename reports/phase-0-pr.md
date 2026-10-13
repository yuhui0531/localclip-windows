Establish the behavioral baseline needed to implement LocalClip for Windows against macOS 0.3.29 (`c097591`). This adds the repository setup, a Swift exporter using only public LocalClipCore APIs, and all 15 committed fixture files (440 cases).

The validator independently checks neutral-format SHA-256 fingerprints, coverage requirements and storage invariants. Two final exports were byte-identical; `rea` remained clean and generated databases were removed. The exporter rejects unavailable macOS image type services, avoiding misleading sandbox-derived fixtures.

[Phase report](https://github.com/yuhui0531/localclip-windows/blob/feat/phase-0-vectors/reports/phase-0.md) · [Task checklist](https://github.com/yuhui0531/localclip-windows/blob/feat/phase-0-vectors/reports/phase-0-tasks.md)

Validation: `python3 scripts/check_phase0.py`; `git diff --check`. Exported 50 web links, 103 cases across all 25 transforms, seven join styles and seven storage scenarios. No new third-party dependency, source copy, submodule, or user-history access.

Scope is Phase 0 only. Rust/Windows tests and CI were not executed. The report identifies scaled AC17 coverage, explicit Windows format/policy adaptations, the stale PRD transaction description, and the unresolved Windows filename policy. Existing remote LICENSE is preserved.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
