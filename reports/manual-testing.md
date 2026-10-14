# Manual acceptance checklist

Use generated content and newly opened blank targets only. Never test a paste in a
real conversation, mail draft or document. Record actual receipt separately from
LocalClip's commandSent result. All boxes below are unexecuted unless linked evidence
in the phase report says otherwise.

- [ ] Win10 22H2: install, first launch, tray/hotkey, capture, search, copy/paste, restart, uninstall.
- [ ] Clean Win11 x64 and ARM64: same lifecycle. WebView2 absence/bootstrapper where applicable.
- [ ] WAC01: password-manager markers and four synthetic markers; run `scripts/scan-private.py` against a generated directory and external needles.
- [ ] WAC02/03/04: Paint/Chrome/16-bit PNG original byte retention, Word bold+newlines, Chinese/space Explorer file COPY.
- [ ] WAC05–08: elevated/closed/racing targets and held modifiers; receipt in blank Notepad, Word, VS Code, Chrome address/input, Edge, Terminal, Explorer and empty chat.
- [ ] WAC09: mouse-selected monitor and mixed 100%/150% DPI.
- [ ] WAC10–13: paste after exit, busy clipboard recovery, Ditto coexistence, shortcut conflict with tray fallback.
- [ ] WAC14/15: 8-hour sleep/expiry, Chinese+space username storage.
- [ ] WAC17–20: link kinds, prefix errors/quoted literals, image-file intersection, transforms/joins without extra history.
- [ ] WAC21/22: queue order/deleted item/modifier timeout, pinned recent-target paste (post-output hide discrepancy documented).
- [ ] WAC23: local bilingual OCR and missing-language prompt; scan history/WAL for OCR output.
- [ ] WAC25: Swift/Markdown/RTF/URL/DIBV5 save-as and collision numbering; physical drag drop/cancel and temporary-file cleanup.
- [ ] AC01–27: execute the complete inherited cases, especially protected quota/expiry previews, cancellation and undo.
- [ ] Uninstall: default retains generated history; explicitly checked removal deletes current user's history only; backups remain.
- [ ] WAC24 (keys required): disposable signing keys and test releases; correct signature installs only after explicit click and snapshot; tampered, wrong-architecture and downgrade packages rejected; failed startup permits manual recovery.
- [ ] §10 performance: required 4 CPU/16 GiB/NVMe Win11 hardware and specified dataset; ≥60 samples per latency, 10-minute idle CPU, memory after 5 minutes hidden.

Record: OS/build/architecture, app commit+SHA256, exact steps, expected result, actual
receipt or error, pass/fail/unexecuted, and screenshot names. No real clipboard body,
file paths, search terms or OCR text should be included in diagnostic logs.

## 0.1.2 input and code-feature checks

- [ ] Scroll history with a normal vertical mouse wheel, reverse direction, then
  continue past the first loaded page. Confirm horizontal trackpad input still
  works, the scrollbar disappears after idle and reappears at its bottom hit area.
- [ ] Copy a generated code snippet: confirm colored tokens and unchanged source
  text. Enable Hide Contents and confirm preview contents disappear.
- [ ] In Settings → General, enable **代码文件智能粘贴** (off by default). Copy a
  generated `.rs`/`.py`/`.json` file, then directly paste its history card into a
  new blank text editor. Verify the complete source body and original line endings.
- [ ] Repeat with the option off, Copy instead of Direct Paste, a file-management
  target, a read-only control and drag-out: the original file must remain a file.
  Unknown editor/browser controls intentionally fall back to original files.
- [ ] In a generated fixture, try a mixed supported/unsupported group, invalid
  encoding, a missing file and an over-limit file. No partial source conversion.
- [ ] Restart LocalClip and confirm the smart-paste setting persists; existing
  history and preferences must remain available. Broader target coverage remains
  manual even when the controlled receiver tests pass.
