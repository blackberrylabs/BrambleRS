//! `bramblers`: the command line of BrambleRS. No subcommand shows the saved reports;
//! `report` runs the six modules (or one named module) and prints what they find;
//! `refresh` reruns them and shows the combined list. `--json` for scripts. The desktop app
//! is the same binary without a subcommand.

use std::ffi::OsString;
use std::io::Write;

use anyhow::{bail, Result};
use bramblers_core::cache::age_label;
use bramblers_core::dashboard::ReportState;
use bramblers_core::{Cache, Dashboard};
use bramblers_report::{ActionKind, Severity};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "bramblers",
    version,
    about = "BrambleRS — everything your AI agents do to your machine, on one screen. Read-only."
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
    /// Run the six modules (or the ones named), save their reports, then show the list.
    Refresh { ids: Vec<String> },
    /// Run the six modules and print their report: all of them, or the one named.
    Report { module: Option<String> },
}

macro_rules! out {
    ($($arg:tt)*) => {{
        let mut o = std::io::stdout().lock();
        let _ = writeln!(o, $($arg)*);
    }};
}

/// Replace control characters in a human line. A finding's title, detail or command comes
/// from a file on the machine and must not drive the terminal (OSC 52, title changes,
/// carriage returns). JSON output is untouched: `serde_json` escapes it already.
fn safe(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// Run the command line. `args` includes the program name (as `std::env::args` gives it).
pub fn run<I, T>(args: I) -> Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    if bramblers_hostrules::env::home_dir().is_none() {
        bail!(
            "no home directory: HOME is unset or / and /etc/passwd has no entry; refusing to scan /"
        );
    }
    let cli = Cli::parse_from(args);
    let cache = Cache::default_location();
    let mut dash = Dashboard::from_saved(&cache);
    let min = Severity::try_from(cli.min_severity).expect("1..=4");

    match cli.command.unwrap_or(Command::List) {
        Command::List => list(&dash, min, cli.module.as_deref(), cli.json),
        Command::Refresh { ids } => {
            let ids = chosen_ids(&dash, ids);
            run_modules(&mut dash, &cache, &ids);
            list(&dash, min, cli.module.as_deref(), cli.json);
        }
        Command::Report { module: Some(id) } => {
            let Some(provider) = bramblers_core::registry::get(&id) else {
                bail!(
                    "unknown module {id:?}; the modules are: {}",
                    bramblers_core::registry::all()
                        .iter()
                        .map(|p| p.id)
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            };
            eprint!("{} … ", provider.name);
            let run = dash.refresh(&id, &cache)?;
            eprintln!(
                "{} finding(s) in {:.1} s",
                run.report.findings.len(),
                run.elapsed.as_secs_f64()
            );
            if cli.json {
                print!("{}", run.json);
            } else {
                list(&dash, min, Some(&id), false);
            }
        }
        Command::Report { module: None } => {
            let ids = chosen_ids(&dash, Vec::new());
            run_modules(&mut dash, &cache, &ids);
            if cli.json {
                print!("{}", dash.to_json());
            } else {
                list(&dash, min, None, false);
            }
        }
    }
    Ok(())
}

/// The ids asked for, or every module in family order.
fn chosen_ids(dash: &Dashboard, ids: Vec<String>) -> Vec<String> {
    if ids.is_empty() {
        dash.modules
            .iter()
            .map(|m| m.provider.id.to_string())
            .collect()
    } else {
        ids
    }
}

fn run_modules(dash: &mut Dashboard, cache: &Cache, ids: &[String]) {
    for id in ids {
        let name = dash
            .module(id)
            .map(|m| m.provider.name)
            .unwrap_or(id.as_str());
        eprint!("{} … ", safe(name));
        match dash.refresh(id, cache) {
            Ok(run) => eprintln!(
                "{} finding(s) in {:.1} s",
                run.report.findings.len(),
                run.elapsed.as_secs_f64()
            ),
            Err(e) => eprintln!("{}", safe(&e.to_string())),
        }
    }
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
        ReportState::NoReport => "no report yet — run `bramblers report`".to_string(),
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
        out!("  {:<10} {}", m.provider.name, safe(&state_line(m)));
    }
    out!();
    if shown.is_empty() {
        if dash.lines.is_empty() {
            out!(
                "No finding yet. `bramblers report` runs the six modules and saves their reports."
            );
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
            safe(&f.title),
            name_w = name_w
        );
        if !f.detail.is_empty() {
            out!(
                "{:>3} {:<name_w$}  {}",
                "",
                "",
                safe(&f.detail),
                name_w = name_w
            );
        }
        for a in &f.actions {
            let line = match (&a.kind, &a.target) {
                (ActionKind::Hint, _) => safe(&a.label),
                (ActionKind::Command, Some(t)) => {
                    format!("{}: `{}`", safe(&a.label), safe(t))
                }
                (ActionKind::OpenFile, Some(t)) | (ActionKind::OpenUrl, Some(t)) => {
                    format!("{}: {}", safe(&a.label), safe(t))
                }
                (_, None) => safe(&a.label),
            };
            out!("{:>3} {:<name_w$}  → {line}", "", "", name_w = name_w);
        }
    }
    out!();
    out!(
        "{} finding(s) shown. BrambleRS changes nothing: every action above is for you to take.",
        shown.len()
    );
}

fn sev_tag(s: Severity) -> &'static str {
    match s {
        Severity::ActNow => "[1]",
        Severity::Review => "[2]",
        Severity::GoodToKnow => "[3]",
        Severity::Info => "[4]",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_characters_are_removed_from_human_lines() {
        let line = "x\u{1b}]52;c;ZXZpbA==\u{7}y\r\nz";
        let cleaned = safe(line);
        assert!(!cleaned.contains('\u{1b}'), "{cleaned:?}");
        assert!(!cleaned.contains('\r'));
        assert!(!cleaned.contains('\n'));
        assert!(cleaned.starts_with("x ]52"));
    }

    #[test]
    fn no_subcommand_is_the_saved_list() {
        let cli = Cli::try_parse_from(["bramblers"]).unwrap();
        assert!(cli.command.is_none());
    }

    #[test]
    fn report_takes_one_module_and_json() {
        let cli = Cli::try_parse_from(["bramblers", "report", "permitt", "--json"]).unwrap();
        assert!(cli.json);
        match cli.command.unwrap() {
            Command::Report { module } => assert_eq!(module.as_deref(), Some("permitt")),
            _ => panic!("not a report command"),
        }
    }

    #[test]
    fn every_fixture_round_trips_through_json() {
        for name in [
            "billfold",
            "cleanator",
            "lastrun",
            "linstruct",
            "mcproster",
            "permitt",
        ] {
            let text = match name {
                "billfold" => {
                    include_str!("../../bramblers-core/tests/fixtures/reports/billfold.json")
                }
                "cleanator" => {
                    include_str!("../../bramblers-core/tests/fixtures/reports/cleanator.json")
                }
                "lastrun" => {
                    include_str!("../../bramblers-core/tests/fixtures/reports/lastrun.json")
                }
                "linstruct" => {
                    include_str!("../../bramblers-core/tests/fixtures/reports/linstruct.json")
                }
                "mcproster" => {
                    include_str!("../../bramblers-core/tests/fixtures/reports/mcproster.json")
                }
                "permitt" => {
                    include_str!("../../bramblers-core/tests/fixtures/reports/permitt.json")
                }
                _ => unreachable!(),
            };
            let report = bramblers_report::conformance::check(text)
                .unwrap_or_else(|e| panic!("{name}: {e:?}"));
            let again: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
            let original: serde_json::Value = serde_json::from_str(text).unwrap();
            assert_eq!(again, original, "{name}");
        }
    }
}
