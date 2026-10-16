# Windows 0.1.4 / macOS 0.3.38 behavior mapping

Reference: macOS `c097591` (0.3.29) → `844924e` (0.3.38). The macOS repository is read-only. Windows implementation is native Rust/Tauri; no macOS source or user database was copied.

| macOS change | Windows implementation / boundary |
|---|---|
| Tray About and manual update check (0492851) | Both entries open the About settings page; manual check invokes the existing GitHub Releases updater. |
| Batch/protected deletion confirmation; output limits (404fc0f) | Multi-selection/protected deletion uses a preview token and confirmation. Output metadata is checked before loading; composed output is limited to 50 MiB. Hidden panels reject output. Existing transactional undo already refuses quota overflow without trimming other records. |
| Unknown initial focus (db60bce) | Preserve target process identity when child focus cannot be read; latch restored focus and recheck foreground before SendInput. Smart code-file conversion still requires a verified writable text target. |
| Smart code-file drag (db60bce) | With the existing switch enabled, publish both Unicode source text and original CF_HDROP paths. Unsupported/oversized/unreadable groups retain original file output. File handles stay open through the drag. |
| Persistent dragged images (db60bce, ca874a9, f769430) | User-approved DragImages cache, byte-verified deduplication, independent copies after external edits, no automatic expiry. Successful references survive drag completion. Storage shows usage; manual cleanup requires confirmation and excludes unrelated files. Original history representations remain unchanged. |
| Rich text joins Text (3cba2fd) | New HTML/RTF records classify as Text. Legacy richText rows participate in Text search and retain all original bytes. Windows keeps schema 1 and does not adopt macOS's schema-4 rejection. Five explicit vector expectation overrides are recorded separately; original reference vectors remain intact. |
| File OCR and content protection (f769430) | OCR reads actual local image-file contents, including extensionless images, never file icons. Cancellation, hiding, record removal and intervening clipboard changes prevent publication. Original resolution is retained for OCR. |
| Syntax/detail layout (f769430, 014d42f) | Paths/URLs/prose no longer qualify merely through weak highlighting relevance. Detail and inspection data are awaited together; text highlighting is synchronous and bounded. Detail preview height is bounded. |
| Image/file sizes (ba85357) | Saved image byte size and current local file size, with unknown/folder cases explicit. No recursive directory size scans. |
| Actual file previews (937856c) | Read full stored references, inspect content for images regardless of extension, render PDF first page with Windows.Data.Pdf, request Office/RTF thumbnails from installed Windows Shell providers, fall back to strictly decoded bounded text. External previews stay in memory and are keyed by file metadata version. Unsupported/provider-less documents show a fallback. |
| Finder staging recovery (ba85357) | Windows uses OLE streams for exported text and original paths for real files; it has no Finder staging journal. Persistent image files are managed by the separate approved cache. |
| macOS updater sessions/garbage collection (ca874a9, ba85357) | Windows retains the PRD-specific signed MSI mechanism, pre-install SQLite snapshot and startup marker. No macOS app-bundle swap/session directories exist to migrate. Snapshot retention is unchanged; no automatic deletion of existing backups is introduced. Signed installation remains disabled until keys are supplied, as previously approved. Signed update/recovery acceptance remains open. |
| Swift/compiler/test-host changes | Platform-specific; not transplanted into Rust. |

## Installer changes

- Only the production `localclip.exe` is installed. Development executables remain available for repository testing.
- `embedBootstrapper` replaces the PowerShell WebView2 download CustomAction. The embedded Microsoft bootstrapper still installs WebView2 when needed.
- Both architecture MSIs must be inspected after packaging; source configuration alone is not acceptance evidence.
- Unsigned test packages remain unsigned. A clean Defender result cannot establish that 360 accepts a sample or identify the original detection cause.
