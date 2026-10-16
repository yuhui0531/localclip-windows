# Windows menu parity / 0.1.5

User scope: complete the missing screenshot actions, align card context menus and OCR entry with macOS, verify each action, rebuild and install into the Windows VM preserving history/settings and antivirus protection. Reference macOS 844924e; source repository remains read-only.

- [ ] Card-anchored, viewport-bounded context menu; original copy, transforms, OCR, rename, favorite, category assignment, locate history/category, details, preview, open, save, delete.
- [ ] OCR in image/file-image details, cancel and privacy guards; visible real recognition result from generated image.
- [ ] Ctrl+G locate, Space preview, Ctrl+O open; pagination and stale-selection protection; mixed filter and category plus destination.
- [ ] Per-action frontend/native verification and drag, smart-paste, highlighting, scrolling, focus, cache regression checks.
- [ ] ARM64/x64 builds, installer inspection and antivirus scan; preserve and verify personal data, upgrade VM and launch normally.

Implementation uses existing store/preview/OCR/output paths. Windows preview uses the existing local image/PDF/text preview instead of macOS Quick Look. Opening is an explicit user action on one validated stored target; multi-target cards require selection. No schema migration or new background capture is introduced.
