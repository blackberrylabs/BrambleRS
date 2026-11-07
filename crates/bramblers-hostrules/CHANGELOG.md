# Changelog

## [0.5.0] — 2026-10-17

### Added

- `Format::Jsonl`, `Format::Dotenv`, `Format::Sqlite`. JSONL (one JSON document per line —
  the hosts' session logs and rollouts) parses to a `Value::Array` of the lines that parse;
  dotenv (`KEY=value` lines — `.env`, `environment.d`, `secrets.env`) to a `Value::Object`
  of strings with dotenv's rules (first assignment wins, quotes stripped, nothing expanded);
  SQLite is refused by `parse` (a tool reads it with its own driver) but can be described by
  a location. For Billfold, the module that reads the hosts' credential files and usage logs.
  Existing formats are unchanged.

### Changed

- An active host may have no location when it carries at least one `references` URL: a
  host whose state lives on a server or in the keyring (Trae, JetBrains AI) can be
  described and detected without a readable file. A host with neither is still refused.

## [0.4.0] — 2026-10-17

### Added

- `Format::Markdown`: a location that is prose, not a tree. `read_tree` / `parse` return
  the text as a `serde_json::Value::String`, so a tool can inspect it as lines, bytes, front
  matter and imports. For Linstruct, the linter of the hosts' instruction files
  (`CLAUDE.md`, `AGENTS.md`, `.cursor/rules/*.mdc`…). Existing formats are unchanged.

## [0.3.0] — 2026-10-16

### Changed

- **Renamed `hostrules` → `bbl-hostrules`** (`use bbl_hostrules::…`), like `bbl-ui` and
  `bbl-report`: the shared foundations of the Blackberrylabs tools all carry the family
  prefix. Nothing else changes; the repository moved to
  `github.com/blackberrylabs/bbl-hostrules` (the old address redirects).

## [0.2.0] — 2026-10-16

### Added

- `HostExtension` and the second type parameter of `HostRule<X, H = NoExt>` / `Rules<X, H>`:
  a tool's fields about the host as a whole (Permitt: defaults, settings semantics,
  precedence), flattened next to the common fields and validated by the tool. Existing
  users (`HostRule<X>`) are unchanged.

## [0.1.0] — 2026-10-16

- Extracted from MCProster 0.2.0: `rules`, `env`, `read`, `secrets`, `resolve`.
