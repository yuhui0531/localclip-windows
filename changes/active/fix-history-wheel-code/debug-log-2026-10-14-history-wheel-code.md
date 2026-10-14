# History wheel scrolling and code-feature preservation — 0.1.2

Project: `/Users/ronny/VscodeProjects/localclip-windows`  
Change: `changes/active/fix-history-wheel-code`  
Code revision: `6c84f446e598a1468ca96988f9dbe00f71349457`.

## Request and scope

The owner requested ordinary mouse-wheel history navigation, an idle-hidden scrollbar,
and preservation of syntax highlighting and smart code-file paste. Windows PRD §5.6
had explicitly deferred `smartCodeFilePaste` to P1. This request brings that feature
into the current manual-test build; it is not claimed to have existed in 0.1.1.
Reference behavior was read from the immutable macOS baseline
`c097591431c00982ad34186068064b40965c95fd`, never from changes in its working tree.

## Diagnosis and completed work

- [x] Trace history scroll, pagination, virtualization, and scrollbar styles.
  High confidence: horizontal overflow had no vertical-wheel translation; the thumb
  was always colored. This is an input-handling omission, not missing history data.
- [x] Add a non-passive wheel listener only on the history row. Vertical pixel,
  line, and page deltas move horizontally; existing horizontal input remains native.
  Ctrl/Meta gestures are not intercepted. Existing `scroll` handling continues to
  drive pagination and virtual cards.
- [x] Keep scrollbar space stable; make the idle thumb transparent after 800 ms.
  Reveal it on scrolling or when pointing at its bottom hit area. Forced-color
  mode retains system scrollbar visibility for accessibility.
- [x] Fix repeated All history selection: clone the filter value so clearing cards
  always triggers a reload, including when the initial all-history filter is selected again.
- [x] Audit syntax highlighting against `ContentInspector.codeExtensions`.
  The existing 19-language set, local highlight.js bundle, Swift registration,
  cache, hidden-content guard and escaped markup remain intact. No network syntax
  service or file-content indexing was introduced.
- [x] Add an explicit `smartCodeFilePaste` setting, default **off**, including
  backward reading of old settings without that field. The General settings page
  explains its target restrictions and size limits.
- [x] Keep original files for copy/drag, file managers, file dialogs, unknown,
  disabled, password and read-only targets. Only direct paste into a confirmed
  editable target converts all selected supported files into source text.
- [x] Strict decoding: UTF-8 (optional BOM), or BOM-marked UTF-16/32, both endian
  variants. Preserve whitespace/line endings/order; join files with one newline.
  Reject invalid encodings, binary control characters, unsupported types, missing
  files, direct reparse points, UNC/device/alternate-stream paths, more than 64
  files, over 4 MiB per file or 8 MiB combined. Conversion is all-or-nothing;
  fallback retains file output and existing missing-file reporting still applies.
- [x] Hold Windows file handles without write/delete sharing until output completes;
  compare file identity/size/mtime and reopen original paths before conversion.
  Revalidate settings/privacy, target process/focus/control and clipboard ownership.
  Compare clipboard sequence and owner **inside OpenClipboard** before replacing/restoring;
  a newer user copy is never overwritten by a stale conversion. The owner check
  also covers a different application taking ownership just before the previous
  write receipt captured its sequence number.
- [x] VM capability probe exposed standard WinForms multiline Edit as a generic
  UIA Pane. Native Edit/RichEdit window styles now provide a conservative fallback.
  Other editable UIA controls use ValuePattern or TextPattern read-only metadata;
  no target text is read. [Microsoft UIA timeout interface documentation](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nn-uiautomationclient-iuiautomation2).

## Verification

- Local TypeScript/Vite build, Rust formatting and workspace Clippy pass.
- Local core suite passes: 440 reference vectors across 16 test functions,
  10 storage tests, export tests, and 50 forced-termination cycles.
- Portable code-file tests cover supported names, strict UTF-8/16/32, invalid data,
  exact text preservation and old/default/new settings parsing.
- Browser regression passes: vertical wheel 240 px; native horizontal then reaches
  360 px; line-mode reaches 424 px; page/reverse/boundary/Ctrl gestures, pagination,
  18 virtual cards, repeated All history reload, idle-transparent scrollbar, syntax token rendering with exact
  source text and escaped HTML, and saving the smart-paste toggle. No page errors.
  [Browser evidence](evidence/browser.json).
- Windows native adapter and desktop unit-test steps pass, including real file
  handle write/delete exclusion, exact multi-file text, 4/8 MiB limits, binary and
  mixed-group rejection, native writable/read-only/password/disabled classification,
  and clipboard sequence/owner replacement guards. Final Windows/macOS
  [CI 38528252650](https://github.com/yuhui0531/localclip-windows/actions/runs/38528252650)
  and both [installer builds 38528252879](https://github.com/yuhui0531/localclip-windows/actions/runs/38528252879)
  pass at the code revision above. [CI evidence](evidence/ci.json),
  [installer evidence](evidence/installers.json).
- After the owner closed the personal instance, the final ARM64 executable ran
  with generated history, capture paused and a blank generated receiver. Seven
  cases passed: option off, Copy, option on, non-text target, returning to the text
  control, read-only control and Hide Contents. Both conversion cases delivered
  all 68 generated characters exactly; the other cases retained FileDrop with no
  plain-text representation. Receipt was checked separately from `commandSent`.
  [Native paste/wheel evidence](evidence/native.json).
- Native WebView2 wheel input moved history by 240 px, rendered 17 virtual cards,
  and returned to an idle-transparent scrollbar. Re-selecting All history retained
  the records after the filter fix.
- The runtime investigation found two focus issues: cached foreground events could
  retain a stale child control, and top-level activation preceded child focus.
  The tracker now captures the current external control before opening history;
  paste waits for both original handles within its existing 150 ms deadline.
  Target detection reads only control metadata, never target text.
- Final generated-file drop and focus tests: identical bytes received, source
  preserved, unpinned panel hidden on outside click, pinned panel visible, and
  panel hidden again after drag. Escape cancellation created no destination file.
  [Drop/focus evidence](evidence/drag-focus-code-0.1.2.json),
  [cancellation evidence](evidence/drag-focus-code-0.1.2-cancel.json).
  The native input harness holds Escape for 180 ms so the Windows drag loop can
  observe it; an earlier zero-duration key pulse was a harness timing failure.
- The smart-paste preference persisted across restarting the generated instance;
  capture remained paused and Hide Contents remained off. [Persistence evidence](evidence/persistence.json).
  The isolated app and receiver were then stopped; the local UI server was stopped
  and the VM's original idle-pause setting was restored.

## Manual-test packages

Both downloaded MSIs match workflow SHA256 metadata. The standalone PE machines
are ARM64 `0xaa64` and x64 `0x8664`; the ARM64 executable hash matches the exact
VM-tested binary. [Artifact verification](evidence/artifacts.json).
The final MSIs were inspected, not installed in this turn. Product version and
architecture match; default history deletion is unset, and deletion requires
explicit opt-in during full uninstall, excluding upgrades. [MSI metadata](evidence/msi.json).

| Package | Bytes | SHA256 |
|---|---:|---|
| LocalClip_0.1.2_arm64.msi | 9,809,920 | `61a56aca584a3e90d0d9f8ce5f55f5b4d4a7eca96405faa78f576d93847a5479` |
| LocalClip_0.1.2_x64.msi | 10,977,280 | `7c2be6307e25df2b4dd4bdc55069c699bfd45110f557a46bffc4b0f4d049f214` |

Local delivery: `dist/manual-test-0.1.2/arm64/` and `dist/manual-test-0.1.2/x64/`.
Use ARM64 for the owner's Apple Silicon Parallels VM. Both are unsigned; automatic
installation remains disabled. No tag, public release or merge was performed.

## Limits and self-review

This remains an unsigned manual-test build with automatic installation disabled.
An editor/browser without a verifiable writable native/UIA text control falls back
to original files; no application-name-only guess or custom editor opt-in bypass is
provided. Full Word/browser/chat/file-manager acceptance remains open. The prior
performance, clean-machine lifecycle and signed-update gates remain open.
The macOS repository and personal Windows history were not modified or inspected.

Self-review: changes address the requested input behavior and missing feature;
no storage schema change, source-body persistence, new runtime dependency, UI
redesign or production release. Drag and focus protections from 0.1.1 are retained.
