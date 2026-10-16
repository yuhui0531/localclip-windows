# Windows menu parity / 0.1.5

User scope: complete the missing screenshot actions, align card context menus and OCR entry with macOS, verify each action, rebuild and install into the Windows VM preserving history/settings and antivirus protection. Reference macOS 844924e; source repository remains read-only.

- [x] Implement card-anchored, viewport-bounded menu; original copy, transforms, OCR, rename, favorite, category assignment/creation, locate history/category, details, preview, open, save, delete.
- [x] Expose OCR in card menus and details with progress/cancel and hidden-content protection.
- [x] Implement Ctrl+G locate, Space preview, Ctrl+O open; paginated locating, mixed filter, and category plus destination.
- [x] Frontend typecheck/build, 14 browser interaction cases, 43 local Rust tests, Clippy; first implementation Windows CI passed.
- [ ] Native per-action acceptance with actual OCR/clipboard, save dialog and associated-app open.
- [ ] Recheck actual drag, smart paste, highlighting, scrolling, focus, long-lived image cache and manual cleanup on the final binary.
- [ ] Final ARM64/x64 builds and final-hash installer/antivirus checks.
- [ ] Back up and verify personal history/settings, install in VM, verify installed hash/version and unchanged data before launch.

Native acceptance currently waits for the personal LocalClip instance to close. Do not force-kill it or test against its data. No native test on 0.1.5 has been marked passed. Windows preview uses the existing local image/PDF/text preview instead of macOS Quick Look. Opening requires one validated stored target; multiple targets require selection. No schema migration or new background capture is introduced.
