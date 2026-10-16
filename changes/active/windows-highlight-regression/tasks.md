# Syntax highlighting regression

- [x] Trace history cards to 180-character summaries; read bounded original text for visible text cards while preserving search excerpts and privacy guards.
- [x] Reproduce missing syntax on long detail text and HTML-backed code.
- [x] Preserve bounded highlighting together with complete escaped text; prefer code rendering for recognized code in Details.
- [x] Verify 15 focused cases and 11 browser layout cases, including actual token colors and full text equality.
- [x] Recheck menus; build 0.1.7 installers and run Windows verification.
- [x] Install after the personal app is closed, preserving history and settings.

User confirmed the problem is on history cards. The exact language/sample is not supplied. Short ambiguous snippets retain the macOS relevance threshold; no claim that these two confirmed cases explain every missing-highlight report.

Delivered 0.1.7: CI/installers succeeded; 10 native checks and hide/restore passed; four Defender scans passed; exact ARM64 MSI installed with six existing data files unchanged. See verification.md and reports/highlight-0.1.7.json.
