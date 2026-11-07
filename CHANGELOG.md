# Changelog

## [0.1.6] - 2026-11-07

BrambleRS 0.1.6 brings Lastrun's screen into its tab. The six tabs now carry the detailed
screens of the six old applications.

### Lastrun

- Four tabs on the left: Sessions (per host, project, permission modes, pruning date),
  Deleted (path, by a tool or by a command, outside the project, whether it exists again),
  Packages (manager and names) and Refused (what was denied, with the approval in three
  states).
- The middle list keeps the four severities and the host chips. The right panel shows the
  selected finding, or the selected session with its chronology: every command, file, MCP
  call and web access in order, with its status and approval, filters by kind, a search over
  commands and paths, and the diff of an edit read on demand.
- The 30 or 90 day window is saved in `~/.config/bramblers/lastrun/settings.json`; the old
  `~/.config/lastrun/settings.json` is read once.
- Prompts, replies and command output are never read. Only the actions are.

### Parity

- The six module tabs have their own screens. The 1.0 release is the state where no feature
  of the six old applications is missing.
- The ten old repositories, their releases and the five tower tags were removed on
  2026-11-07. Only BrambleRS remains.

### Promises

- Read-only: nothing on the machine is modified. Every corrective action is displayed, never
  executed.
- The only writes are the settings files, the probe results, the cleanup script and the saved
  reports.
- `#![forbid(unsafe_code)]` across the workspace.

## [0.1.5] - 2026-11-07

BrambleRS 0.1.5 brings Billfold's screen into its tab.

### Billfold

- Three tabs. Wiring shows each host, the account it is logged into, the credit behind it
  (subscription, plan, API key, keyring), the quota bars and the expiry. Keys lists every
  provider key found, masked, with the file, the git state and the hosts that read it. Meter
  groups tokens by host, model, project or day, with the API-equivalent dollars, the cache
  share and the log files the host will prune.
- The findings list keeps the four severities, the host chips, the search and the information
  toggle. The detail gives the meaning, where the key sits, the host's own command and the
  source.
- The 30, 90 or 365 day window is saved in `~/.config/bramblers/billfold/settings.json`; the
  old `~/.config/billfold/settings.json` is read once.
- The scan runs on the home, on a thread. Keys are masked the moment they are read; nothing is
  kept and nothing is written outside the settings file.

### Still to come

- The Lastrun tab still shows the shared finding list. Its detailed screen comes in 0.1.6.
- No website, no telemetry, no network request.

### Promises

- Read-only: nothing on the machine is modified. Every corrective action is displayed, never
  executed.
- The only writes are the settings files, the probe results, the cleanup script and the saved
  reports.
- `#![forbid(unsafe_code)]` across the workspace.

## [0.1.4] - 2026-11-07

BrambleRS 0.1.4 brings Cleanator's screen into its tab.

### Cleanator

- The Explorer draws the folder as a squarified treemap. A click on a tile opens the folder,
  "Up" goes back, and the legend names the rule categories behind the colours.
- Recommendations are grouped by rule when a rule has many items, with the size, the path,
  the why, and, on demand, the command, the last use, the cost and the rule's documentation.
  Safe and review items are separated; "Select safe", "Clear", copy and save the script.
- Three tabs: Explorer, AI assistants (configuration, history, caches per assistant) and
  Projects (regenerable weight, last touch, dormant or active, one checkbox per project).
- The saved script goes to `~/.local/state/bramblers/cleanator/cleanup.sh`; the old app wrote
  it to the home directory.
- The scan runs on the home directory, on a thread. The header refreshes it.
- A subtraction in the Docker probe no longer overflows when shared layers are larger than the
  image size.

### Still to come

- The two other module tabs still show the shared finding list. Their detailed screens come in
  0.1.5 and 0.1.6.
- No website, no telemetry, no network request.

### Promises

- Read-only: nothing on the machine is modified. Every corrective action is displayed, never
  executed.
- The only writes are the settings files, the probe results, the cleanup script and the saved
  reports.
- `#![forbid(unsafe_code)]` across the workspace.

## [0.1.3] - 2026-11-07

BrambleRS 0.1.3 brings Linstruct's screen into its tab.

### Linstruct

- One card per host present on the machine: what it loads, in its own order, with its budget
  gauge, the size and line count of each file, and the state of each slot (loaded, on demand,
  cut, skipped, shadowed, ignored). A click on a host filters the findings.
- Projects: the folders you remember, added and removed in place; a project filter; the
  user-level files above them. Hosts that are not installed appear under "what if".
- The findings list keeps the four severities, the host chips, the search and the information
  toggle. The detail gives the meaning, the path, the host's own check command and the source.
- Project folders move to `~/.config/bramblers/linstruct/settings.json`; the old
  `~/.config/linstruct/settings.json` is read once.

### Still to come

- The three other module tabs still show the shared finding list. Their detailed screens come
  in 0.1.4 to 0.1.6.
- No website, no telemetry, no network request.

### Promises

- Read-only: nothing on the machine is modified. Every corrective action is displayed, never
  executed.
- The only writes are the settings files, the probe results and the saved reports.
- `#![forbid(unsafe_code)]` across the workspace.

## [0.1.2] - 2026-11-07

BrambleRS 0.1.2 brings MCProster's screen into its tab.

### MCProster

- The tab shows the table of MCP servers and where each one is declared: transport, version,
  probed size, and one column per host when six or fewer hosts have configuration.
- Filters: name, package, URL or file; one host; findings only; disabled and legacy entries.
  The detail shows each declaration with its file, launch command, environment, headers and
  masked secrets, then the probe.
- The probe starts a stdio server as its host would, sends initialize and tools/list, stops it,
  and adds the tool definitions to the context cost per host. A confirmation comes first.
- Probe results move to `~/.config/bramblers/mcproster/probes.json`; the old
  `~/.config/mcproster/probes.json` is read once.
- Remote servers are listed, never contacted.

### Still to come

- The four other module tabs still show the shared finding list. Their detailed screens come
  in 0.1.3 to 0.1.6.
- No website, no telemetry, no network request.

### Promises

- Read-only: nothing on the machine is modified. Every corrective action is displayed, never
  executed.
- The only writes are the settings files, the probe results and the saved reports.
- `#![forbid(unsafe_code)]` across the workspace.

## [0.1.1] - 2026-11-07

BrambleRS 0.1.1 brings the first detailed module screen, Permitt, inside its tab. The rest of
the screen is unchanged.

### Permitt

- The tab shows Permitt's own screen: the sheet of hosts and capabilities, four figures, the
  filters, the evidence behind each state and the findings. The left rail and the other tabs
  stay in place.
- One row per assistant and one state per capability (commands, files, network, MCP). A click
  on a cell opens its precedence, the settings that set it, the settings written but not
  honoured, the shell start-up hits and the action to take.
- Filters: host name, user level or one project, installed only, findings only, and whether
  the shell start-up files are read. Rescan reads every settings file again on a thread.
- Permitt's own preference moves to `~/.config/bramblers/permitt/settings.json`. The old
  `~/.config/permitt/settings.json` is read once.

### Still to come

- The five other module tabs still show the shared finding list. Their detailed screens come
  in 0.1.2 to 0.1.6.
- No website, no telemetry, no network request.

### Promises

- Read-only: nothing on the machine is modified. Every corrective action is displayed, never
  executed.
- The only writes are the settings files and the saved reports.
- `#![forbid(unsafe_code)]` across the workspace.

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
  `blackberrylabs`, `blackberrylabs-full` and the six module packages. Their releases stayed
  downloadable until 2026-11-07, when the old repositories were removed.

### The screen

- The tower screen is kept: four severity counts, one finding list across the six modules,
  and the detail of the selected finding with its evidence and its actions.
- The six modules are tabs in a left rail. The overview keeps the combined list; a module
  tab limits the counts, the list and the detail to that module.
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

The releases before 0.1.0 were the Blackberrylabs tower and the six separate tools. The old
repositories and their releases were removed on 2026-11-07. BrambleRS 0.1.0 is the first
release kept.
