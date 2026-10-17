# Double-click copies instead of pasting

- [x] Read App.tsx Card.onDoubleClick → onAction → execute(), commands.rs perform_output, desktop.rs request_panel, and win/paste.rs focus restoration/SendInput.
- [x] High-confidence cause: onAction calls execute() without an explicit action. execute() falls back to settings.defaultAction. With “copy”, the native output path returns “copied” before restoring the target or sending Ctrl+V. This is a routing defect; direct paste functionality remains present.
- [x] Question chain: the current user request explicitly requires double-click paste. Fix the event's action and clicked ID; changing preferences, focus safety or the native paste engine would be unnecessary and could affect other operations.
- [ ] Browser failure before fix, then direct-paste dispatch with clicked ID; native generated-target receipt verified separately.

Existing header double-click rename and detail-button double-click isolation are intentional separate actions and remain. Enter/main button continue to follow the default action preference.

The generated browser regression reproduced the bug before the fix: a card-content double-click emitted action=copy while directPaste was required. Initial Ctrl-click fixture setup was changed to a synthetic ctrlKey event because macOS treats physical Ctrl-click as a context-menu action; the product action failure was then observed independently.
