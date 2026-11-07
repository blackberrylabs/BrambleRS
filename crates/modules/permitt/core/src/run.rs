//! How BrambleRS runs Permitt in-process: the same inventory and report as
//! `permitt report`, with the command line's defaults. Nothing is launched.

use std::time::Instant;

use bramblers_report::Report;

/// Read the hosts' permission settings and build the common report.
pub fn report() -> Result<Report, String> {
    let started = Instant::now();
    let rules = crate::rules::embedded();
    let shell = crate::rules::embedded_shell();
    let env = crate::Env::current();
    let inv = crate::inventory::run(
        &rules,
        &shell,
        &env,
        crate::Options {
            no_shell: false,
            all_hosts: false,
        },
    );
    Ok(crate::report::build(
        &inv,
        &rules,
        &env,
        started.elapsed().as_millis() as u64,
    ))
}
