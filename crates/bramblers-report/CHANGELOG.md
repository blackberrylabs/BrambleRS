# Changelog

## 0.1.0 — 2026-10-16

First release: the `bbl-report/1` contract.

- Types: `Report`, `Module`, `Summary`, `Tone`, `Finding`, `Severity`, `Subject`,
  `SubjectKind`, `Evidence`, `Action`, `ActionKind`, `Open`.
- `Severity`: four levels on the wire as 1–4, named and explained in words.
- `Report::validate` lists every defect at once; `conformance::check` parses and validates
  one document, naming another schema instead of failing on it.
- `peek_schema`, `now_utc` (RFC 3339 UTC without a date dependency).
- Promise test: no file, process or network API in the source; `serde` and `serde_json` only.
