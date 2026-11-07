//! The environment an inventory runs against: home directory, variables, operating system,
//! `PATH`, project roots. Built from the real process for a normal run, or from a test
//! fixture — the inventory never touches `std::env` itself, so tests are hermetic.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    Linux,
    Macos,
    Windows,
}

#[derive(Debug, Clone)]
pub struct Env {
    pub home: PathBuf,
    pub os: Os,
    pub vars: BTreeMap<String, String>,
    /// Directories searched for a bare command.
    pub path: Vec<PathBuf>,
    /// Project roots where per-project files are looked for.
    pub roots: Vec<PathBuf>,
}

/// The real home directory. `HOME` when it is set and is not `/`; otherwise the entry of
/// `$USER` or `$LOGNAME` in `/etc/passwd`. `None` when neither gives a home: a caller that
/// scans or writes must refuse to fall back to `/`.
pub fn home_dir() -> Option<PathBuf> {
    if let Some(home) = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
    {
        if !home.as_os_str().is_empty() && home != Path::new("/") {
            return Some(home);
        }
    }
    #[cfg(unix)]
    {
        let user = std::env::var_os("USER").or_else(|| std::env::var_os("LOGNAME"))?;
        let text = std::fs::read_to_string("/etc/passwd").ok()?;
        home_from_passwd(&user.to_string_lossy(), &text)
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// The sixth field of the line for `user` in a passwd file, when it is not `/`.
fn home_from_passwd(user: &str, text: &str) -> Option<PathBuf> {
    text.lines().find_map(|line| {
        let mut fields = line.split(':');
        if fields.next() != Some(user) {
            return None;
        }
        let home = fields.nth(4)?;
        (!home.is_empty() && home != "/").then(|| PathBuf::from(home))
    })
}

impl Env {
    /// The real environment of this process.
    pub fn current() -> Self {
        let mut vars: BTreeMap<String, String> = std::env::vars().collect();
        let home = home_dir().unwrap_or_else(|| PathBuf::from("/"));
        // `$HOME` must expand to the same directory as `~`, even when the variable was
        // missing or pointed at `/` and `/etc/passwd gave the real home.
        vars.insert("HOME".to_string(), home.display().to_string());
        let os = if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::Macos
        } else {
            Os::Linux
        };
        let path = vars
            .get("PATH")
            .map(|p| std::env::split_paths(p).collect())
            .unwrap_or_default();
        Env {
            home,
            os,
            vars,
            path,
            roots: Vec::new(),
        }
    }

    /// A hermetic environment rooted at `home` (tests, `--home`).
    pub fn fixture(home: &Path, os: Os) -> Self {
        let mut vars = BTreeMap::new();
        vars.insert("HOME".to_string(), home.display().to_string());
        Env {
            home: home.to_path_buf(),
            os,
            vars,
            path: vec![home.join("bin")],
            roots: Vec::new(),
        }
    }

    pub fn with_roots(mut self, roots: Vec<PathBuf>) -> Self {
        self.roots = roots;
        self
    }

    /// Expand `~`, `$VAR`, `${VAR}` and `%VAR%` in a rule path. `None` when a variable is
    /// unset: the candidate simply does not apply on this machine.
    pub fn expand(&self, spec: &str) -> Option<PathBuf> {
        let mut s = spec.to_string();
        if let Some(rest) = s.strip_prefix("~/") {
            s = format!("{}/{}", self.home.display(), rest);
        } else if s == "~" {
            s = self.home.display().to_string();
        }
        // ${VAR}
        while let Some(start) = s.find("${") {
            let end = s[start..].find('}')? + start;
            let name = &s[start + 2..end];
            let value = self.vars.get(name)?.clone();
            s.replace_range(start..=end, &value);
        }
        // %VAR%
        while let Some(start) = s.find('%') {
            let end = s[start + 1..].find('%')? + start + 1;
            let name = &s[start + 1..end];
            let value = self.vars.get(name)?.clone();
            s.replace_range(start..=end, &value);
        }
        // $VAR
        while let Some(start) = s.find('$') {
            let tail = &s[start + 1..];
            let len = tail
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(tail.len());
            if len == 0 {
                return None;
            }
            let name = &tail[..len];
            let value = self.vars.get(name)?.clone();
            s.replace_range(start..start + 1 + len, &value);
        }
        Some(PathBuf::from(s))
    }

    /// Find a bare command on `PATH`, or check an explicit path. `None` when not found.
    pub fn which(&self, command: &str) -> Option<PathBuf> {
        let p = Path::new(command);
        if p.components().count() > 1 || p.is_absolute() {
            let full = if p.is_absolute() {
                p.to_path_buf()
            } else {
                self.home.join(p)
            };
            return full.is_file().then_some(full);
        }
        self.path
            .iter()
            .map(|d| d.join(command))
            .find(|c| c.is_file())
    }

    pub fn home_str(&self) -> String {
        self.home.display().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwd_entry_gives_a_home_but_root_is_refused() {
        let text = "root:x:0:0:root:/root:/bin/bash\n\
                    alex:x:1000:1000:Alex:/home/alex:/bin/bash\n\
                    nohome:x:1:1::/:/bin/false\n";
        assert_eq!(
            home_from_passwd("alex", text),
            Some(PathBuf::from("/home/alex"))
        );
        assert_eq!(home_from_passwd("root", text), Some(PathBuf::from("/root")));
        assert_eq!(home_from_passwd("nohome", text), None);
        assert_eq!(home_from_passwd("missing", text), None);
    }

    #[test]
    fn expands_home_and_variables() {
        let mut e = Env::fixture(Path::new("/h"), Os::Linux);
        e.vars.insert("APPDATA".into(), "/h/app".into());
        assert_eq!(e.expand("~/.x.json").unwrap(), PathBuf::from("/h/.x.json"));
        assert_eq!(
            e.expand("%APPDATA%\\Claude\\c.json").unwrap(),
            PathBuf::from("/h/app\\Claude\\c.json")
        );
        assert_eq!(e.expand("$HOME/.y").unwrap(), PathBuf::from("/h/.y"));
        assert_eq!(e.expand("${HOME}/.z").unwrap(), PathBuf::from("/h/.z"));
        assert!(e.expand("$COPILOT_HOME/mcp-config.json").is_none());
    }
}
