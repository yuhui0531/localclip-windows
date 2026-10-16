# Windows menu parity / 0.1.5

User scope: complete the missing screenshot actions, align card context menus and OCR entry with macOS, verify each action, rebuild and install into the Windows VM preserving history/settings and antivirus protection. Reference menu definitions: macOS 844924e; current macOS HEAD 509ce4e has the same menu definitions. Source repository remained read-only and clean.

- [x] Implement card-anchored, viewport-bounded menu; original copy, transforms, OCR, rename, favorite, category assignment/creation, locate history/category, details, preview, open, save, delete.
- [x] Expose OCR in card menus and details with progress/cancel and hidden-content protection.
- [x] Implement Ctrl+G locate, Space preview, Ctrl+O open; paginated locating, mixed filter, and category plus destination.
- [x] Frontend typecheck/build, 14 browser interaction cases, 43 local Rust tests, Clippy; first implementation Windows CI passed.
- [x] Native per-action acceptance with actual OCR/clipboard, save dialog and associated-app open.
- [x] Recheck actual drag, smart paste, highlighting, scrolling, focus, long-lived image cache and manual cleanup on the final binary.
- [x] Final ARM64/x64 builds and final-hash installer/antivirus checks.
- [x] Back up and verify personal history/settings, install in VM, verify installed hash/version and unchanged data before launch.

Final application commit: `ad5517e49a446b569ea2a1c4f251007182e63c0e`. Final native checks used an isolated generated-data instance of the exact ARM64 executable now installed in the VM. MSI exit 0; registered/file version 0.1.5; all six existing personal data files matched the verified backup before normal launch. No AV settings changed; VM idle pause restored.

Windows preview uses the existing local image/PDF/text preview instead of macOS Quick Look. Opening requires one validated stored target; multiple targets require selection. No schema migration or new background capture is introduced. Windows OCR recognition accuracy and the absent English language pack remain documented limitations. See `verification.md` and `reports/menu-parity-0.1.5.json`.
