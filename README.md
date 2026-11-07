# BrambleRS

BrambleRS shows what AI coding agents do to a Linux machine, most serious first, on one
screen.

It is one program with six modules inside. It reads the configuration, instruction files,
transcripts and caches the agents already leave, merges their findings into one list, and
writes only its own settings and saved reports.

## What it answers

| Module | Question |
|---|---|
| Agent permissions | What each AI assistant may do without asking, and which setting says so |
| MCP servers | Which MCP servers are declared, once per server, with duplicates, clear secrets and dead commands |
| Instruction files | Which instruction files each host would load, in which order, and what it would drop |
| Disk cleanup | What AI tooling left on the disk, and what could go |
| Keys & spend | Which accounts and keys this machine holds, and how many tokens each host burned |
| Activity | What the agents actually ran, created, edited or deleted, session by session |

The four severities are act now, review, good to know and information. Each finding says
where it comes from and what you could do.

The tabs use these functional names. The internal ids (`permitt`, `mcproster`, `linstruct`,
`cleanator`, `billfold`, `lastrun`) stay in the crate names, the settings and the saved
reports, so an upgrade keeps its paths.

## Read-only

BrambleRS never edits an agent's configuration, never deletes a file, and makes no network
request. Every action in the interface is yours to take.

The only writes are its own settings under `~/.config/bramblers/` (language, theme, the
module preferences, the MCP probe results), the saved reports under
`~/.local/state/bramblers/<module>/report.json`, and the cleanup script the disk cleanup tab
offers to save.

## Install

The releases page has the `.deb`, `.rpm` and AppImage for x86_64 Linux:
[github.com/blackberrylabs/BrambleRS/releases](https://github.com/blackberrylabs/BrambleRS/releases).
The package replaces the old `blackberrylabs` package and the six separate module packages.
Those repositories and their releases were removed on 2026-11-07; BrambleRS is the only
source and the only download.

From source:

```sh
git clone https://github.com/blackberrylabs/BrambleRS.git
cd BrambleRS
cargo build --release -p bramblers --bin bramblers
./target/release/bramblers
```

The build needs a Rust toolchain and the system libraries iced uses (X11 or Wayland, with
Vulkan recommended).

## Use

With no argument, `bramblers` opens the screen. It shows the saved reports at once, runs the
six modules in the background, and replaces each report when it is ready.

```
bramblers report --json              # the global report, all six modules
bramblers report permitt --json      # one module's report
bramblers report                     # run the six modules and show the list
bramblers list                       # the saved reports, nothing is run
bramblers refresh permitt            # rerun one module and show the list
bramblers --min-severity 2           # act now and review only
bramblers --module permitt           # one module's findings
```

`bramblers report --json` prints the assembled dashboard; `bramblers report <module> --json`
prints that module's `bramblers-report/1` document. Both are meant for scripts.

## Build

One Cargo workspace, one version, one binary.

- `crates/bramblers-core`: the module registry, the in-process runner, the saved reports, the sorting.
- `crates/bramblers-report`: the report types and the conformance check.
- `crates/bramblers-hostrules`: the host sheets and the readers that never write.
- `crates/bramblers-ui`: the palette and the iced building blocks.
- `crates/bramblers-cli`: the command line, linked into the binary.
- `crates/bramblers`: the iced shell, package and binary `bramblers`.
- `crates/modules/<id>/core`: the six modules, each a crate, with its embedded rules and host sheets.
- `crates/modules/<id>/view`: the iced screen each module shows in its own tab.

`packaging/build.sh` builds the `.deb`, `.rpm` and AppImage into `dist/`. The whole workspace
is `#![forbid(unsafe_code)]`. The design brief, plan and journal are kept in French at the
root; code, comments and documentation are in English.

By [Alexandre Do](https://github.com/blackberrylabs).

## License

MIT, see [LICENSE](LICENSE).
