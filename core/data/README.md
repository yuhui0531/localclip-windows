# Offline link recognition

`tlds.txt` is the unmodified IANA list downloaded from
https://data.iana.org/TLD/tlds-alpha-by-domain.txt on 2026-10-14.
Its own header identifies the source version. The application never refreshes it over the network.

Product approved preserving macOS recognition on 2026-10-14: bare-domain
recognition excludes `.ai`, `.app`, `.dev`, `.md` and Unicode host labels,
matching the exported detector cases. This is a documented compatibility
restriction, not a statement that those TLDs are invalid. Explicit HTTP(S)
URLs still accept those domains. Future changes require new reference cases
and product approval; do not change the immutable Phase 0 vectors.

Unicode labels in explicit URLs use the encoding algorithm in RFC 3492 §6.3:
https://www.rfc-editor.org/rfc/rfc3492#section-6.3.
