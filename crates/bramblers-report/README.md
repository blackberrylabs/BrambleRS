# bbl-report

The report contract between the [Blackberrylabs](https://github.com/blackberrylabs) control
tower and its modules.

A module — [Cleanator](https://github.com/blackberrylabs/cleanator),
[MCProster](https://github.com/blackberrylabs/mcproster),
[Permitt](https://github.com/blackberrylabs/permitt),
[Linstruct](https://github.com/blackberrylabs/linstruct), … — answers `<module> report --json`
with one `bbl-report/1` document on stdout. The tower reads those documents, merges them
into one list of findings sorted by severity, and opens the module at the right place. This
crate is the only thing the two sides share.

It holds the Rust types of that document, their JSON form, a validation that lists every
defect at once, the four-level severity scale in words, and a conformance check every module
runs on its own output. It reads nothing, writes nothing and never touches the network;
its only dependencies are `serde` and `serde_json`.

## The report

```jsonc
{
  "schema": "bbl-report/1",
  "module": { "id": "permitt", "name": "Permitt", "version": "0.1.0", "verb": "inspect",
              "tagline": "What your AI agents may do without asking",
              "app": "permitt-app", "cli": "permitt",
              "home": "https://github.com/blackberrylabs/permitt" },
  "generated_at": "2026-10-16T21:03:00Z",
  "duration_ms": 120,
  "summary": [                                     // 1 to 4 figures: the module's KPI cards
    { "key": "hosts", "label": "Hosts installed", "value": "4", "tone": "neutral" },
    { "key": "auto",  "label": "Run without asking", "value": "6", "tone": "alert" }
  ],
  "findings": [
    { "id": "runs-without-asking/opencode/commands",   // stable across runs
      "severity": 2,                                   // 1 act now · 2 review · 3 good to know · 4 info
      "kind": "runs-without-asking",                   // the module's own slug
      "title": "OpenCode runs shell commands without asking",
      "subject": { "kind": "host", "label": "OpenCode" },
      "detail": "by default, nothing changed it",
      "evidence": [ { "label": "~/.config/opencode/opencode.json", "path": "/home/me/.config/opencode/opencode.json" } ],
      "actions": [ { "kind": "hint", "label": "Add a permission block in ~/.config/opencode/opencode.json" },
                   { "kind": "open-file", "label": "Open file", "target": "/home/me/.config/opencode/opencode.json" } ],
      "open": { "app": "permitt-app", "args": ["--select", "opencode:commands"] }
    }
  ],
  "notes": [ "3 shell files read" ],
  "errors": []
}
```

### Severity

Four common levels. Each module maps its own findings onto them and says how in its
changelog; the tower never reinterprets.

| Wire | Name | Meaning |
|---|---|---|
| 1 | `act-now` | data loss possible, a secret in clear, a bypass in a shell file |
| 2 | `review` | acts without asking, broad access, a large recoverable volume |
| 3 | `good-to-know` | a duplicate, a version that differs, a deny list alone |
| 4 | `info` | disabled, not cached, not readable |

### Rules

- **Nothing new in the report**: a module puts in it only what it already shows on screen.
  Secrets are masked upstream; this crate masks nothing.
- **Actions are shown, never run** — by the tower or anyone reading the report.
- **Forward compatible**: readers ignore unknown fields and every list or optional field has
  a default, so a field *added* keeps `bbl-report/1`. A field removed or whose meaning
  changes becomes `bbl-report/2`; `peek_schema` reads the version before any typed parsing.

## Part of Blackberrylabs

One of the three shared crates of the [Blackberrylabs](https://github.com/blackberrylabs)
family — with [`bbl-hostrules`](https://github.com/blackberrylabs/bbl-hostrules) (the files of AI hosts) and [`bbl-ui`](https://github.com/blackberrylabs/bbl-ui) (the shared face) — behind the control tower and its modules (Cleanator, MCProster,
Permitt). A new module starts from the [module template](https://github.com/blackberrylabs/template);
the procedure is [`docs/new-module.md`](https://github.com/blackberrylabs/blackberrylabs/blob/main/docs/new-module.md)
in the tower.

## Use

```toml
[dependencies]
bbl-report = { git = "https://github.com/blackberrylabs/bbl-report", tag = "v0.1.0" }
```

Producing, in a module:

```rust
use bbl_report::*;

let mut report = Report::new(Module {
    id: "permitt".into(), name: "Permitt".into(), version: env!("CARGO_PKG_VERSION").into(),
    verb: "inspect".into(), tagline: "What your AI agents may do without asking".into(),
    app: Some("permitt-app".into()), cli: Some("permitt".into()),
    home: "https://github.com/blackberrylabs/permitt".into(),
});
report.summary.push(Summary { key: "hosts".into(), label: "Hosts installed".into(), value: "4".into(), tone: Tone::Neutral });
// … findings …
print!("{}", report.to_json());
```

Checking, in the module's tests and in the tower:

```rust
let report = bbl_report::conformance::check(&json)?;   // Err(Vec<String>) lists every defect
```

## Adding a module

The whole procedure — template, report, severity mapping, `--select`, the sheet in the
tower — is [`docs/new-module.md`](https://github.com/blackberrylabs/blackberrylabs/blob/main/docs/new-module.md).

## Development

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

`tests/promises.rs` checks the round trip, that unknown fields are ignored, that another
schema is named and refused, that every validation rule fires, and that the source mentions
no file, process or network API.

## License

MIT — Alexandre Do.
