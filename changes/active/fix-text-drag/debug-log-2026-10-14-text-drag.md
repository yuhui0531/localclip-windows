# Text drag-out to Notepad

Project: `/Users/ronny/VscodeProjects/localclip-windows`.
Change: `changes/active/fix-text-drag`.

- [x] Read the report and screenshot: Notepad reports a missing path when dragging a text history card.
- [x] Trace `commands::perform` → `win::drag::export_drag`.
- [x] High-confidence hypothesis: all non-file content is converted to a temporary file and offered only as CF_HDROP. `Temporary::drop` removes it immediately after SHDoDragDrop returns, before a deferred receiver opens it.
- [x] Challenge the root cause: an actual-file regression cannot verify temporary exported-content lifetime. Text insertion and file export are different receiving contracts; the user confirmed insertion into the current Notepad document.
- [x] Reproduce with a generated delayed receiver: no Unicode text format offered, exported file exists during Drop, missing 400 ms after Drop returns. [Evidence](evidence/before.json).
- [ ] Dedicated blank Notepad test: user closed existing windows; inventory confirms no Notepad or LocalClip process. Version is 11.2606.15.0 ARM64.
- [ ] Add protocol/lifetime tests, fix and build.
- [ ] Verify the exact binary and report target limitations.

The fix offers CF_UNICODETEXT for text and FILEDESCRIPTOR/FILECONTENTS backed by an owned COM stream for export; real files retain CF_HDROP. No personal document was inspected or modified. Microsoft recommends FILEDESCRIPTOR/FILECONTENTS for file-like data that does not already exist as a filesystem file: [Shell clipboard formats](https://learn.microsoft.com/en-us/windows/win32/shell/clipboard).

The generated test app/receiver were stopped after baseline capture. Build and actual target acceptance are still pending. Extending temporary-file retention would change the original privacy/retention rule, so no arbitrary delay or persistent temp cache has been introduced.
