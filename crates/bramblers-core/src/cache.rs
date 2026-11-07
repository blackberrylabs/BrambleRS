//! The saved reports: `~/.local/state/bramblers/<module>/report.json`, written by
//! BrambleRS and by nobody else (rule R2). The modules keep their own promise — nothing
//! written outside their settings — because BrambleRS does the saving.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use bramblers_report::Report;

/// Where the saved reports live.
#[derive(Debug, Clone)]
pub struct Cache {
    pub dir: PathBuf,
}

/// A saved report and its age.
#[derive(Debug, Clone)]
pub struct Saved {
    pub report: Report,
    pub json: String,
    /// Time since the file was written.
    pub age: Duration,
    pub path: PathBuf,
}

impl Cache {
    /// `$BRAMBLERS_STATE_DIR`, else `$XDG_STATE_HOME/bramblers`, else
    /// `~/.local/state/bramblers`. `$BBL_STATE_DIR` is still read for the old scripts.
    pub fn default_location() -> Cache {
        let dir =
            std::env::var_os("BRAMBLERS_STATE_DIR").or_else(|| std::env::var_os("BBL_STATE_DIR"));
        if let Some(d) = dir {
            return Cache {
                dir: PathBuf::from(d),
            };
        }
        if let Some(d) = std::env::var_os("XDG_STATE_HOME").filter(|d| !d.is_empty()) {
            return Cache {
                dir: PathBuf::from(d).join("bramblers"),
            };
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/"));
        Cache {
            dir: home.join(".local/state/bramblers"),
        }
    }

    pub fn at(dir: impl Into<PathBuf>) -> Cache {
        Cache { dir: dir.into() }
    }

    /// `<dir>/<module>/report.json`.
    pub fn path(&self, module: &str) -> PathBuf {
        self.dir.join(module).join("report.json")
    }

    /// The saved report of a module, if any and readable. A document of another schema or
    /// a broken file is reported as an error, not ignored, so the person can see why.
    pub fn read(&self, module: &str) -> Option<Result<Saved, String>> {
        let path = self.path(module);
        let json = match std::fs::read_to_string(&path) {
            Ok(j) => j,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => return Some(Err(format!("{}: {e}", path.display()))),
        };
        let age = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| SystemTime::now().duration_since(t).ok())
            .unwrap_or_default();
        // The old tower wrote `bbl-report/1`; those reports predate this schema and carry
        // nothing BrambleRS can use. They are ignored, not shown as a permanent failure.
        if bramblers_report::peek_schema(&json).is_some_and(|s| s.starts_with("bbl-report/")) {
            return None;
        }
        match bramblers_report::conformance::check(&json) {
            Ok(report) => Some(Ok(Saved {
                report,
                json,
                age,
                path,
            })),
            Err(errs) => Some(Err(format!("{}: {}", path.display(), errs.join("; ")))),
        }
    }

    /// Save a module's report: the module directory is created, the file is written next
    /// to its final name and renamed into place, so a reader never sees a half file.
    pub fn write(&self, module: &str, json: &str) -> Result<PathBuf, String> {
        let path = self.path(module);
        let dir = path.parent().expect("a module directory");
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let tmp = dir.join("report.json.tmp");
        std::fs::write(&tmp, json).map_err(|e| format!("{}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(path)
    }

    /// Every module id that has a saved report.
    pub fn modules(&self) -> Vec<String> {
        let Ok(rd) = std::fs::read_dir(&self.dir) else {
            return Vec::new();
        };
        let mut out: Vec<String> = rd
            .flatten()
            .filter(|e| e.path().join("report.json").is_file())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        out.sort();
        out
    }
}

/// `3 min`, `2 h`, `5 d`: an age in words, for the module cards.
pub fn age_label(age: Duration) -> String {
    let s = age.as_secs();
    if s < 60 {
        "just now".to_string()
    } else if s < 3600 {
        format!("{} min ago", s / 60)
    } else if s < 86_400 {
        format!("{} h ago", s / 3600)
    } else {
        format!("{} d ago", s / 86_400)
    }
}

/// True when `p` is inside `dir`.
pub fn is_under(p: &Path, dir: &Path) -> bool {
    p.starts_with(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ages_in_words() {
        assert_eq!(age_label(Duration::from_secs(5)), "just now");
        assert_eq!(age_label(Duration::from_secs(185)), "3 min ago");
        assert_eq!(age_label(Duration::from_secs(7300)), "2 h ago");
        assert_eq!(age_label(Duration::from_secs(200_000)), "2 d ago");
    }

    #[test]
    fn location_follows_the_environment() {
        let c = Cache::at("/tmp/x");
        assert_eq!(
            c.path("permitt"),
            PathBuf::from("/tmp/x/permitt/report.json")
        );
    }

    #[test]
    fn a_report_of_the_old_schema_is_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let c = Cache::at(tmp.path());
        let path = c.path("permitt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            r#"{ "schema": "bbl-report/1", "module": { "id": "permitt" } }"#,
        )
        .unwrap();
        assert!(c.read("permitt").is_none());
    }
}
