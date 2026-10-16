# Card / detail / image preview diagnosis

- [x] Read the user screenshot and trace card DOM, detail CSS, thumbnail endpoint and native panel sizing.
- [x] Compare macOS inspector layout; keep macOS repository read-only.
- [x] Root cause (high confidence): title was explicitly below content; detail used a 600px popover inside the 340px history window; image view reused a 640x480 thumbnail with card sizing rules. Text highlighter also silently limited long detail text to 16,384 characters.
- [x] Implement top title, temporary centered native inspection workspace, full-resolution proportional image viewer, and untruncated long text.
- [x] Verify eight browser layout cases and existing 14 menu/OCR cases. Native validation and package installation completed; see verification.md.

The fix follows the user's explicit presentation correction. Native inspection geometry is bounded by the current monitor work area and never persists as the history height. The command has its own main-window ACL entry. Image requests keep the existing record/path/privacy/40-million-pixel checks and separate full-image versus thumbnail cache keys. No historical content, drag output, smart-paste rules, long-lived image cache or AV configuration is changed.

Generated browser screenshots: reports/raw/layout-preview-2x.png and layout-preview-narrow.png. Tall and wide image edges, aspect ratio, 100%/zoom, close control, long-text end marker, narrow layout and native-layout command dispatch are asserted; these do not prove native window geometry.

Native evidence: 11 layout checks passed on the final ARM64 executable. The window grew from 4122×695 to 2426×1615 physical pixels inside the monitor work area, then restored exactly. The viewer decoded the original 1000×240 pixels instead of the old reduced thumbnail. Real pointer panning, original size, proportional fit, close and reopen behavior passed. Actual OCR, clipboard, Save As, associated-app Open, smart paste, highlighting, Notepad/Explorer drops, cache cleanup and focus regressions passed.

The first generated-receiver drag attempt stopped at its obscured-target guard before dragging because the harness read stale receiver metadata during startup. The harness now waits for the current process metadata and validates both receiver PID and window title before UI actions. The rerun passed; no application change was required. Generated clipboard content was cleared only while still owned by the isolated app, then that app was stopped.

Installation: verified backup before-0.1.6-20261017-011357; MSI exit 0; installed version 0.1.6 and final hash match; all six existing personal files unchanged before launch. Defender remained enabled; idle pause restored. macOS checkout remained clean and read-only. No unrelated release gate or 360 result is claimed.
