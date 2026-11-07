//! A location → the concrete files it designates on this machine. Pure path work: the
//! files are not opened here.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::env::{Env, Os};
use crate::read::read_tree;
use crate::rules::{Extension, Format, Location, OneOrMany, Scope};

/// One file a location designates. `project` is the project root for a per-project file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    pub path: PathBuf,
    pub project: Option<PathBuf>,
}

/// Project roots: the explicit ones of the environment, plus the projects Claude Code knows
/// (`projects.*` in `~/.claude.json`) — absolute paths of directories the user actually
/// works in, which is the best list of project roots a machine offers. Only directories
/// that exist are kept. Read-only.
pub fn project_roots(env: &Env) -> Vec<PathBuf> {
    let mut roots: BTreeSet<PathBuf> = env.roots.iter().cloned().collect();
    let claude = env.home.join(".claude.json");
    if let Ok((_, tree)) = read_tree(&claude, Format::Json) {
        if let Some(projects) = tree.get("projects").and_then(Value::as_object) {
            for k in projects.keys() {
                let p = PathBuf::from(k);
                if p.is_dir() {
                    roots.insert(p);
                }
            }
        }
    }
    roots.into_iter().collect()
}

/// The files a location designates for this environment, globs expanded, in a stable order
/// without duplicates. Per-project locations are looked for under every root; a per-project
/// location whose project is a key *inside* a user file (`project_key`) resolves like a
/// user path. Files are not checked for existence here.
pub fn resolve<X: Extension>(loc: &Location<X>, env: &Env, roots: &[PathBuf]) -> Vec<Resolved> {
    let mut specs: Vec<&str> = Vec::new();
    let os_specs: Option<&OneOrMany> = match env.os {
        Os::Linux => loc.path.linux.as_ref(),
        Os::Macos => loc.path.macos.as_ref(),
        Os::Windows => loc.path.windows.as_ref(),
    };
    if let Some(a) = &loc.path.any {
        specs.extend(a.iter());
    }
    if let Some(o) = os_specs {
        specs.extend(o.iter());
    }
    let mut out: Vec<Resolved> = Vec::new();
    let mut seen = BTreeSet::new();
    let mut push = |path: PathBuf, project: Option<PathBuf>| {
        if seen.insert(path.clone()) {
            out.push(Resolved { path, project });
        }
    };
    for spec in specs {
        let in_file = loc.project_key.is_some();
        match loc.scope {
            Scope::Project if !in_file => {
                for root in roots {
                    for p in expand_glob(&root.join(spec)) {
                        push(p, Some(root.clone()));
                    }
                }
            }
            Scope::User | Scope::System | Scope::Project => {
                if let Some(expanded) = env.expand(spec) {
                    for p in expand_glob(&expanded) {
                        push(p, None);
                    }
                }
            }
        }
    }
    out
}

/// Expand a path containing `*` with `glob`, sorted; a path without `*` is returned as is.
pub fn expand_glob(p: &Path) -> Vec<PathBuf> {
    let s = p.to_string_lossy();
    if !s.contains('*') {
        return vec![p.to_path_buf()];
    }
    match glob::glob(&s) {
        Ok(paths) => {
            let mut v: Vec<PathBuf> = paths.flatten().collect();
            v.sort();
            v
        }
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{NoExt, Rules};

    fn env(home: &Path, roots: Vec<PathBuf>) -> Env {
        Env {
            home: home.to_path_buf(),
            os: Os::Linux,
            vars: [(
                "XDG_CONFIG_HOME".to_string(),
                home.join("cfg").display().to_string(),
            )]
            .into_iter()
            .collect(),
            path: Vec::new(),
            roots,
        }
    }

    fn rules(text: &str) -> Rules<NoExt> {
        Rules::load([("t.toml", text)]).unwrap()
    }

    const HEAD: &str = "id = \"x\"\nname = \"X\"\nkind = \"cli\"\nstatus = \"active\"\n";
    const SRC: &str =
        "[locations.source]\nkind = \"observed\"\nread_on = 2026-10-16\nquote = \"q\"\n";

    #[test]
    fn user_paths_expand_home_and_variables_per_os() {
        let tmp = tempfile::tempdir().unwrap();
        let text = format!(
            "{HEAD}[[locations]]\nscope = \"user\"\nformat = \"json\"\npath.linux = \"${{XDG_CONFIG_HOME}}/x.json\"\npath.macos = \"~/Library/x.json\"\npath.any = \"~/.x.json\"\n{SRC}"
        );
        let r = rules(&text);
        let e = env(tmp.path(), vec![]);
        let files = resolve(&r.hosts[0].locations[0], &e, &[]);
        let paths: Vec<_> = files.iter().map(|f| f.path.clone()).collect();
        assert_eq!(
            paths,
            vec![tmp.path().join(".x.json"), tmp.path().join("cfg/x.json")]
        );
        assert!(files.iter().all(|f| f.project.is_none()));
    }

    #[test]
    fn project_paths_are_looked_for_under_every_root_with_globs() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        std::fs::create_dir_all(a.join(".cfg")).unwrap();
        std::fs::create_dir_all(b.join(".cfg")).unwrap();
        std::fs::write(a.join(".cfg/one.json"), "{}").unwrap();
        std::fs::write(b.join(".cfg/two.json"), "{}").unwrap();
        let text = format!(
            "{HEAD}[[locations]]\nscope = \"project\"\nformat = \"json\"\npath.any = \".cfg/*.json\"\n{SRC}"
        );
        let r = rules(&text);
        let e = env(tmp.path(), vec![]);
        let files = resolve(&r.hosts[0].locations[0], &e, &[a.clone(), b.clone()]);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, a.join(".cfg/one.json"));
        assert_eq!(files[0].project.as_deref(), Some(a.as_path()));
        assert_eq!(files[1].project.as_deref(), Some(b.as_path()));
    }

    #[test]
    fn a_project_key_location_resolves_like_a_user_path() {
        let tmp = tempfile::tempdir().unwrap();
        let text = format!(
            "{HEAD}[[locations]]\nscope = \"project\"\nformat = \"json\"\npath.any = \"~/.claude.json\"\nproject_key = \"projects\"\n{SRC}"
        );
        let r = rules(&text);
        let e = env(tmp.path(), vec![]);
        let files = resolve(&r.hosts[0].locations[0], &e, &[tmp.path().join("p")]);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, tmp.path().join(".claude.json"));
        assert!(files[0].project.is_none());
    }

    #[test]
    fn project_roots_add_the_projects_claude_code_knows_that_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("proj");
        std::fs::create_dir(&p).unwrap();
        std::fs::write(
            tmp.path().join(".claude.json"),
            format!(
                "{{\"projects\": {{\"{}\": {{}}, \"/nonexistent/zzz\": {{}}}}}}",
                p.display()
            ),
        )
        .unwrap();
        let e = env(tmp.path(), vec![tmp.path().join("explicit")]);
        let roots = project_roots(&e);
        assert_eq!(roots, vec![tmp.path().join("explicit"), p]);
    }
}
