# macOS visual alignment

Project root: `/Users/ronny/VscodeProjects/localclip-windows`.
Change directory: `/Users/ronny/VscodeProjects/localclip-windows/changes/active/windows-macos-ui`.
Baseline: `1d667fc`; macOS reference is read-only and was clean at entry.

Use current macOS HistoryView.swift (not the older documentation mockups): one 48/40 px toolbar, 8 px handle band, square viewport-sized cards, pastel kind headers with title/time and source fallback icon, copy/size metadata and quick keys below, neutral materials and blue selection/actions. Reuse the official pink mark. Match the settings tab bar and editor surface while retaining expanded Windows detail/image viewers.

- [x] Align toolbar, filters and square cards; preserve wheel, selection, drag, syntax preview and all context actions.
- [x] Align footer, menus, settings and editor surfaces; keep all existing settings and actions available at narrow widths.
- [x] Run existing UI regressions plus responsive/light/dark/compact visual checks on generated data; inspect screenshots.
- [ ] Build the final combined update and complete native test/data-preserving installation when the app is closed. Never disable antivirus or add exclusions.

No storage/schema/payload/native-output/network-policy changes. Windows shortcuts and controls remain Windows-native. Existing content editing and its slow-save focus correction are included. Do not read personal history or modify/build the macOS repository.

Character counts are derived from full text with Unicode grapheme segmentation; the existing representation, fingerprint and storage schema are unchanged. Source/date filters remain accessible through the filter popover. Scope and category actions keep their existing semantics. Full Windows details and image zoom/pan remain available.
