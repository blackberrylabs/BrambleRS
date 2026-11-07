//! The report contract between the Blackberrylabs control tower and its modules.
//!
//! A module (Cleanator, MCProster, Permitt, …) answers `<module> report --json` with one
//! `bbl-report/1` document on stdout. The tower reads those documents, merges them into one
//! sorted list of findings and opens the module at the right place. This crate is the only
//! thing the two sides share: the types of that document, their JSON form, a validation that
//! lists every defect, the four-level severity scale in words, and a conformance check every
//! module runs on its own output.
//!
//! What this crate does **not** do: read a file, write a file, touch the network, mask a
//! secret, or interpret a finding. A module puts in its report only what it already shows on
//! screen, secrets masked upstream; the tower displays actions, it never runs them.
//!
//! # Compatibility
//!
//! Readers ignore unknown fields and every list or optional field has a default, so a field
//! *added* to the report keeps the schema at `bbl-report/1`. A field removed or whose meaning
//! changes becomes `bbl-report/2`, and the tower reads both. [`peek_schema`] reads the schema
//! string before any typed parsing.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod conformance;

use serde::{Deserialize, Serialize};
use std::fmt;

/// The schema this crate produces and validates.
pub const SCHEMA: &str = "bbl-report/1";

/// One report: what a module found, ready for the tower.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// Always [`SCHEMA`] for documents produced by this crate.
    pub schema: String,
    /// The module that produced the report.
    pub module: Module,
    /// RFC 3339 UTC timestamp, e.g. `2026-10-16T21:03:00Z`. See [`now_utc`].
    pub generated_at: String,
    /// How long the module took to produce the report.
    #[serde(default)]
    pub duration_ms: u64,
    /// One to four figures, the ones the module shows on its KPI cards.
    #[serde(default)]
    pub summary: Vec<Summary>,
    /// What the module found, in the module's own order; the tower sorts by severity.
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// Short free sentences, e.g. `3 shell files read`.
    #[serde(default)]
    pub notes: Vec<String>,
    /// What could not be read, as short sentences.
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Who produced the report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    /// Stable slug: lowercase ASCII letters, digits and `-`, e.g. `permitt`.
    pub id: String,
    /// Display name, e.g. `Permitt`.
    pub name: String,
    /// The module's own version, e.g. `0.1.0`.
    pub version: String,
    /// What the module does, one word: `clean`, `inventory`, `inspect`, …
    pub verb: String,
    /// One line, as shown on the tower's module card.
    #[serde(default)]
    pub tagline: String,
    /// Binary name of the desktop app, if the module has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// Binary name of the command line, if the module has one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cli: Option<String>,
    /// Home page, shown and never followed by the tower.
    #[serde(default)]
    pub home: String,
}

/// One KPI figure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Summary {
    /// Stable key inside the module, e.g. `hosts`.
    pub key: String,
    /// Label as shown, e.g. `Hosts installed`.
    pub label: String,
    /// The figure, already formatted by the module, e.g. `4` or `1.2 GB`.
    pub value: String,
    /// Colour hint, named after the shared palette.
    #[serde(default)]
    pub tone: Tone,
}

/// Colour hint for a figure; the names of the `bbl-ui` palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tone {
    /// Nothing to say about it.
    #[default]
    Neutral,
    /// Good news.
    Good,
    /// Worth a look.
    Amber,
    /// Something is wrong.
    Alert,
}

/// The four common severities. Each module maps its own findings onto this scale; the tower
/// never reinterprets. Serialised as the number, named in words here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum Severity {
    /// 1 — act now: data loss possible, a secret in clear, a bypass in a shell file.
    ActNow = 1,
    /// 2 — review: acts without asking, broad access, a large recoverable volume.
    Review = 2,
    /// 3 — good to know: a duplicate, a version that differs, a deny list alone.
    GoodToKnow = 3,
    /// 4 — information: disabled, not cached, not readable.
    Info = 4,
}

impl Severity {
    /// All four, strongest first.
    pub const ALL: [Severity; 4] = [Severity::ActNow, Severity::Review, Severity::GoodToKnow, Severity::Info];

    /// The number on the wire: 1 to 4.
    pub fn number(self) -> u8 {
        self as u8
    }

    /// Short slug: `act-now`, `review`, `good-to-know`, `info`.
    pub fn name(self) -> &'static str {
        match self {
            Severity::ActNow => "act-now",
            Severity::Review => "review",
            Severity::GoodToKnow => "good-to-know",
            Severity::Info => "info",
        }
    }

    /// The scale in words, as agreed between the tower and the modules.
    pub fn meaning(self) -> &'static str {
        match self {
            Severity::ActNow => "act now: data loss possible, a secret in clear, a bypass in a shell file",
            Severity::Review => "review: acts without asking, broad access, a large recoverable volume",
            Severity::GoodToKnow => "good to know: a duplicate, a version that differs, a deny list alone",
            Severity::Info => "information: disabled, not cached, not readable",
        }
    }
}

impl TryFrom<u8> for Severity {
    type Error = String;
    fn try_from(n: u8) -> Result<Self, Self::Error> {
        match n {
            1 => Ok(Severity::ActNow),
            2 => Ok(Severity::Review),
            3 => Ok(Severity::GoodToKnow),
            4 => Ok(Severity::Info),
            other => Err(format!("severity must be 1 to 4, got {other}")),
        }
    }
}

impl From<Severity> for u8 {
    fn from(s: Severity) -> u8 {
        s.number()
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// One finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    /// Stable inside the module across runs, e.g. `runs-without-asking/opencode/commands`;
    /// the tower uses it to tell a finding already seen from a new one.
    pub id: String,
    /// Where it sits on the common scale.
    pub severity: Severity,
    /// The module's own slug for this kind of finding, e.g. `runs-without-asking`.
    pub kind: String,
    /// One line, as shown in the list.
    pub title: String,
    /// What the finding is about.
    pub subject: Subject,
    /// A short paragraph, shown in the detail panel.
    #[serde(default)]
    pub detail: String,
    /// Where the module saw it.
    #[serde(default)]
    pub evidence: Vec<Evidence>,
    /// What the person could do about it — shown, never run.
    #[serde(default)]
    pub actions: Vec<Action>,
    /// How to open the module at this finding, if the module has an app.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<Open>,
}

/// What a finding is about.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subject {
    /// Family of subject.
    pub kind: SubjectKind,
    /// As shown, e.g. `OpenCode`, `~/.cache/pip`, `github (Claude Code)`.
    pub label: String,
}

/// Families of subject the tower knows how to label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubjectKind {
    /// An AI host (editor, CLI, agent).
    Host,
    /// An MCP server or similar declared service.
    Server,
    /// A directory on disk.
    Path,
    /// A single file.
    File,
    /// A setting key.
    Setting,
    /// Anything else; `label` carries the meaning.
    Other,
}

/// Where the module saw a finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    /// As shown, e.g. `~/.config/opencode/opencode.json` or `defaults`.
    pub label: String,
    /// Absolute path when there is a file behind the label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// 1-based line in that file, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

/// Something the person could do. The tower shows it; it never runs it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Action {
    /// What kind of action.
    pub kind: ActionKind,
    /// As shown, e.g. `Open file`, `Add a permission block in ~/.config/opencode/opencode.json`.
    pub label: String,
    /// The file to open, the URL to show, or the command to copy; absent for a hint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Kinds of action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionKind {
    /// A sentence; nothing to click.
    Hint,
    /// Open `target` with the system's default application.
    OpenFile,
    /// Show `target` as a link (the tower never follows it by itself).
    OpenUrl,
    /// A command line to copy; `target` is the command.
    Command,
}

/// How to open the module at a finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Open {
    /// Binary of the app, e.g. `permitt-app`.
    pub app: String,
    /// Arguments, e.g. `["--select", "opencode:commands"]`.
    #[serde(default)]
    pub args: Vec<String>,
}

impl Report {
    /// An empty report for `module`, stamped now.
    pub fn new(module: Module) -> Report {
        Report {
            schema: SCHEMA.to_string(),
            module,
            generated_at: now_utc(),
            duration_ms: 0,
            summary: Vec::new(),
            findings: Vec::new(),
            notes: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Pretty JSON with a trailing newline, as written on stdout.
    pub fn to_json(&self) -> String {
        let mut s = serde_json::to_string_pretty(self).expect("a report always serialises");
        s.push('\n');
        s
    }

    /// Parse without validating. Unknown fields are ignored.
    pub fn from_json(json: &str) -> Result<Report, String> {
        serde_json::from_str(json).map_err(|e| e.to_string())
    }

    /// Every defect, or nothing. The tower and the modules both call this.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        if self.schema != SCHEMA {
            errors.push(format!("schema: expected {SCHEMA:?}, got {:?}", self.schema));
        }
        if !is_slug(&self.module.id) {
            errors.push(format!("module.id: {:?} is not a slug (lowercase letters, digits, '-')", self.module.id));
        }
        for (field, value) in [
            ("module.name", &self.module.name),
            ("module.version", &self.module.version),
            ("module.verb", &self.module.verb),
        ] {
            if value.trim().is_empty() {
                errors.push(format!("{field}: empty"));
            }
        }
        if let Some(app) = &self.module.app {
            if app.trim().is_empty() {
                errors.push("module.app: empty (omit it instead)".to_string());
            }
        }
        if let Some(cli) = &self.module.cli {
            if cli.trim().is_empty() {
                errors.push("module.cli: empty (omit it instead)".to_string());
            }
        }
        if !is_rfc3339_utc(&self.generated_at) {
            errors.push(format!("generated_at: {:?} is not RFC 3339 UTC (YYYY-MM-DDTHH:MM:SSZ)", self.generated_at));
        }
        if self.summary.is_empty() || self.summary.len() > 4 {
            errors.push(format!("summary: 1 to 4 figures expected, got {}", self.summary.len()));
        }
        let mut keys = std::collections::HashSet::new();
        for (i, s) in self.summary.iter().enumerate() {
            if s.key.trim().is_empty() {
                errors.push(format!("summary[{i}].key: empty"));
            } else if !keys.insert(s.key.as_str()) {
                errors.push(format!("summary[{i}].key: {:?} appears twice", s.key));
            }
            if s.label.trim().is_empty() {
                errors.push(format!("summary[{i}].label: empty"));
            }
        }
        let mut ids = std::collections::HashSet::new();
        for (i, f) in self.findings.iter().enumerate() {
            if f.id.trim().is_empty() {
                errors.push(format!("findings[{i}].id: empty"));
            } else if !ids.insert(f.id.as_str()) {
                errors.push(format!("findings[{i}].id: {:?} appears twice", f.id));
            }
            if f.kind.trim().is_empty() {
                errors.push(format!("findings[{i}].kind: empty"));
            }
            if f.title.trim().is_empty() {
                errors.push(format!("findings[{i}].title: empty"));
            }
            if f.subject.label.trim().is_empty() {
                errors.push(format!("findings[{i}].subject.label: empty"));
            }
            for (j, e) in f.evidence.iter().enumerate() {
                if e.label.trim().is_empty() {
                    errors.push(format!("findings[{i}].evidence[{j}].label: empty"));
                }
                if e.line.is_some() && e.path.is_none() {
                    errors.push(format!("findings[{i}].evidence[{j}]: a line without a path"));
                }
            }
            for (j, a) in f.actions.iter().enumerate() {
                if a.label.trim().is_empty() {
                    errors.push(format!("findings[{i}].actions[{j}].label: empty"));
                }
                let needs_target = !matches!(a.kind, ActionKind::Hint);
                if needs_target && a.target.as_deref().map_or(true, |t| t.trim().is_empty()) {
                    errors.push(format!("findings[{i}].actions[{j}]: {:?} action without a target", a.kind));
                }
            }
            if let Some(open) = &f.open {
                if open.app.trim().is_empty() {
                    errors.push(format!("findings[{i}].open.app: empty"));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// The `schema` string of a document, read before any typed parsing — so the tower can tell
/// a `bbl-report/2` from garbage without failing on fields it does not know.
pub fn peek_schema(json: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    v.get("schema")?.as_str().map(str::to_string)
}

fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !s.starts_with('-')
        && !s.ends_with('-')
}

/// `YYYY-MM-DDTHH:MM:SSZ`, digits in the right places and fields in range.
fn is_rfc3339_utc(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 20 || b[4] != b'-' || b[7] != b'-' || b[10] != b'T' || b[13] != b':' || b[16] != b':' || b[19] != b'Z' {
        return false;
    }
    let num = |from: usize, to: usize| -> Option<u32> { s.get(from..to)?.parse().ok() };
    let (Some(_y), Some(mo), Some(d), Some(h), Some(mi), Some(se)) =
        (num(0, 4), num(5, 7), num(8, 10), num(11, 13), num(14, 16), num(17, 19))
    else {
        return false;
    };
    (1..=12).contains(&mo) && (1..=31).contains(&d) && h < 24 && mi < 60 && se < 60
}

/// The current time as RFC 3339 UTC, `YYYY-MM-DDTHH:MM:SSZ`, with no date dependency.
pub fn now_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    rfc3339_from_unix(secs)
}

/// Unix seconds → `YYYY-MM-DDTHH:MM:SSZ` (proleptic Gregorian, Howard Hinnant's algorithm).
pub fn rfc3339_from_unix(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, mi, se) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{se:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_dates() {
        assert_eq!(rfc3339_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(rfc3339_from_unix(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(rfc3339_from_unix(1_792_184_580), "2026-10-16T21:03:00Z");
        assert_eq!(rfc3339_from_unix(4_102_444_799), "2099-12-31T23:59:59Z");
    }

    #[test]
    fn timestamp_shape() {
        assert!(is_rfc3339_utc("2026-10-16T21:03:00Z"));
        assert!(!is_rfc3339_utc("2026-10-16T21:03:00+02:00"));
        assert!(!is_rfc3339_utc("2026-13-01T00:00:00Z"));
        assert!(!is_rfc3339_utc("yesterday"));
        assert!(is_rfc3339_utc(&now_utc()));
    }

    #[test]
    fn slugs() {
        assert!(is_slug("permitt"));
        assert!(is_slug("bbl-report"));
        assert!(!is_slug("Permitt"));
        assert!(!is_slug("-x"));
        assert!(!is_slug(""));
    }

    #[test]
    fn severity_on_the_wire() {
        assert_eq!(serde_json::to_string(&Severity::Review).unwrap(), "2");
        assert_eq!(serde_json::from_str::<Severity>("4").unwrap(), Severity::Info);
        assert!(serde_json::from_str::<Severity>("0").is_err());
        assert!(serde_json::from_str::<Severity>("5").is_err());
        assert!(Severity::ActNow < Severity::Info);
        for s in Severity::ALL {
            assert_eq!(Severity::try_from(s.number()).unwrap(), s);
            assert!(!s.meaning().is_empty());
        }
    }
}
