# Text drag-out to Notepad

Project: `/Users/ronny/VscodeProjects/localclip-windows`.
Change: `changes/active/fix-text-drag`.

- [x] Read the report and screenshot: Notepad reports a missing path when dragging a text history card.
- [x] Trace `commands::perform` → `win::drag::export_drag`.
- [x] High-confidence hypothesis: all non-file content is converted to a temporary file and offered only as CF_HDROP. `Temporary::drop` removes it immediately after SHDoDragDrop returns, before a deferred receiver opens it.
- [x] Challenge the root cause: an actual-file regression cannot verify temporary exported-content lifetime. Text insertion and file export are different receiving contracts; the user confirmed insertion into the current Notepad document.
- [x] Reproduce with a generated delayed receiver: no Unicode text format offered, exported file exists during Drop, missing 400 ms after Drop returns. [Evidence](evidence/before.json).
- [x] Dedicated blank Notepad test: user closed existing windows; inventory confirms no Notepad or LocalClip process. Version is 11.2606.15.0 ARM64.
- [x] Add protocol/lifetime tests, fix and build x64 and ARM64 installers.
- [x] Verify the exact binary and report target limitations.

The fix offers CF_UNICODETEXT for text and FILEDESCRIPTOR/FILECONTENTS backed by an owned COM stream for export; real files retain CF_HDROP. No personal document was inspected or modified. Microsoft recommends FILEDESCRIPTOR/FILECONTENTS for file-like data that does not already exist as a filesystem file: [Shell clipboard formats](https://learn.microsoft.com/en-us/windows/win32/shell/clipboard).

The generated test app/receiver were stopped after baseline capture. Installer builds and the specific desktop regressions below are complete. Extending temporary-file retention would change the original privacy/retention rule, so no arbitrary delay or persistent temp cache has been introduced.


## Contract and implementation

The user explicitly confirmed that dragging text into Notepad must insert the body
into the current document. Dragging must remain available. The previous CF_HDROP-only
implementation conflated content with existing files and treated return from
SHDoDragDrop as proof that a deferred target had already opened its path.

`app/src/win/drag.rs` now publishes Unicode text for text-capable targets and Shell
virtual-file descriptors plus an owned memory stream for file-capable targets.
Existing file cards still publish their original CF_HDROP paths. Export extensions
and bytes still come from `core::export::prepare`, including code, URL, rich text and
original images. The receiver chooses the applicable format. There is no app-name
special case, simulated paste, fixed retention delay, or temporary export directory.

This replaces PRD §5.13's transient export-file mechanism with memory-backed virtual
files; it preserves file export without extending the lifetime of clipboard content
on disk. LocalClip never deletes an exported destination file owned by the receiver.
Cancellation does not create an export path. No dependency or capability was added.

Windows protocol tests cover Unicode (including CJK, supplementary characters and
line breaks), no synthetic CF_HDROP path, data-object ownership after the payload is
dropped, empty stream export, code/URL/image bytes and original-file paths. Actual
Notepad/Explorer receipt is a separate acceptance step.

## Validation notes

- Local workspace tests, Clippy and frontend build passed.
- Initial Windows compilation caught that windows-rs FD_FLAGS has no BitOr impl;
  combining its raw flag values fixed the compile error. Both Windows Clippy steps
  subsequently passed. This was a build failure, not a target-app pass.
- Final [CI 38534274285](https://github.com/yuhui0531/localclip-windows/actions/runs/38534274285) passed Windows and macOS, including formatting, both Clippy configurations, core tests, Windows adapters, desktop tests, release builds and benchmarks. [Test excerpt](evidence/ci-checks.txt). Benchmark completion is not PRD performance acceptance.
- The first protocol test run used HRESULT::is_err for absent formats; Windows Shell returns S_FALSE (1), not a failing HRESULT. A native empty-data-object [probe](evidence/query-format-probe.json) confirms this. Tests now require S_OK (0) for availability. Commit 93b979f changes only tests, not shipping code.

## Scope review

The change fixes the missing-path cause by removing synthetic filesystem paths and
supplies the text contract requested by the user. Actual-file dragging uses the
previous path protocol. Changes are restricted to the drag adapter, its protocol
regressions and the displayed/package version. Existing wheel, syntax highlighting,
smart-paste and focus behavior are retained; the specific runtime regression results are recorded below.


## Packages and desktop test status

Both MSI builds succeeded in [38533089887](https://github.com/yuhui0531/localclip-windows/actions/runs/38533089887)
for production code `3f2600d42f11045bd7806380b9a38df4b907355b`. Both downloaded MSI
hashes match workflow metadata; their standalone binaries have the expected PE
architectures. The ARM64 executable downloaded for VM testing matches the executable
in the ARM64 installer artifact. [Artifact checks](evidence/artifacts.json).

[MSI metadata](evidence/msi-metadata.json) confirms LocalClip 0.1.3, Arm64/x64,
no default history-deletion opt-in, and deletion gated on explicit full uninstall
outside an upgrade. Both packages are unsigned; automatic installation remains disabled.

| Package | Bytes | SHA256 |
|---|---:|---|
| LocalClip_0.1.3_arm64.msi | 9,805,824 | `070560e012583abc902e5164055fa67818adef0d11976514deb502e8d6275665` |
| LocalClip_0.1.3_x64.msi | 10,969,088 | `2b31d48df6e3e6b0b20bd6f34183aa3f63564be6f01373b3108fd91e8976dd37` |

Delivery directories: `dist/manual-test-0.1.3/arm64` and `dist/manual-test-0.1.3/x64`.
No personal installation has been overwritten.

## Actual ARM64 target receipt

The exact delivered ARM64 executable was tested with generated, capture-paused
history. The regular app reopened during preparation, so the isolated launcher
refused to start. After the owner closed it again, the isolated instance launched
with the verified SHA256 above. No personal history was read or overwritten.

- **Notepad 11.2606.15.0:** physical text drag inserted the complete generated body
  into the same existing document. Removing that inserted substring left the exact
  pre-drop text (after line-ending normalization); no exported-file tab was opened.
  [Receipt](evidence/notepad.json). Initial harness checks assumed fixed line-ending
  behavior and an uninterrupted document; the final run uses a fresh generated tab,
  the uniquely visible editor and its actual pre-drop value.
- **Explorer:** in a fresh empty generated folder, Escape cancellation produced no
  file, text drag created `LocalClip.txt` with exact contents, and an existing file
  card copied `sample.rs` with an identical hash while preserving the source.
  [Receipt](evidence/explorer.json). An earlier cancellation attempt found an
  output file without an empty-before snapshot and was inconclusive; the fresh-folder
  run above is the cancellation evidence.
- **Smart code-file paste:** seven controlled cases passed: off, Copy, enabled text
  control, unknown control, return to text control, read-only and Hide Contents.
  The two enabled writable cases received all 68 source characters; fallback cases
  retained original file formats. Wheel scrolling advanced 240 pixels and the
  scrollbar became transparent after idle. [Receipt](evidence/smart-paste-wheel.json).
- **Highlighting:** the native WebView rendered two code previews and ten colored
  tokens; Hide Contents removed them and disabling it restored them.
  [Receipt](evidence/highlight.json).
- **Focus:** unpinned outside-click hiding, pinned visibility and post-drag hiding
  passed in the controlled receiver. That receiver attempt did not receive its file,
  so it is **not** counted as file delivery evidence; the subsequent Explorer
  receipt above verifies actual file delivery. [Attempt](evidence/focus-attempt.json).

The test app was stopped and test Explorer windows closed after verification;
Notepad's temporary topmost setting was removed without closing other documents.
The personal installed executable was not replaced. Other receiving apps remain
outside this specific regression run.

The broader deferred desktop matrix, Windows 10 lifecycle, signed updates and PRD
performance gates remain open. No release, merge, or full acceptance is claimed.
