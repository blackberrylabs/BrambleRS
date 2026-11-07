# Changelog

## [0.1.0] - 2026-11-07

BrambleRS 0.1.0 is the first release of the single software. It keeps the screen of the old
Blackberrylabs tower and puts the six tools inside it.

### One software

- The program is now BrambleRS, package and binary `bramblers`. The organization stays
  Blackberrylabs.
- The six tools are internal crates: Permitt, MCProster, Linstruct, Cleanator, Billfold and
  Lastrun. They run in the same process as the screen. Nothing is looked up on `PATH`, and no
  module is started as a subprocess.
- One workspace, one version, one `.deb`, one `.rpm`, one AppImage. The package replaces
  `blackberrylabs`, `blackberrylabs-full` and the six module packages. Their releases stay
  downloadable.

### The screen

- The tower screen is kept: four severity counts, one finding list across the six modules,
  and the detail of the selected finding with its evidence and its actions.
- The module cards show each module's figures and the age of its report.
- Removed: the "Get it" button, the "Open in <Module>" button and the not-installed state.
  The six modules are always there.
- English by default, French available, light and dark.

### Report and paths

- The internal report schema is `bramblers-report/1`. The old `bbl-report/1` reports are
  ignored; the six modules rebuild theirs on the first run.
- Settings live in `~/.config/bramblers/settings.json`, saved reports in
  `~/.local/state/bramblers/<module>/report.json`. The settings of the old `blackberrylabs`
  app are read once at the first launch.

### Command line

- One binary: `bramblers` with no argument opens the screen.
- `bramblers report --json` prints the global dashboard, `bramblers report <module> --json`
  prints one module's report. `list`, `refresh`, `--min-severity` and `--module` are kept.

### Not in 0.1.0

- The detailed screens of the six old apps (their tables, filters and dedicated views) are
  not in this release. They come back in 0.2.0 and later, one view at a time, until 1.0 has
  every feature of the six apps inside BrambleRS.
- No website, no telemetry, no network request.

### Promises

- Read-only: nothing on the machine is modified. Every corrective action is displayed, never
  executed.
- The only writes are the settings file and the saved reports.
- `#![forbid(unsafe_code)]` across the workspace.

## Older releases

The releases before 0.1.0 were the Blackberrylabs tower and the six separate tools. Their
notes are on the archived repositories under
[github.com/blackberrylabs](https://github.com/blackberrylabs).
