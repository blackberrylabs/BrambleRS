//! The saved reports: `~/.local/state/bramblers/<module>/report.json`, written by
//! BrambleRS and by nobody else (rule R2). The modules keep their own promise — nothing
//! written outside their settings — because BrambleRS does the saving.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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
        let home = bramblers_hostrules::env::home_dir().unwrap_or_else(|| PathBuf::from("/"));
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
        write_private(&path, json.as_bytes()).map_err(|e| format!("{}: {e}", path.display()))?;
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

/// Write `data` to a new private file next to `path`, then rename it over `path`. The
/// temporary name is created with `create_new`, so a symlink planted there is not followed;
/// renaming replaces a symlink at the final name instead of writing through it. The final
/// file is 0600: a saved report is nobody else's business.
fn write_private(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(dir)?;
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let tmp = dir.join(format!(".{name}.{}.{nanos}.tmp", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&tmp)?;
    if let Err(e) = file.write_all(data).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    drop(file);
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
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
    #[cfg(unix)]
    fn a_symlink_at_the_report_path_is_replaced_not_followed() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let tmp = tempfile::tempdir().unwrap();
        let c = Cache::at(tmp.path());
        let path = c.path("permitt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let target = tmp.path().join("target");
        std::fs::write(&target, "keep").unwrap();
        symlink(&target, &path).unwrap();
        c.write("permitt", "{\"x\":1}").unwrap();
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "keep");
        assert!(!std::fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "{\"x\":1}");
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
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
