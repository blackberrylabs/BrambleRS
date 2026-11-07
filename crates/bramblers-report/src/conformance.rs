//! The conformance check every module runs on its own `report --json` output, and the
//! tower runs on what it reads.

use crate::{peek_schema, Report, SCHEMA};

/// Parse and validate one document. Every defect is listed; a document of another schema is
/// refused with that schema named, so the caller can decide what to do with it.
pub fn check(json: &str) -> Result<Report, Vec<String>> {
    match peek_schema(json) {
        None => return Err(vec!["schema: missing, not a string, or the document is not JSON".to_string()]),
        Some(s) if s != SCHEMA => return Err(vec![format!("schema: {s:?} is not {SCHEMA:?}; this crate reads {SCHEMA} only")]),
        Some(_) => {}
    }
    let report = Report::from_json(json).map_err(|e| vec![format!("parse: {e}")])?;
    report.validate()?;
    Ok(report)
}

/// A minimal valid report, for tests and examples.
pub fn example() -> Report {
    use crate::*;
    Report {
        schema: SCHEMA.to_string(),
        module: Module {
            id: "permitt".into(),
            name: "Permitt".into(),
            version: "0.1.0".into(),
            verb: "inspect".into(),
            tagline: "What your AI agents may do without asking".into(),
            app: Some("permitt-app".into()),
            cli: Some("permitt".into()),
            home: "https://github.com/blackberrylabs/permitt".into(),
        },
        generated_at: "2026-10-16T21:03:00Z".into(),
        duration_ms: 120,
        summary: vec![
            Summary { key: "hosts".into(), label: "Hosts installed".into(), value: "4".into(), tone: Tone::Neutral },
            Summary { key: "auto".into(), label: "Run without asking".into(), value: "6".into(), tone: Tone::Alert },
        ],
        findings: vec![Finding {
            id: "runs-without-asking/opencode/commands".into(),
            severity: Severity::Review,
            kind: "runs-without-asking".into(),
            title: "OpenCode runs shell commands without asking".into(),
            subject: Subject { kind: SubjectKind::Host, label: "OpenCode".into() },
            detail: "by default, nothing changed it".into(),
            evidence: vec![Evidence {
                label: "~/.config/opencode/opencode.json".into(),
                path: Some("/home/me/.config/opencode/opencode.json".into()),
                line: None,
            }],
            actions: vec![
                Action { kind: ActionKind::Hint, label: "Add a permission block in ~/.config/opencode/opencode.json".into(), target: None },
                Action { kind: ActionKind::OpenFile, label: "Open file".into(), target: Some("/home/me/.config/opencode/opencode.json".into()) },
            ],
            open: Some(Open { app: "permitt-app".into(), args: vec!["--select".into(), "opencode:commands".into()] }),
        }],
        notes: vec!["3 shell files read".into()],
        errors: vec![],
    }
}
