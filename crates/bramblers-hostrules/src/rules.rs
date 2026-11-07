//! Host rules (`hosts/*.toml`): data model, loading, validation. The common part of the
//! schema lives here; the fields a tool needs for its own job live in its [`Extension`],
//! flattened into every location. A location without a source is refused.

use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Where a location applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    Project,
    System,
}

/// Text format of a file a host reads. The four structured formats become a
/// `serde_json::Value` tree; `Markdown` is kept as text — a `Value::String` — for the
/// hosts' instruction files (CLAUDE.md, AGENTS.md, `.cursor/rules/*.mdc`…), which a tool
/// inspects as prose: lines, bytes, front matter, imports. `Jsonl` (one JSON document per
/// line: session logs, rollouts) becomes a `Value::Array` of the lines that parse; `Dotenv`
/// (`KEY=value` lines: `.env`, `environment.d`, `secrets.env`) becomes a `Value::Object` of
/// strings; `Sqlite` names a binary database a tool reads with its own driver — `parse`
/// refuses it, a location may still describe it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Json,
    Jsonc,
    Toml,
    Yaml,
    Markdown,
    Jsonl,
    Dotenv,
    Sqlite,
}

/// How a location was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    Documentation,
    SourceCode,
    Observed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Active,
    OutOfScope,
}

/// A string or a list of strings.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        let v: Vec<&str> = match self {
            OneOrMany::One(s) => vec![s.as_str()],
            OneOrMany::Many(v) => v.iter().map(String::as_str).collect(),
        };
        v.into_iter()
    }
}

/// Paths per operating system. `any` applies everywhere.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct PathSpec {
    pub any: Option<OneOrMany>,
    pub linux: Option<OneOrMany>,
    pub macos: Option<OneOrMany>,
    pub windows: Option<OneOrMany>,
}

impl PathSpec {
    pub fn is_empty(&self) -> bool {
        self.any.is_none() && self.linux.is_none() && self.macos.is_none() && self.windows.is_none()
    }
}

/// Where the path, format and keys of a location were read.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Source {
    pub kind: SourceKind,
    pub url: Option<String>,
    pub read_on: toml::value::Datetime,
    pub quote: String,
    pub note: Option<String>,
}

/// The fields a tool adds to every location. Flattened into the TOML table of the
/// location, next to the common fields. `validate` receives the whole location so a rule
/// can be checked against the common fields too (a role, a scope).
pub trait Extension: DeserializeOwned + Serialize + Clone + Debug {
    /// Tool-specific checks on one location. The error is the message only; the file name
    /// and the location index are added by the loader.
    fn validate(_loc: &Location<Self>) -> Result<(), String> {
        Ok(())
    }
}

/// The fields a tool adds to every host rule, next to the common ones (identity, hints,
/// locations): what the tool knows about the host as a whole — defaults, the meaning of
/// its settings, its precedence. `validate` receives the whole rule.
pub trait HostExtension: DeserializeOwned + Serialize + Clone + Debug {
    fn validate<X: Extension>(_rule: &HostRule<X, Self>) -> Result<(), String> {
        Ok(())
    }
}

/// No tool-specific fields: a rule that only says where a host keeps a file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NoExt {}

impl Extension for NoExt {}
impl HostExtension for NoExt {}

/// One file (or glob) where a host keeps something.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound = "X: Extension")]
pub struct Location<X: Extension> {
    pub scope: Scope,
    pub format: Format,
    pub path: PathSpec,
    /// For a per-project location whose project is a key *inside* a user file (Claude Code's
    /// local scope): the key path to the project table, the key being the project path.
    pub project_key: Option<String>,
    /// An older location the host still reads.
    #[serde(default)]
    pub legacy: bool,
    /// Other host ids that read this very file (one file, several hosts).
    #[serde(default)]
    pub shared_with: Vec<String>,
    /// `registry` / `settings` for a metadata file, `None` for the main file.
    pub role: Option<String>,
    pub source: Source,
    #[serde(flatten)]
    pub ext: X,
}

impl<X: Extension> Location<X> {
    pub fn is_registry(&self) -> bool {
        self.role.as_deref() == Some("registry")
    }
}

/// A host left out, with the reason.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotRead {
    pub id: String,
    pub name: String,
    pub reason: String,
}

/// One `hosts/*.toml` file. `X` is the tool's extension of every location, `H` its
/// extension of the host as a whole (none by default).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(bound = "X: Extension, H: HostExtension")]
pub struct HostRule<X: Extension, H: HostExtension = NoExt> {
    pub id: String,
    pub name: String,
    pub vendor: Option<String>,
    pub kind: String,
    pub status: Status,
    /// The application several rules belong to (Claude Desktop: configuration file and
    /// `.mcpb` extensions). Defaults to the host name.
    pub application: Option<String>,
    #[serde(default)]
    pub references: Vec<String>,
    /// Commands of the host a tool may show (`remove`, `list`, …), never run.
    #[serde(default)]
    pub hints: BTreeMap<String, String>,
    #[serde(default = "Vec::new")]
    pub locations: Vec<Location<X>>,
    #[serde(default)]
    pub hosts: Vec<NotRead>,
    #[serde(flatten)]
    pub ext: H,
}

impl<X: Extension, H: HostExtension> HostRule<X, H> {
    pub fn application(&self) -> &str {
        self.application.as_deref().unwrap_or(&self.name)
    }

    pub fn hint(&self, key: &str) -> Option<&str> {
        self.hints.get(key).map(String::as_str)
    }
}

#[derive(Debug, Error)]
pub enum RuleError {
    #[error("{file}: {source}")]
    Parse {
        file: String,
        #[source]
        source: toml::de::Error,
    },
    #[error("{file}: {message}")]
    Invalid { file: String, message: String },
}

/// A validated set of host rules.
#[derive(Debug, Clone)]
pub struct Rules<X: Extension, H: HostExtension = NoExt> {
    pub hosts: Vec<HostRule<X, H>>,
    pub not_read: Vec<NotRead>,
}

impl<X: Extension, H: HostExtension> Rules<X, H> {
    /// Parse and validate rule texts, `(file name, text)`. Hosts are sorted by name; an
    /// out-of-scope file feeds `not_read`.
    pub fn load<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Self, RuleError> {
        let mut hosts = Vec::new();
        let mut not_read = Vec::new();
        let mut ids = HashSet::new();
        for (file, text) in files {
            let rule: HostRule<X, H> = toml::from_str(text).map_err(|source| RuleError::Parse {
                file: file.to_string(),
                source,
            })?;
            validate(file, &rule)?;
            if !ids.insert(rule.id.clone()) {
                return Err(RuleError::Invalid {
                    file: file.to_string(),
                    message: format!("duplicate host id `{}`", rule.id),
                });
            }
            if rule.status == Status::OutOfScope {
                not_read.extend(rule.hosts.iter().cloned());
            } else {
                hosts.push(rule);
            }
        }
        hosts.sort_by_key(|h| h.name.to_lowercase());
        Ok(Rules { hosts, not_read })
    }

    pub fn host(&self, id: &str) -> Option<&HostRule<X, H>> {
        self.hosts.iter().find(|h| h.id == id)
    }

    /// The display name of a host id, or the id itself when unknown.
    pub fn name_of<'a>(&'a self, id: &'a str) -> &'a str {
        self.host(id).map(|h| h.name.as_str()).unwrap_or(id)
    }

    pub fn locations(&self) -> usize {
        self.hosts.iter().map(|h| h.locations.len()).sum()
    }
}

fn validate<X: Extension, H: HostExtension>(
    file: &str,
    rule: &HostRule<X, H>,
) -> Result<(), RuleError> {
    let invalid = |message: String| RuleError::Invalid {
        file: file.to_string(),
        message,
    };
    if rule.id.is_empty() || rule.name.is_empty() {
        return Err(invalid("id and name are required".into()));
    }
    match rule.status {
        Status::OutOfScope => {
            if rule.hosts.is_empty() {
                return Err(invalid(
                    "an out-of-scope list needs at least one host".into(),
                ));
            }
            for h in &rule.hosts {
                if h.reason.trim().is_empty() {
                    return Err(invalid(format!(
                        "host `{}` left out without a reason",
                        h.id
                    )));
                }
            }
        }
        Status::Active => {
            // a host with no readable file at all (its state lives on a server or in the
            // keyring) may still be described, as long as it points at its documentation
            if rule.locations.is_empty() && rule.references.is_empty() {
                return Err(invalid(
                    "an active host needs at least one location, or a reference when it keeps no readable file".into(),
                ));
            }
            for (i, loc) in rule.locations.iter().enumerate() {
                if loc.path.is_empty() {
                    return Err(invalid(format!("location {i}: no path")));
                }
                if loc.source.quote.trim().is_empty() {
                    return Err(invalid(format!(
                        "location {i}: a source needs the words it was read from"
                    )));
                }
                if loc.source.kind != SourceKind::Observed && loc.source.url.is_none() {
                    return Err(invalid(format!(
                        "location {i}: a documentation or source-code source needs a URL"
                    )));
                }
                X::validate(loc).map_err(|m| invalid(format!("location {i}: {m}")))?;
            }
        }
    }
    H::validate(rule).map_err(invalid)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = r#"
id = "x"
name = "X"
kind = "cli"
status = "active"
hints.list = "x list"
[[locations]]
scope = "user"
format = "json"
path.any = "~/x.json"
[locations.source]
kind = "observed"
read_on = 2026-10-16
quote = "seen on a machine"
"#;

    #[test]
    fn a_minimal_rule_loads_with_no_extension() {
        let rules: Rules<NoExt> = Rules::load([("x.toml", MINIMAL)]).unwrap();
        assert_eq!(rules.hosts.len(), 1);
        assert_eq!(rules.hosts[0].hint("list"), Some("x list"));
        assert_eq!(rules.hosts[0].application(), "X");
        assert_eq!(rules.locations(), 1);
    }

    #[test]
    fn a_documented_location_without_url_is_refused() {
        let text = MINIMAL.replace("kind = \"observed\"", "kind = \"documentation\"");
        let err = Rules::<NoExt>::load([("x.toml", text.as_str())]).unwrap_err();
        assert!(err.to_string().contains("needs a URL"), "{err}");
    }

    #[test]
    fn an_empty_quote_is_refused() {
        let text = MINIMAL.replace("quote = \"seen on a machine\"", "quote = \"  \"");
        let err = Rules::<NoExt>::load([("x.toml", text.as_str())]).unwrap_err();
        assert!(err.to_string().contains("words it was read from"), "{err}");
    }

    #[test]
    fn duplicate_ids_are_refused() {
        let err = Rules::<NoExt>::load([("a.toml", MINIMAL), ("b.toml", MINIMAL)]).unwrap_err();
        assert!(err.to_string().contains("duplicate host id"), "{err}");
    }

    #[test]
    fn out_of_scope_hosts_need_a_reason() {
        let text = r#"
id = "_not-read"
name = "Not read"
kind = "list"
status = "out-of-scope"
[[hosts]]
id = "foo"
name = "Foo"
reason = ""
"#;
        let err = Rules::<NoExt>::load([("n.toml", text)]).unwrap_err();
        assert!(err.to_string().contains("without a reason"), "{err}");
    }

    /// An extension with a required field and its own check.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct KeyExt {
        key: String,
    }

    impl Extension for KeyExt {
        fn validate(loc: &Location<Self>) -> Result<(), String> {
            if loc.ext.key.is_empty() {
                return Err("empty key".into());
            }
            Ok(())
        }
    }

    /// A host-level extension with a required table and its own check.
    #[derive(Debug, Clone, Deserialize, Serialize)]
    struct DefaultsExt {
        defaults: BTreeMap<String, String>,
    }

    impl HostExtension for DefaultsExt {
        fn validate<X: Extension>(rule: &HostRule<X, Self>) -> Result<(), String> {
            if rule.ext.defaults.is_empty() {
                return Err("defaults must name at least one capability".into());
            }
            Ok(())
        }
    }

    #[test]
    fn a_host_extension_is_read_next_to_the_common_fields_and_validated() {
        let text = MINIMAL.replace(
            "hints.list = \"x list\"",
            "hints.list = \"x list\"\n[defaults]\ncommands = \"asks\"",
        );
        let rules: Rules<NoExt, DefaultsExt> = Rules::load([("x.toml", text.as_str())]).unwrap();
        assert_eq!(rules.hosts[0].ext.defaults["commands"], "asks");
        assert_eq!(rules.hosts[0].hint("list"), Some("x list"));
        assert_eq!(rules.hosts[0].locations.len(), 1);

        let text = MINIMAL.replace(
            "hints.list = \"x list\"",
            "hints.list = \"x list\"\n[defaults]",
        );
        let err = Rules::<NoExt, DefaultsExt>::load([("x.toml", text.as_str())]).unwrap_err();
        assert!(err.to_string().contains("at least one capability"), "{err}");

        // Without the table the rule does not parse: the extension is required.
        let err = Rules::<NoExt, DefaultsExt>::load([("x.toml", MINIMAL)]).unwrap_err();
        assert!(matches!(err, RuleError::Parse { .. }), "{err}");
    }

    #[test]
    fn an_extension_is_read_from_the_location_and_validated() {
        let text = MINIMAL.replace(
            "path.any = \"~/x.json\"",
            "path.any = \"~/x.json\"\nkey = \"servers\"",
        );
        let rules: Rules<KeyExt> = Rules::load([("x.toml", text.as_str())]).unwrap();
        assert_eq!(rules.hosts[0].locations[0].ext.key, "servers");

        let text = MINIMAL.replace(
            "path.any = \"~/x.json\"",
            "path.any = \"~/x.json\"\nkey = \"\"",
        );
        let err = Rules::<KeyExt>::load([("x.toml", text.as_str())]).unwrap_err();
        assert!(err.to_string().contains("location 0: empty key"), "{err}");

        let err = Rules::<KeyExt>::load([("x.toml", MINIMAL)]).unwrap_err();
        assert!(matches!(err, RuleError::Parse { .. }), "{err}");
    }
}
