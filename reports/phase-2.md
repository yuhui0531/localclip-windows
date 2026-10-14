# Phase 2 — Windows integration

Implementation revision: `9f73b24`. Based on Phase 1 PR #2, `182b11f`.

## Scope and gate decision

Native capture, eager clipboard output, checked foreground paste, file formats,
process identity, foreground WinEvent tracking, local WinRT OCR, runtime, tray,
autostart plugin, single-instance plugin and dynamically registered queue shortcuts
are implemented. The command-line demo requires an explicit synthetic data directory.
The Tauri shell has no history UI in this phase; that is Phase 3.

On 2026-10-14 the product owner approved: **“Build for my manual testing; defer
unavailable desktop gates.”** This permits subsequent implementation/build phases;
it does not count unexecuted desktop acceptance as passing. No production data was
read. The reachable Parallels Windows 11 VM has 4 CPUs/8 GiB, not the specified
16 GiB performance configuration. KeePassXC, Ditto, Chrome, Paint and Word were not
found in the checked installation paths/packages. Windows 10 is unavailable.

## Implementation mapping

| Tasks | Rules | Implementation |
|---|---|---|
| P2-01 | §4.1, §5.11 | UTF-16/CF_HTML byte offsets; UTF-16 and ANSI HDROP; COPY drop effect; retained original PNG/JPEG/DIBV5 representations, including coexisting file/image formats |
| P2-02 | W-CAP-1–9, §5.4 | Message-only window, 50 ms coalescing, 20/40/80/160 ms open retries, sequence validation, policy and four sensitive markers before body reads |
| P2-03 | §5.2, §5.10 | Private random receipt, sequence gate, eager HGLOBAL writes, no delayed owner data |
| P2-04 | §5.3 | Owner/foreground basis, lowercase exe identity, file description, ApplicationFrameHost child resolution |
| P2-05 | §5.8, §5.15 | Process creation identity and focus checks, elevation fallback, one foreground attempt, 150 ms limit, modifier release and SendInput; success is only commandSent |
| P2-06 | §5.9–10, §5.14 | Single-instance first; tray/pause, autostart plugin default off, main shortcut conflict handling; queue registration only while active, busy guard, skipped indexes, generation guard, 10-minute idle expiry |
| P2-07 | §5.16 | Local WinRT OCR, PNG/JPEG/DIB priority, 40 MP pre-decode check, configured language order, cancellation, copy-only output without persistence |
| P2-08 | §13 | localclip-demo capture / paste ID / ocr ID; metadata-only output |

A disposable same-executable reader process bounds each clipboard format read to
500 ms. Win32 GetClipboardData can block on an unresponsive delayed-rendering
owner, so a thread alone cannot enforce that bound. The private child pipe is
internal transport, not a frontend event or log. Failed reads are killed and never
stored. All representations count toward the 50 MiB aggregate limit.

Expiry runs at startup and once per minute; opening the panel expires first.
The visible-window resume hook belongs to Phase 3. Rust/Tauri dependencies are the
PRD-approved crates plus standard Tauri build tooling; no new clipboard library,
keyboard hook, fs/shell/http plugin or telemetry was introduced.

## Executed checks and evidence

- `sh scripts/cargo-local.sh test -p localclip-app --offline`: 3 format tests and
  1 queue test passed on macOS; Windows-only tests correctly execute zero cases there.
- `sh scripts/cargo-local.sh clippy --workspace --all-targets --offline -- -D warnings`:
  passed. Formatting completed with `cargo fmt --all`.
- The lone-CR CF_HTML test first failed, then passed after accepting both permitted
  header line endings. Byte offsets and fragments remain unchanged.
- Windows CI [38500465989](https://github.com/yuhui0531/localclip-windows/actions/runs/38500465989)
  passed core vectors/storage/WAC16, native process identity, format tests and all
  four synthetic sensitive-marker cases. Raw summary: [native tests](phase-2-evidence/ci-38500465989-native-tests.txt).
- `scripts/scan-private.py` requires a marked generated data directory and an external
  needle file; scans UTF-8 and UTF-16LE in overlapping chunks, including SQLite/WAL.
  It prints only counts and fails on matches or unreadable files. Cross-chunk positive
  control and a clean negative control passed locally. The Windows test also runs it.
- Final implementation CI [38502268546](https://github.com/yuhui0531/localclip-windows/actions/runs/38502268546)
  completed successfully on macOS and Windows. Windows workspace/shell Clippy, serial native
  tests, core vectors/WAC16, Release demo and Release Tauri shell builds all passed.
  The documentation-only head `8bee7ec` also passed both platforms in runs
  `38502413562` and `38502421968`.

## Deferred acceptance (not passed)

| Gate | Status |
|---|---|
| WAC01 | Synthetic four-marker filtering passed; real password-manager case unexecuted |
| WAC02 | Core raw-byte round trip and format codecs passed; Paint/Chrome/16-bit PNG desktop flow unexecuted |
| WAC04 | HDROP codecs tested; Explorer destination receipt unexecuted |
| WAC05–07 | Native checks implemented; elevated/closed/racing target and held-modifier desktop flows unexecuted |
| WAC10 | Eager writes implemented; exit then target-app receipt unexecuted |
| WAC11–13 | Bounded reads/receipt/shortcut logic implemented; busy-owner, Ditto coexistence and desktop conflict scenarios unexecuted |
| WAC23 | OCR implemented; image recognition/language-pack desktop cases and OCR text privacy scan unexecuted |

No headless demo paste log is claimed: no actual target-app paste was performed.
The Phase 1 raw performance results remain retained, including failed cold-cache
ratios. This phase has no measured GUI latency, idle CPU or WebView memory result.

## Read-only reference

`git -C /Users/ronny/VscodeProjects/rea status --porcelain` returned empty at
Phase 2 start and at the Phase 2 implementation/report checkpoint. No build,
formatter, database access or write was run in that repository.
