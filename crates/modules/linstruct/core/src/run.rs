//! How BrambleRS runs Linstruct in-process: the same roots and scan as `linstruct report`.
//! Nothing is launched.

use std::time::Instant;

use bramblers_hostrules::Env;
use bramblers_report::Report;

/// Scan the instruction files and build the common report.
pub fn report() -> Result<Report, String> {
    let started = Instant::now();
    let env = Env::current();
    let sheets = crate::rules::embedded();
    // The command line's rule: remembered folders and known projects, then the working
    // directory when there is none.
    let mut roots = env.roots.clone();
    if roots.is_empty() {
        roots = crate::projects::default_roots(&env, &sheets);
        if roots.is_empty() {
            if let Ok(cwd) = std::env::current_dir() {
                roots.push(cwd);
            }
        }
    }
    let env = env.with_roots(roots);
    let inv = crate::scan_with(&env, &sheets);
    Ok(crate::report::build(
        &inv,
        started.elapsed().as_millis() as u64,
    ))
}
