# Edit history content

User approved updating the selected record in place, preserving title, favorite and category membership.
Project root: `/Users/ronny/VscodeProjects/localclip-windows`.
No separate design document: narrow feature using the existing writer transaction, IPC and detail window.

- [x] Atomic full-text edit with stale/duplicate/visibility guards, metadata preservation and rollback tests.
- [x] Chinese 编辑内容 menu and details entry, multiline editor, save/cancel, IME-safe keyboard and privacy guards.
- [ ] Verify search, copy payload, syntax highlighting and existing UI regressions; build Windows packages.

Edited rich text becomes plain text after explicit Save; the editor explains this. Unchanged Save preserves original formats. Single textual records only; images, files and mixed records are not modified. Empty/NUL/over-50-MiB text rejected. Duplicate content rejected rather than merging or deleting another record. Save does not write the clipboard or change copy timestamps/counts. No schema/dependency/OS permission/network changes.
