Implement the Windows adapter for LocalClip: privacy-filtered clipboard capture,
lossless native format output, checked foreground paste, local OCR, and the Tauri
runtime with tray, shortcuts and paste queue. Clipboard reads have a killable
500 ms boundary; events and demo logs contain metadata only.

This PR is stacked on Phase 1 PR #2. It does not include the history UI.

Validation: macOS workspace Clippy, portable format/queue tests; Windows serial
marker filtering and source-identity tests; core vectors and 50 forced-crash checks.
Final Windows shell build is tracked in the report. Desktop target-app acceptance
and OCR privacy evidence remain explicitly unexecuted under the product owner's
approval to deliver a build for manual testing.

[Phase report](https://github.com/yuhui0531/localclip-windows/blob/feat/phase-2-windows/reports/phase-2.md)
· [Task mapping](https://github.com/yuhui0531/localclip-windows/blob/feat/phase-2-windows/reports/phase-2-tasks.md)

🤖 Generated with [Claude Code](https://claude.com/claude-code)
