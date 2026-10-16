# Syntax highlighting — 0.1.7

Application source: `0c8f15ca4ef24c884963e258267807477484deab`.

History cards now use bounded original text instead of only the stored 180-character summary. Search excerpts and privacy guards remain in place. Long Details preserve full text alongside bounded coloring, and recognized HTML-backed code uses syntax rendering.

Completed automated checks:
- 15 focused highlighting/rendering cases; 11 Chromium card/layout cases; 14 Chromium menu/OCR cases.
- 43 local Rust tests, including 50 forced-crash recoveries; formatting, Clippy and UI build.
- [Windows/macOS CI](https://github.com/yuhui0531/localclip-windows/actions/runs/38940410368): success.
- [ARM64/x64 installers](https://github.com/yuhui0531/localclip-windows/actions/runs/38940449968): success.

Native Windows 11 ARM64 verification:
- 10 checks passed using generated clipboard content and an isolated history directory: capture, code beyond a comment-only card summary, full long text, syntax colors, escaped source markup, HTML-backed code and ordinary rich text.
- Hide removes all code previews; restoring visibility restores colors. The native panel screenshot was visually inspected, including text, JSON and Rust-file colors.
- Initial CF_HTML string fixture was rejected as unsupported; explicit UTF-8 CF_HTML bytes passed on the unchanged executable. This setup failure was not counted as a product pass.
- Four Defender scans (both MSI/executable pairs) passed with real-time protection enabled. This is not a 360 false-positive resolution claim.
- Installed exact tested ARM64 MSI, exit 0. Registered/file version 0.1.7 and executable SHA-256 `716feadee0d55ea9eb1613dc328d3254c9a92c0b2a75d9aa252fef264abb939f` match. All six existing data files were backed up and hash-verified unchanged before normal user launch. Backup: `%LocalAppData%\LocalClip-UpgradeBackups\before-0.1.7-20261017-015531`. Antivirus and real-time protection remained enabled; VM idle pause restored.

[Hash-bound verification](../../../reports/highlight-0.1.7.json). Prior full drag/paste/cache acceptance remains documented against 0.1.6; those output paths were not changed or re-certified by the narrower 0.1.7 native checks.

Limits: the user identified history cards but supplied no particular language/sample. The generated regression specifically covers code hidden beyond a comment-only summary. Very short ambiguous snippets retain the macOS detection threshold. Coloring remains bounded to 16,384 UTF-16 units, while Details retain the full text. No antivirus bypass or allowlist is used.
