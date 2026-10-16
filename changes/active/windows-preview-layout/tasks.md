# Card and preview layout correction

Scope: user's three reported layout problems in 0.1.5. No fourth item was specified.

- [x] Put custom/detected title above card content, preserving favorite/category indicators.
- [x] Expand native detail/preview workspace within current monitor; restore history size on close/reopen. Keep metadata and full text scrollable.
- [x] Show full-resolution image proportionally, initially fit, with fit/100%/zoom/pan controls and visible close button.
- [x] Verify short/narrow/high-DPI layouts, image bounds/zoom, OCR entry and existing menu behavior; native window lifecycle and drag/paste checks.
- [x] Build and install tested package with verified data backup; keep antivirus enabled.

Delivered: 0.1.6 application source `a5f44eab214e7b3a390f9066974db9a50735998a`. Eight browser layout cases, 14 browser menu cases, 11 native layout checks, 18 native menu/OCR checks and targeted drag/paste/cache/focus regressions passed. CI and both MSI builds passed. Installed ARM64 MSI with exit 0; installed version/hash and all six existing data files verified before normal launch. See `verification.md` and `reports/preview-layout-0.1.6.json`.
