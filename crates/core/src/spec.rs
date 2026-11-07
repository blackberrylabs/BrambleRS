//! Module sheets: how the tower finds, launches and presents one module. Declarative TOML in
//! `modules/*.toml`, embedded in the binary. Adding a module to the family = one sheet and a
//! binary that answers `report` with a `bbl-report/1` document.

use serde::{Deserialize, Serialize};

/// One module sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spec {
    /// Stable slug, the same as `module.id` in the module's reports.
    pub id: String,
    pub name: String,
    /// What the module does, one word: `clean`, `inventory`, `inspect`.
    pub verb: String,
    pub tagline: String,
    /// A short paragraph for the module card when the module is not installed.
    #[serde(default)]
    pub description: String,
    /// Binary of the command line, looked up on `PATH`.
    pub cli: String,
    /// Binary of the desktop app, looked up on `PATH`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// Arguments that make the command line write its report on stdout.
    pub report: Vec<String>,
    /// How long the tower waits for the report before giving up.
    #[serde(default = "default_timeout")]
    pub timeout_s: u64,
    /// The report schema the module speaks.
    #[serde(default = "default_schema")]
    pub schema: String,
    /// Home page; shown, never followed.
    pub home: String,
    /// Where to take the module from when it is not installed; shown, never followed.
    #[serde(default)]
    pub releases: String,
    /// The latest version the tower knows a release for, `x.y.z`: with the family's package
    /// naming it gives the exact file of each release asset (see `packages`). Updated in the
    /// tower when a module releases; until then the tower offers the previous version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest: Option<String>,
}

impl Spec {
    /// The release asset of `kind` for the latest known version, by the family's naming:
    /// `<id>_<v>-1_amd64.deb`, `<id>-<v>-1.x86_64.rpm`, `<Name>-<v>-x86_64.AppImage`, under
    /// `<releases>/download/v<v>/`. None without a known version or a releases page.
    pub fn package_url(&self, kind: crate::packages::Kind) -> Option<String> {
        let v = self.latest.as_deref()?;
        if self.releases.is_empty() {
            return None;
        }
        let file = match kind {
            crate::packages::Kind::Deb => format!("{}_{v}-1_amd64.deb", self.id),
            crate::packages::Kind::Rpm => format!("{}-{v}-1.x86_64.rpm", self.id),
            crate::packages::Kind::AppImage => format!("{}-{v}-x86_64.AppImage", self.name),
        };
        Some(format!(
            "{}/download/v{v}/{file}",
            self.releases.trim_end_matches('/')
        ))
    }
}

fn default_timeout() -> u64 {
    120
}

fn default_schema() -> String {
    bramblers_report::SCHEMA.to_string()
}

/// The sheets, in family order.
#[derive(Debug, Clone)]
pub struct Modules {
    pub specs: Vec<Spec>,
}

/// The sheets shipped with this version of the tower.
pub const EMBEDDED: &[(&str, &str)] = &[
    (
        "billfold.toml",
        include_str!("../../../modules/billfold.toml"),
    ),
    (
        "cleanator.toml",
        include_str!("../../../modules/cleanator.toml"),
    ),
    (
        "lastrun.toml",
        include_str!("../../../modules/lastrun.toml"),
    ),
    (
        "linstruct.toml",
        include_str!("../../../modules/linstruct.toml"),
    ),
    (
        "mcproster.toml",
        include_str!("../../../modules/mcproster.toml"),
    ),
    (
        "permitt.toml",
        include_str!("../../../modules/permitt.toml"),
    ),
];

impl Modules {
    /// The embedded sheets, validated.
    pub fn embedded() -> Modules {
        Modules::load(EMBEDDED.iter().map(|(n, t)| (*n, *t)))
            .expect("the embedded sheets are valid")
    }

    /// Parse and validate sheets from `(file name, text)` pairs.
    pub fn load<'a>(
        files: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Result<Modules, String> {
        let mut specs = Vec::new();
        for (name, text) in files {
            let spec: Spec = toml::from_str(text).map_err(|e| format!("{name}: {e}"))?;
            spec.validate().map_err(|e| format!("{name}: {e}"))?;
            if specs.iter().any(|s: &Spec| s.id == spec.id) {
                return Err(format!("{name}: id {:?} appears twice", spec.id));
            }
            specs.push(spec);
        }
        Ok(Modules { specs })
    }

    pub fn get(&self, id: &str) -> Option<&Spec> {
        self.specs.iter().find(|s| s.id == id)
    }
}

impl Spec {
    fn validate(&self) -> Result<(), String> {
        let slug = !self.id.is_empty()
            && self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !slug {
            return Err(format!("id {:?} is not a slug", self.id));
        }
        for (field, value) in [
            ("name", &self.name),
            ("verb", &self.verb),
            ("tagline", &self.tagline),
            ("cli", &self.cli),
            ("home", &self.home),
        ] {
            if value.trim().is_empty() {
                return Err(format!("{field}: empty"));
            }
        }
        if self.report.is_empty() {
            return Err("report: no arguments".into());
        }
        if self.timeout_s == 0 {
            return Err("timeout_s: 0".into());
        }
        if !self.schema.starts_with("bbl-report/") {
            return Err(format!(
                "schema {:?} is not a bbl-report schema",
                self.schema
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_sheets_are_valid_and_in_family_order() {
        let m = Modules::embedded();
        let ids: Vec<&str> = m.specs.iter().map(|s| s.id.as_str()).collect();
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
        assert_eq!(m.get("cleanator").unwrap().cli, "cleanator-cli");
        assert_eq!(
            m.get("permitt").unwrap().app.as_deref(),
            Some("permitt-app")
        );
    }

    #[test]
    fn a_bad_sheet_is_refused_with_its_name() {
        let err = Modules::load([("x.toml", "id = \"X\"\nname=\"x\"\nverb=\"v\"\ntagline=\"t\"\ncli=\"c\"\nreport=[\"report\"]\nhome=\"h\"")]).unwrap_err();
        assert!(err.starts_with("x.toml:"), "{err}");
    }
}
