# Phase 3 — Windows history UI

Base: Phase 2 PR #3. Windows build verification is in progress.

## Scope and approved acceptance deferral

The owner approved continuing implementation and delivering a manual-test installer
while unavailable desktop gates remain unexecuted. Synthetic browser results below
are frontend checks, not Windows desktop acceptance or proof of target-app receipt.

| Task | Rules | Implementation |
|---|---|---|
| P3-01 | §5.5/5.13, §7 | Core metadata/read APIs, prefix/filter intersection, original output composition, transforms and joins |
| P3-02 | §6/7 | Explicit IPC allowlist, async blocking workers, metadata-only events; update commands belong to Phase 4 |
| P3-03 | W-UI-1–4, §5.15 | Precreated bottom panel, work-area/DPI positioning, focus memory, hide/pin; non-focusable feedback window |
| P3-04 | W-UI-5/6/8/9/12 | Virtual cards, multi-selection, keyboard, 20 ms search debounce, stable refresh, prefix status and filters |
| P3-05 | W-UI-7/10/11/15/16 | Custom thumbnail protocol, 16 MiB Rust cache, safe HTML sandbox, local highlighting, themes and hidden-content mode |
| P3-06 | W-UI-13/14, §5.13–16 | Details, transform menu, seven merge choices, save dialog, inspector, queue and OCR cancellation/feedback |
| P3-07 | §5.6/5.12 | Settings, exclusions with exe/running-app picker, categories, storage preview/confirmation, protected content, 30-second undo |

No cloud resources, telemetry, remote fonts, clipboard plugin or broad filesystem/
shell/HTTP permissions were introduced. UI dependencies are the PRD's React,
TypeScript, Vite, Tauri API/CLI and highlight.js; the latter's license is retained.

## Executed checks

- `npm --prefix ui run build`: TypeScript and Vite production build passed.
- `sh scripts/cargo-local.sh fmt --all --check`: passed.
- Windows run 38505776476 passed both Clippy checks, core vectors/storage and serial
  native adapter tests. Its old frontend hook failed because Tauri resolves the
  frontend working directory differently from its Rust crate. Explicit hook cwd
  fixes that issue in 837afc3; the replacement run is 38506436983.
- Synthetic Chromium/Playwright checks: generated 120 records, 8 initial DOM cards,
  18 DOM cards after virtual scrolling, no page errors. Search, detail and storage
  views, light/dark themes and the minimum 220 px panel were exercised. Evidence:
  [result](phase-3-evidence/result.json), [light](phase-3-evidence/panel-light.png),
  [dark](phase-3-evidence/panel-dark.png), [minimum](phase-3-evidence/panel-minimum.png),
  [details](phase-3-evidence/detail.png), [storage](phase-3-evidence/storage.png),
  [search](phase-3-evidence/search.png).
- The test harness injects Tauri's official mock IPC into the preview browser only;
  shipped sources contain no fixture/mock switch. A separate generated SQLite
  fixture tool leaves capture paused for native VM testing.

## Acceptance ledger — not passed by implementation alone

| Checks | Current evidence / remaining gate |
|---|---|
| AC01–05 | Core scenarios/vectors pass; full copy/restart desktop sequence unexecuted |
| AC06–11 | Native output/filter/receipt/pause implementation; target-app desktop cases unexecuted |
| AC12–15 | Core retention/protection/delete/failure tests; UI desktop flows unexecuted |
| AC16 | Service measurements retained in Phase 1; full GUI/resource performance unexecuted |
| AC17–20 | Core quota/policy/expiry/confirmation scenarios pass; desktop flows unexecuted |
| AC21–24 | Rich output, sensitive markers, shortcut conflict and focus guards implemented; full desktop matrix unexecuted |
| AC25–27 | Transaction/usage/protected preview logic tested; full desktop confirmation/recovery unexecuted |
| WAC03 | Word rich/plain actual receipt unexecuted |
| WAC08 | Notepad, Word, VS Code, Chrome address/input, Edge, Terminal, Explorer, WeChat/Feishu: all actual receipt unexecuted |
| WAC09 | Mixed 100%/150% DPI multi-monitor test unexecuted |
| WAC14 | Expiry-on-open/minute/resume implemented; 8-hour sleep desktop test unexecuted |
| WAC15 | Chinese/space Windows username clean install unexecuted |
| WAC17–20 | URL/query/image-file/transform/join vectors and filter intersection pass; real application copying flows unexecuted |
| WAC21 | Queue native state logic tested; target receipt/delete-mid-queue/modifier timeout unexecuted |
| WAC22 | Pinned target tracking implemented; actual Word receipt unexecuted; visibility conflict below |
| WAC25 | Export bytes/filename core tests pass; five-format save-dialog desktop flow unexecuted |

## Complete §10 performance table

Required environment: Windows 11 23H2+, 4 cores / 16 GiB / NVMe, Release,
10,000 texts + 100 images, seed 0xC11F; nearest-rank P95, at least 60 samples.
Available VM has 8 GiB and does not meet that condition.

| Metric | Target | Measured boundary / samples / result |
|---|---|---|
| Hotkey → visible/input-ready | P95 <150 ms | Hotkey callback → frontend rAF; 0 samples, unexecuted |
| Search input → painted results | P95 <100 ms | Keyboard event → first result frame including debounce; 0 samples, unexecuted |
| Clipboard update → visible history | P95 <300 ms | WM_CLIPBOARDUPDATE → frontend render; 0 samples, unexecuted |
| First search after commit vs hot cache | Gap <20% | Core service timings, ≥60 samples, retained in Phase 1 reports; failures remain open; not GUI evidence |
| Hidden idle CPU | <0.1% / 10 min | Full process tree, 0 samples, unexecuted |
| Hidden resident memory | <150 MB after 5 min | Includes WebView2, 0 samples, unexecuted |
| Installer | <15 MB excluding WebView2 | Phase 4 MSI measurement pending |

## Deviations and limits

- WAC22 requests the pinned panel stay visible after paste, while §5.15/W-UI-4
  and the pinned macOS reference hide after output. The reference resolves this
  conflict in favor of hiding after output; pin only suppresses focus-loss hiding.
- Heterogeneous text/image multi-selection cannot be represented as independent
  Windows clipboard items. It returns an explicit unsupported-combination error,
  preserving history. Homogeneous files and text/rich-text merges are supported.
- When merging HTML-only and RTF-only inputs, each rich representation uses plain
  text for the other format. Word format preservation is not yet accepted.
- Physical drag-out uses a synchronous Shell COPY operation. Original files stay
  in place; generated exports live only for the drop operation and are cleaned up
  on completion/cancellation. Physical receiver behavior remains unexecuted.
- OCR language order is configurable and persisted; old preference documents
  default to zh-Hans, zh-Hant, en-US without changing existing settings.

## Read-only reference

Phase 3 start was clean. The later read-only status check found external changes
in rea (README files, Package.swift, core/mac UI sources and tests). This task has
not written, built, formatted or run tests there. The pinned reference remains
c097591431c00982ad34186068064b40965c95fd. The requested empty end status therefore
cannot be asserted; external changes are left untouched.
