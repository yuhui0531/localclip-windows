# Double-click copies instead of pasting

- [x] Read App.tsx Card.onDoubleClick → onAction → execute(), commands.rs perform_output, desktop.rs request_panel, and win/paste.rs focus restoration/SendInput.
- [x] High-confidence cause: onAction calls execute() without an explicit action. execute() falls back to settings.defaultAction. With “copy”, the native output path returns “copied” before restoring the target or sending Ctrl+V. This is a routing defect; direct paste functionality remains present.
- [x] Question chain: the current user request explicitly requires double-click paste. Fix the event's action and clicked ID; changing preferences, focus safety or the native paste engine would be unnecessary and could affect other operations.
- [x] Browser failure before fix, then direct-paste dispatch with clicked ID; native generated-target receipt verified separately.

Existing header double-click rename and detail-button double-click isolation are intentional separate actions and remain. Enter/main button continue to follow the default action preference.

The generated browser regression reproduced the bug before the fix: a card-content double-click emitted action=copy while directPaste was required. Initial Ctrl-click fixture setup was changed to a synthetic ctrlKey event because macOS treats physical Ctrl-click as a context-menu action; the product action failure was then observed independently.

Final package verification: 27 menu/editor and 13 history/settings browser checks, full Windows/macOS CI and both installer builds passed. The exact packaged ARM64 executable passed 15 native checks using generated data and a uniquely named Notepad document. Actual mouse double-click inserted the entire Chinese multiline/code body exactly once between existing text, restored the original target and hid the panel for both copy and directPaste defaults. Enter retained copy-only behavior, and a physical text-card drag inserted once without changing existing text.

Native harness setup initially stopped on a CDP/Notepad variable collision, an ambiguous fixture search, a terminal obscuring the generated target, and a hidden-window handle cached as zero. The harness now separates target variables, uses uniquely titled generated fixtures, activates only the guarded generated Notepad window, and obtains the panel handle after showing it. These were test-harness corrections; the packaged application was unchanged. Input/content guards remained enabled. The final complete run passed all 15 checks.

Installed 0.1.14 in place after a verified backup of all six existing history/settings files. MSI exit 0, installed executable hash and registry version matched the tested build; all original files were hash-identical before normal startup. Defender scans passed for both MSI/EXE pairs with antivirus/realtime enabled; no exclusions or security changes. This is not a 360 rescan or a rerun of the broader external target-app OCR/smart-paste matrix. Exact evidence: `reports/double-click-0.1.14.json`.
