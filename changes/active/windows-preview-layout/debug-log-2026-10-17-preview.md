# Card / detail / image preview diagnosis

- [x] Read the user screenshot and trace card DOM, detail CSS, thumbnail endpoint and native panel sizing.
- [x] Compare macOS inspector layout; keep macOS repository read-only.
- [x] Root cause (high confidence): title was explicitly below content; detail used a 600px popover inside the 340px history window; image view reused a 640x480 thumbnail with card sizing rules. Text highlighter also silently limited long detail text to 16,384 characters.
- [x] Implement top title, temporary centered native inspection workspace, full-resolution proportional image viewer, and untruncated long text.
- [x] Verify seven browser layout cases and existing 14 menu/OCR cases. Native validation and package installation pending.

The fix follows the user's explicit presentation correction. Native inspection geometry is bounded by the current monitor work area and never persists as the history height. The command has its own main-window ACL entry. Image requests keep the existing record/path/privacy/40-million-pixel checks and separate full-image versus thumbnail cache keys. No historical content, drag output, smart-paste rules, long-lived image cache or AV configuration is changed.

Generated browser screenshots: reports/raw/layout-preview-2x.png and layout-preview-narrow.png. Full image edges, aspect ratio, 100%/zoom, close control, long-text end marker, narrow layout and native-layout command dispatch are asserted; these do not prove native window geometry.
