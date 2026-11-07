# bbl-hostrules

**Where each AI assistant keeps its files on a machine, and how to read them without
writing — with the source of every claim.**

`bbl-hostrules` is the Rust crate shared by the [Blackberrylabs](https://github.com/blackberrylabs)
tools — [MCProster](https://github.com/blackberrylabs/mcproster) (inventory of MCP
servers) and Permitt (inventory of agent permissions). Each tool ships its own rule files;
this crate owns what they have in common.

A **host rule** is a TOML file: the host's identity, the locations of its files per
operating system and scope (user / project / system), the file format, and — for every
location — the **source** it was established from: the vendor's documentation, the host's
source code, or an observation on a real machine, with the URL, the date and the words it
was read from. A rule whose location has no source is refused at load time. No rule may
rest on memory.

```toml
id = "codex"
name = "Codex CLI"
kind = "cli"
status = "active"
hints.remove = "codex mcp remove <name>"   # displayed by a tool, never run

[[locations]]
scope = "user"
format = "toml"
path.any = "~/.codex/config.toml"
servers_key = "mcp_servers"                # a field of the tool's own extension

[locations.source]
kind = "documentation"
url = "https://…"
read_on = 2026-10-16
quote = "the words the path and key were read from"
```

## What the crate does

- `rules` — the rule model, loading and validation. `Location<X>` is generic over an
  `Extension`: the fields a tool needs in every location (which key holds the MCP servers),
  flattened into the location's table and validated by the tool's own rule. `HostRule<X, H>`
  adds an optional `HostExtension`: what the tool knows about the host as a whole (its
  defaults, the meaning of its permission settings, its precedence), flattened next to the
  common fields.
- `env` — the environment a run is made against: home, OS, variables, `PATH`, project
  roots; path expansion (`~`, `${VAR}`, `%VAR%`, `$VAR`) and `which`. Built from the
  process, or from a fixture so tests are hermetic.
- `resolve` — a location → the concrete files it designates on this machine: per-OS paths,
  per-project paths under every root, globs, the projects Claude Code knows.
- `read` — JSON, JSONC, TOML and YAML → `serde_json::Value`, Markdown kept as text (for
  the hosts' instruction files), JSONL as an array of its lines, dotenv as an object of
  strings, SQLite described but read by the tool's own driver; key paths (`a.b.*.c`, `\.`
  for a literal dot); the line a key sits on.
- `secrets` — classify a value the moment it is read: a *reference* the host expands
  itself (`${VAR}`, `${env:NAME}`, `${{ secrets.X }}`) is shown as written; a credential
  is masked (`sk-pro…6789`) and its clear text never leaves the module.

## Two promises

1. **Read-only.** There is no function in this crate that writes, moves or deletes a file;
   a test checks the source for it.
2. **No network.** No network dependency, no network code; a test checks that too.

## Part of Blackberrylabs

One of the three shared crates of the [Blackberrylabs](https://github.com/blackberrylabs)
family — with [`bbl-ui`](https://github.com/blackberrylabs/bbl-ui) (the shared face) and [`bbl-report`](https://github.com/blackberrylabs/bbl-report) (the tower's contract) — behind the control tower and its modules (Cleanator, MCProster,
Permitt). A new module starts from the [module template](https://github.com/blackberrylabs/template);
the procedure is [`docs/new-module.md`](https://github.com/blackberrylabs/blackberrylabs/blob/main/docs/new-module.md)
in the tower.

## Use

```toml
[dependencies]
bbl-hostrules = { git = "https://github.com/blackberrylabs/bbl-hostrules", tag = "v0.3.0" }
```

```rust
use bbl_hostrules::{Extension, Location, Rules};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct MyLocation { servers_key: String }

impl Extension for MyLocation {
    fn validate(loc: &Location<Self>) -> Result<(), String> {
        if loc.ext.servers_key.is_empty() { return Err("empty servers_key".into()); }
        Ok(())
    }
}

let rules: Rules<MyLocation> = Rules::load([("codex.toml", include_str!("../hosts/codex.toml"))])?;
let env = bbl_hostrules::Env::current();
let roots = bbl_hostrules::resolve::project_roots(&env);
for host in &rules.hosts {
    for loc in &host.locations {
        for file in bbl_hostrules::resolve::resolve(loc, &env, &roots) {
            // read_tree(&file.path, loc.format) → serde_json::Value
        }
    }
}
```

`#![forbid(unsafe_code)]`. Rust 1.80+. Status: **0.2.0** (host-level extension, for Permitt); MCProster 0.2.1 uses 0.1.0. Not on
crates.io yet — it will be once two tools depend on it and the API has held a version.

By [Alexandre Do](https://github.com/blackberrylabs). MIT — see [LICENSE](LICENSE).
