//! What the crate promises: round trip, forward compatibility, every validation rule fires
//! once, and no file or network access anywhere in the code.

use bbl_report::conformance::{check, example};
use bbl_report::*;

#[test]
fn round_trip() {
    let r = example();
    assert_eq!(r.validate(), Ok(()));
    let json = r.to_json();
    assert!(json.ends_with('\n'));
    let back = check(&json).expect("the example conforms");
    assert_eq!(back, r);
}

#[test]
fn unknown_fields_are_ignored() {
    let mut v: serde_json::Value = serde_json::from_str(&example().to_json()).unwrap();
    v["weather"] = "sunny".into();
    v["module"]["mascot"] = "blackberry".into();
    v["findings"][0]["confidence"] = 0.9.into();
    v["findings"][0]["subject"]["colour"] = "purple".into();
    let r = check(&v.to_string()).expect("a field added keeps the schema readable");
    assert_eq!(r, example());
}

#[test]
fn optional_lists_default() {
    let json = r#"{
      "schema": "bbl-report/1",
      "module": { "id": "x", "name": "X", "version": "0.0.1", "verb": "test" },
      "generated_at": "2026-10-16T00:00:00Z",
      "summary": [ { "key": "k", "label": "K", "value": "0" } ]
    }"#;
    let r = check(json).expect("minimal report");
    assert!(r.findings.is_empty() && r.notes.is_empty() && r.errors.is_empty());
    assert_eq!(r.duration_ms, 0);
    assert_eq!(r.summary[0].tone, Tone::Neutral);
    assert_eq!(r.module.app, None);
}

#[test]
fn another_schema_is_named_not_parsed() {
    let json = r#"{ "schema": "bbl-report/2", "module": 42 }"#;
    assert_eq!(peek_schema(json).as_deref(), Some("bbl-report/2"));
    let err = check(json).unwrap_err();
    assert_eq!(err.len(), 1);
    assert!(err[0].contains("bbl-report/2"), "{err:?}");
    assert_eq!(peek_schema("not json"), None);
    assert_eq!(peek_schema(r#"{ "schema": 1 }"#), None);
    assert!(check("not json").is_err());
}

fn expect_error(mutate: impl FnOnce(&mut Report), needle: &str) {
    let mut r = example();
    mutate(&mut r);
    let errors = r.validate().expect_err(needle);
    assert!(errors.iter().any(|e| e.contains(needle)), "expected {needle:?} in {errors:?}");
}

#[test]
fn every_rule_fires() {
    expect_error(|r| r.schema = "bbl-report/0".into(), "schema");
    expect_error(|r| r.module.id = "Permitt".into(), "module.id");
    expect_error(|r| r.module.name = " ".into(), "module.name");
    expect_error(|r| r.module.version = "".into(), "module.version");
    expect_error(|r| r.module.verb = "".into(), "module.verb");
    expect_error(|r| r.module.app = Some("".into()), "module.app");
    expect_error(|r| r.module.cli = Some("".into()), "module.cli");
    expect_error(|r| r.generated_at = "2026-10-16 21:03".into(), "generated_at");
    expect_error(|r| r.summary.clear(), "summary: 1 to 4");
    expect_error(|r| r.summary = vec![r.summary[0].clone(); 5], "summary: 1 to 4");
    expect_error(|r| r.summary = vec![r.summary[0].clone(), r.summary[0].clone()], "appears twice");
    expect_error(|r| r.summary[0].key = "".into(), "summary[0].key");
    expect_error(|r| r.summary[0].label = "".into(), "summary[0].label");
    expect_error(|r| r.findings[0].id = "".into(), "findings[0].id");
    expect_error(|r| r.findings.push(r.findings[0].clone()), "findings[1].id");
    expect_error(|r| r.findings[0].kind = "".into(), "findings[0].kind");
    expect_error(|r| r.findings[0].title = "".into(), "findings[0].title");
    expect_error(|r| r.findings[0].subject.label = "".into(), "subject.label");
    expect_error(|r| r.findings[0].evidence[0].label = "".into(), "evidence[0].label");
    expect_error(
        |r| {
            r.findings[0].evidence[0].path = None;
            r.findings[0].evidence[0].line = Some(3);
        },
        "a line without a path",
    );
    expect_error(|r| r.findings[0].actions[0].label = "".into(), "actions[0].label");
    expect_error(|r| r.findings[0].actions[1].target = None, "without a target");
    expect_error(|r| r.findings[0].open = Some(Open { app: "".into(), args: vec![] }), "open.app");
}

#[test]
fn all_defects_are_listed_at_once() {
    let mut r = example();
    r.schema = "x".into();
    r.module.id = "X".into();
    r.summary.clear();
    let errors = r.validate().unwrap_err();
    assert_eq!(errors.len(), 3, "{errors:?}");
}

#[test]
fn severity_scale_in_words() {
    assert_eq!(Severity::ALL.map(|s| s.number()), [1, 2, 3, 4]);
    assert_eq!(Severity::ActNow.name(), "act-now");
    assert_eq!(Severity::GoodToKnow.to_string(), "good-to-know");
    let json = example().to_json();
    assert!(json.contains("\"severity\": 2"), "{json}");
    assert!(json.contains("\"kind\": \"open-file\""));
    assert!(json.contains("\"tone\": \"alert\""));
}

#[test]
fn new_report_is_stamped_and_valid_once_summarised() {
    let mut r = Report::new(example().module);
    assert_eq!(r.schema, SCHEMA);
    assert!(r.validate().is_err(), "no summary yet");
    r.summary.push(Summary { key: "n".into(), label: "N".into(), value: "1".into(), tone: Tone::Good });
    assert_eq!(r.validate(), Ok(()));
}

/// The crate reads nothing and writes nothing: no `std::fs`, no `std::net`, no process
/// spawning, anywhere in its source.
#[test]
fn no_io_in_the_crate() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        for forbidden in ["std::fs", "std::net", "std::process", "fs::", "File::", "Command::", "TcpStream"] {
            assert!(!text.contains(forbidden), "{} mentions {forbidden}", path.display());
        }
    }
    let manifest = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let deps: Vec<&str> = manifest
        .lines()
        .skip_while(|l| !l.starts_with("[dependencies]"))
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .filter(|l| l.contains('='))
        .map(|l| l.split('=').next().unwrap().trim())
        .collect();
    assert_eq!(deps, ["serde", "serde_json"], "serde and serde_json only");
}
