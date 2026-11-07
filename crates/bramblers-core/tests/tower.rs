//! BrambleRS core on the real report fixtures: saved reports are read with their age and
//! sorted across modules; a report of another schema is named, not fatal; applying a run
//! writes under the state directory only; an unknown module is refused. Nothing is launched.

use std::path::{Path, PathBuf};
use std::time::Duration;

use bramblers_core::dashboard::ReportState;
use bramblers_core::{Cache, Dashboard, Run, RunError};
use bramblers_report::Severity;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/reports")
            .join(name),
    )
    .unwrap()
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out);
            } else {
                out.push(p);
            }
        }
    }
}

#[test]
fn saved_reports_are_read_with_their_age_and_sorted_across_modules() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = Cache::at(tmp.path());
    // Nothing saved yet: six modules, no report, no failure.
    let d = Dashboard::from_saved(&cache);
    assert_eq!(d.modules.len(), 6);
    let ids: Vec<&str> = d.modules.iter().map(|m| m.provider.id).collect();
    assert_eq!(
        ids,
        [
            "billfold",
            "cleanator",
            "lastrun",
            "linstruct",
            "mcproster",
            "permitt"
        ]
    );
    assert!(d
        .modules
        .iter()
        .all(|m| m.state == ReportState::NoReport && m.report.is_none()));
    assert!(d.lines.is_empty());

    // Two saved reports, read with their age.
    cache.write("permitt", &fixture("permitt.json")).unwrap();
    cache
        .write("mcproster", &fixture("mcproster.json"))
        .unwrap();
    let d = Dashboard::from_saved(&cache);
    assert!(matches!(
        d.module("permitt").unwrap().state,
        ReportState::Saved { .. }
    ));
    assert!(matches!(
        d.module("mcproster").unwrap().state,
        ReportState::Saved { .. }
    ));
    assert_eq!(d.lines.len(), 40 + 23);
    let sev: Vec<u8> = d
        .lines
        .iter()
        .map(|l| l.finding.severity.number())
        .collect();
    let mut sorted = sev.clone();
    sorted.sort();
    assert_eq!(sev, sorted);
    let first = &d.lines[0];
    assert_eq!(first.finding.severity, Severity::ActNow);
    assert_eq!(
        first.module, "mcproster",
        "same severity: family order first"
    );
    let counts = d.counts();
    assert_eq!(counts[0].0, Severity::ActNow);
    assert_eq!(counts.iter().map(|(_, n)| n).sum::<usize>(), d.lines.len());
    // Figures: one summary row per module card.
    assert_eq!(d.summaries().len(), 8);
}

#[test]
fn a_report_of_another_schema_is_named_not_fatal() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = Cache::at(tmp.path());
    cache
        .write(
            "permitt",
            r#"{ "schema": "bramblers-report/2", "module": { "id": "permitt" } }"#,
        )
        .unwrap();
    let d = Dashboard::from_saved(&cache);
    match &d.module("permitt").unwrap().state {
        ReportState::Failed { error } => assert!(error.contains("bramblers-report/2"), "{error}"),
        other => panic!("{other:?}"),
    }
    assert!(d.lines.iter().all(|l| l.module != "permitt"));
}

#[test]
fn applying_a_run_writes_under_the_state_dir_only() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = Cache::at(tmp.path());
    let json = fixture("permitt.json");
    let report = bramblers_report::conformance::check(&json).unwrap();
    let run = Run {
        report,
        json,
        elapsed: Duration::from_millis(12),
    };
    let mut d = Dashboard::from_saved(&cache);
    d.apply("permitt", &Ok(run), &cache);
    assert!(matches!(
        d.module("permitt").unwrap().state,
        ReportState::Fresh { .. }
    ));
    let mut files = Vec::new();
    walk(tmp.path(), &mut files);
    assert_eq!(files, vec![cache.path("permitt")]);
    assert!(d.to_json().contains("\"state\": \"fresh\""));
}

#[test]
fn an_unknown_module_is_refused() {
    let tmp = tempfile::tempdir().unwrap();
    let cache = Cache::at(tmp.path());
    let mut d = Dashboard::from_saved(&cache);
    match d.refresh("nope", &cache) {
        Err(RunError::Failed(e)) => assert!(e.contains("unknown module"), "{e}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_real_reports_conform() {
    for name in [
        "billfold.json",
        "cleanator.json",
        "lastrun.json",
        "linstruct.json",
        "mcproster.json",
        "permitt.json",
    ] {
        let r = bramblers_report::conformance::check(&fixture(name))
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert!(!r.findings.is_empty(), "{name}: no findings");
        assert!(
            !fixture(name).contains("/tmp/"),
            "{name}: a scratch path leaked into the fixture"
        );
    }
}
