//! How BrambleRS runs Lastrun in-process: the same scan and report as `lastrun report`,
//! with the default 30-day window. Nothing is launched.

use std::time::Instant;

use bramblers_hostrules::Env;
use bramblers_report::Report;

/// Read the hosts' transcripts and build the common report.
pub fn report() -> Result<Report, String> {
    let started = Instant::now();
    let env = Env::current();
    let inv = crate::scan(&env);
    Ok(crate::report::build(
        &inv,
        started.elapsed().as_millis() as u64,
    ))
}
