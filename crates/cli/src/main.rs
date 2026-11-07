//! `bramblers`: the BrambleRS control tower on the command line. `bramblers` shows the saved
//! reports (nothing is launched), `bramblers refresh` runs the installed modules, `bramblers modules`
//! lists the family. `--json` for scripts.

use std::io::Write;

use anyhow::Result;
use bramblers_core::cache::age_label;
use bramblers_core::dashboard::ReportState;
use bramblers_core::{Cache, Dashboard, Modules};
use bramblers_report::{ActionKind, Severity};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bramblers",
    version,
    about = "BrambleRS — everything your AI agents do to your machine, on one screen. Read-only, like the tools behind it."
)]
struct Cli {
    /// Machine-readable output.
    #[arg(long, global = true)]
    json: bool,
    /// Show findings at this severity or stronger: 1 act now · 2 review · 3 good to know · 4 all.
    #[arg(long, global = true, default_value_t = 4, value_parser = clap::value_parser!(u8).range(1..=4))]
    min_severity: u8,
    /// Only this module's findings.
    #[arg(long, global = true)]
    module: Option<String>,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// The list of findings from the saved reports, most serious first (default).
    List,
    /// The modules of the family: installed or not, and the age of their report.
    Modules,
    /// Run the installed modules (or the ones named), save their reports, then show the list.
    Refresh { ids: Vec<String> },
}

macro_rules! out {
    ($($arg:tt)*) => {{
        let mut o = std::io::stdout().lock();
        let _ = writeln!(o, $($arg)*);
    }};
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let modules = Modules::embedded();
    let cache = Cache::default_location();
    let path = bramblers_core::discover::path_dirs();
    let mut dash = Dashboard::from_saved(&modules, &cache, &path);
    let min = Severity::try_from(cli.min_severity).expect("1..=4");

    match cli.command.unwrap_or(Command::List) {
        Command::Modules => modules_cmd(&dash, cli.json),
        Command::Refresh { ids } => {
            let ids: Vec<String> = if ids.is_empty() {
                dash.modules
                    .iter()
                    .filter(|m| m.cli.is_some())
                    .map(|m| m.spec.id.clone())
                    .collect()
            } else {
                ids
            };
            if ids.is_empty() {
                eprintln!("no module installed: nothing to run (see `bramblers modules`)");
            }
            for id in ids {
                let name = dash
                    .module(&id)
                    .map(|m| m.spec.name.clone())
                    .unwrap_or_else(|| id.clone());
                eprint!("{name} … ");
                match dash.refresh(&id, &cache) {
                    Ok(run) => eprintln!(
                        "{} finding(s) in {:.1} s",
                        run.report.findings.len(),
                        run.elapsed.as_secs_f64()
                    ),
                    Err(e) => eprintln!("{e}"),
                }
            }
            list(&dash, min, cli.module.as_deref(), cli.json);
        }
        Command::List => list(&dash, min, cli.module.as_deref(), cli.json),
    }
    Ok(())
}

fn state_line(m: &bramblers_core::ModuleStatus) -> String {
    match &m.state {
        ReportState::Fresh { elapsed_ms } => {
            format!("report just now ({:.1} s)", *elapsed_ms as f64 / 1000.0)
        }
        ReportState::Saved { age_s } => format!(
            "report {}",
            age_label(std::time::Duration::from_secs(*age_s))
        ),
        ReportState::NoReport => "installed, no report yet — run `bramblers refresh`".to_string(),
        ReportState::NotInstalled => format!("not installed — {}", m.spec.releases),
        ReportState::Failed { error } => format!("failed: {error}"),
    }
}

fn list(dash: &Dashboard, min: Severity, only: Option<&str>, json: bool) {
    if json {
        out!("{}", dash.to_json().trim_end());
        return;
    }
    let counts = dash.counts();
    let shown: Vec<_> = dash
        .at_least(min)
        .filter(|l| only.is_none_or(|m| l.module == m))
        .collect();
    out!(
        "{} to act on now · {} to review · {} good to know · {} information",
        counts[0].1,
        counts[1].1,
        counts[2].1,
        counts[3].1
    );
    for m in &dash.modules {
        out!("  {:<10} {}", m.spec.name, state_line(m));
    }
    out!();
    if shown.is_empty() {
        if dash.lines.is_empty() {
            out!("No finding yet. `bramblers refresh` runs the installed modules and saves their reports.");
        } else {
            out!("No finding at this severity.");
        }
        return;
    }
    let name_w = shown.iter().map(|l| l.module_name.len()).max().unwrap_or(8);
    for l in &shown {
        let f = &l.finding;
        out!(
            "{} {:<name_w$}  {}",
            sev_tag(f.severity),
            l.module_name,
            f.title,
            name_w = name_w
        );
        if !f.detail.is_empty() {
            out!("{:>3} {:<name_w$}  {}", "", "", f.detail, name_w = name_w);
        }
        for a in &f.actions {
            let line = match (&a.kind, &a.target) {
                (ActionKind::Hint, _) => a.label.clone(),
                (ActionKind::Command, Some(t)) => format!("{}: `{t}`", a.label),
                (ActionKind::OpenFile, Some(t)) | (ActionKind::OpenUrl, Some(t)) => {
                    format!("{}: {t}", a.label)
                }
                (_, None) => a.label.clone(),
            };
            out!("{:>3} {:<name_w$}  → {line}", "", "", name_w = name_w);
        }
        if let Some(o) = &f.open {
            out!(
                "{:>3} {:<name_w$}  ↗ {} {}",
                "",
                "",
                o.app,
                o.args.join(" "),
                name_w = name_w
            );
        }
    }
    out!();
    out!("{} finding(s) shown. BrambleRS changes nothing: every action above is for you to take, in the module or by hand.", shown.len());
}

fn sev_tag(s: Severity) -> &'static str {
    match s {
        Severity::ActNow => "[1]",
        Severity::Review => "[2]",
        Severity::GoodToKnow => "[3]",
        Severity::Info => "[4]",
    }
}

fn modules_cmd(dash: &Dashboard, json: bool) {
    if json {
        out!("{}", serde_json::to_string_pretty(&dash.modules).unwrap());
        return;
    }
    for m in &dash.modules {
        out!("{} — {} · {}", m.spec.name, m.spec.verb, m.spec.tagline);
        match (&m.cli, &m.app) {
            (Some(c), Some(a)) => out!("  installed: {} · {}", c.display(), a.display()),
            (Some(c), None) => out!("  installed: {} (no desktop app found)", c.display()),
            (None, Some(a)) => out!(
                "  installed: {} (no command line found: the tower cannot refresh it)",
                a.display()
            ),
            (None, None) => out!("  not installed. {}", m.spec.description),
        }
        out!("  {}", state_line(m));
        if let Some(r) = &m.report {
            let figures: Vec<String> = r
                .summary
                .iter()
                .map(|s| format!("{} {}", s.label.to_lowercase(), s.value))
                .collect();
            out!(
                "  {} · v{} · {}",
                figures.join(" · "),
                r.module.version,
                r.generated_at
            );
        }
        out!("  {}", m.spec.home);
        out!();
    }
}
