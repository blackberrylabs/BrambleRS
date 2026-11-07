//! How BrambleRS runs MCProster in-process: the same inventory and report as
//! `mcproster report`, including the probe results the module saved. Nothing is launched.

use std::time::Instant;

use bramblers_report::Report;

/// Read the MCP declarations and build the common report.
pub fn report() -> Result<Report, String> {
    let started = Instant::now();
    let rules = crate::hosts::embedded();
    let env = crate::Env::current();
    let inv = crate::inventory::run(&rules, &env);
    let probes = crate::probe::load_results(&env.home);
    Ok(crate::report::build(
        &inv,
        &rules,
        &env,
        &probes,
        started.elapsed().as_millis() as u64,
    ))
}
