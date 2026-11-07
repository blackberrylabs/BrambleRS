//! The tower on fake modules: shell scripts named like the real binaries, each printing one
//! of the real reports (`tests/fixtures/reports/*.json`, paths scrubbed). What is
//! checked: an absent module is shown, a saved report is read with its age, a refresh writes
//! only under the state directory (R2), a report of another schema or with extra fields is
//! read without breaking, a failing module is reported not crashed.

use std::path::{Path, PathBuf};

use bramblers_core::dashboard::ReportState;
use bramblers_core::{Cache, Dashboard, Modules};
use bramblers_report::Severity;

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/reports")
            .join(name),
    )
    .unwrap()
}

/// A fake module binary: prints `body` on stdout, exits with `code`.
#[cfg(unix)]
fn fake_bin(dir: &Path, name: &str, body_file: &Path, code: i32) {
    use std::os::unix::fs::PermissionsExt;
    let script = format!(
        "#!/bin/sh\n[ \"$1\" = report ] || exit 2\ncat '{}'\nexit {code}\n",
        body_file.display()
    );
    let p = dir.join(name);
    std::fs::write(&p, script).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
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

struct World {
    _tmp: tempfile::TempDir,
    bin: PathBuf,
    state: PathBuf,
    root: PathBuf,
}

/// A temp root with `bin/` (fake Permitt and MCProster, no Cleanator) and `state/`.
fn world() -> World {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    let bin = root.join("bin");
    let data = root.join("data");
    let state = root.join("state");
    std::fs::create_dir_all(&bin).unwrap();
    std::fs::create_dir_all(&data).unwrap();
    for (module, cli) in [("permitt", "permitt"), ("mcproster", "mcproster")] {
        let body = data.join(format!("{module}.json"));
        std::fs::write(&body, fixture(&format!("{module}.json"))).unwrap();
        fake_bin(&bin, cli, &body, 0);
    }
    World {
        _tmp: tmp,
        bin,
        state,
        root,
    }
}

#[test]
fn an_absent_module_is_shown_and_saved_reports_are_read() {
    let w = world();
    let cache = Cache::at(&w.state);
    let modules = Modules::embedded();
    // Nothing saved yet.
    let d = Dashboard::from_saved(&modules, &cache, std::slice::from_ref(&w.bin));
    assert_eq!(d.modules.len(), 6);
    assert_eq!(
        d.module("linstruct").unwrap().state,
        ReportState::NotInstalled
    );
    assert_eq!(
        d.module("cleanator").unwrap().state,
        ReportState::NotInstalled
    );
    assert!(!d.module("cleanator").unwrap().installed);
    assert_eq!(d.module("permitt").unwrap().state, ReportState::NoReport);
    assert!(d.lines.is_empty());
    // A saved report, read with its age.
    cache.write("permitt", &fixture("permitt.json")).unwrap();
    let d = Dashboard::from_saved(&modules, &cache, std::slice::from_ref(&w.bin));
    assert!(matches!(
        d.module("permitt").unwrap().state,
        ReportState::Saved { .. }
    ));
    assert_eq!(d.lines.len(), 40);
    assert!(d.lines.iter().all(|l| l.module == "permitt"));
}

#[test]
fn refresh_writes_under_the_state_dir_only_and_sorts_across_modules() {
    let w = world();
    let cache = Cache::at(&w.state);
    let modules = Modules::embedded();
    let mut before = Vec::new();
    walk(&w.root, &mut before);
    before.sort();

    let mut d = Dashboard::from_saved(&modules, &cache, std::slice::from_ref(&w.bin));
    let results = d.refresh_all(&cache);
    assert_eq!(results.len(), 2, "two installed modules, Cleanator absent");
    assert!(results.iter().all(|(_, r)| r.is_ok()), "{results:?}");

    // R2: the only new files are the two saved reports.
    let mut after = Vec::new();
    walk(&w.root, &mut after);
    after.sort();
    let new: Vec<&PathBuf> = after.iter().filter(|p| !before.contains(p)).collect();
    assert_eq!(
        new,
        vec![
            &w.state.join("mcproster/report.json"),
            &w.state.join("permitt/report.json")
        ]
    );
    assert!(new
        .iter()
        .all(|p| bramblers_core::cache::is_under(p, &w.state)));

    // Both fresh, lines merged and sorted: severity, then family order (MCProster before
    // Permitt), then title.
    assert!(matches!(
        d.module("permitt").unwrap().state,
        ReportState::Fresh { .. }
    ));
    assert!(matches!(
        d.module("mcproster").unwrap().state,
        ReportState::Fresh { .. }
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
    // A second dashboard from the saved files sees the same thing.
    let d2 = Dashboard::from_saved(&modules, &cache, std::slice::from_ref(&w.bin));
    assert_eq!(d2.lines.len(), d.lines.len());
}

#[test]
fn another_schema_and_extra_fields_do_not_break() {
    let w = world();
    let cache = Cache::at(&w.state);
    let modules = Modules::embedded();
    // Extra fields: read as if they were not there.
    let mut v: serde_json::Value = serde_json::from_str(&fixture("mcproster.json")).unwrap();
    v["future"] = serde_json::json!({ "anything": [1, 2, 3] });
    v["findings"][0]["confidence"] = serde_json::json!(0.7);
    cache.write("mcproster", &v.to_string()).unwrap();
    // Another schema: named, not parsed, not fatal.
    cache
        .write(
            "permitt",
            r#"{ "schema": "bbl-report/2", "module": { "id": "permitt" } }"#,
        )
        .unwrap();
    let d = Dashboard::from_saved(&modules, &cache, std::slice::from_ref(&w.bin));
    assert!(matches!(
        d.module("mcproster").unwrap().state,
        ReportState::Saved { .. }
    ));
    assert_eq!(
        d.lines.iter().filter(|l| l.module == "mcproster").count(),
        23
    );
    match &d.module("permitt").unwrap().state {
        ReportState::Failed { error } => assert!(error.contains("bbl-report/2"), "{error}"),
        other => panic!("{other:?}"),
    }
    assert!(d.lines.iter().all(|l| l.module != "permitt"));
}

#[test]
fn a_failing_module_is_reported_not_crashed() {
    let w = world();
    let cache = Cache::at(&w.state);
    let modules = Modules::embedded();
    // Permitt now exits 1 with garbage; its saved report from before stays.
    cache.write("permitt", &fixture("permitt.json")).unwrap();
    let garbage = w.root.join("data/garbage.txt");
    std::fs::write(&garbage, "boom\n").unwrap();
    fake_bin(&w.bin, "permitt", &garbage, 1);
    let mut d = Dashboard::from_saved(&modules, &cache, std::slice::from_ref(&w.bin));
    let r = d.refresh("permitt", &cache);
    assert!(
        matches!(r, Err(bramblers_core::run::RunError::Exit(1, _))),
        "{r:?}"
    );
    assert!(matches!(
        d.module("permitt").unwrap().state,
        ReportState::Failed { .. }
    ));
    assert_eq!(d.lines.len(), 40, "the previous report is still shown");
    // A module that answers but not with a report.
    fake_bin(&w.bin, "mcproster", &garbage, 0);
    let r = d.refresh("mcproster", &cache);
    assert!(
        matches!(r, Err(bramblers_core::run::RunError::NotAReport(_))),
        "{r:?}"
    );
    // Refreshing an absent module says so.
    let r = d.refresh("cleanator", &cache);
    assert!(
        matches!(r, Err(bramblers_core::run::RunError::Spawn(ref e)) if e.contains("not installed")),
        "{r:?}"
    );
    // The dashboard still serialises.
    let json = d.to_json();
    assert!(json.contains("\"state\": \"failed\""));
}

#[test]
fn the_real_reports_conform_and_open_their_module() {
    for (name, app) in [
        ("billfold.json", "billfold-app"),
        ("cleanator.json", "cleanator"),
        ("lastrun.json", "lastrun-app"),
        ("linstruct.json", "linstruct-app"),
        ("mcproster.json", "mcproster-app"),
        ("permitt.json", "permitt-app"),
    ] {
        let r = bramblers_report::conformance::check(&fixture(name))
            .unwrap_or_else(|e| panic!("{name}: {e:?}"));
        assert_eq!(r.module.app.as_deref(), Some(app));
        for f in r.findings.iter().filter(|f| f.open.is_some()) {
            assert_eq!(f.open.as_ref().unwrap().app, app);
        }
        assert!(
            !fixture(name).contains("/tmp/"),
            "{name}: a scratch path leaked into the fixture"
        );
    }
}
