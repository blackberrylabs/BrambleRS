//! How BrambleRS runs Cleanator in-process: the same scan and classification as
//! `cleanator-cli report --no-probes`, rooted at the home directory. Nothing is launched.

use std::time::Instant;

use bramblers_report::Report;

use crate::classify::{classify, ClassifyOptions};
use crate::paths::Env;
use crate::rules::RuleSet;
use crate::scan::{scan, ScanOptions};

/// Scan the home directory, apply the embedded rules, and build the common report.
pub fn report() -> Result<Report, String> {
    let started = Instant::now();
    let env = Env::current();
    let rules = RuleSet::embedded().map_err(|e| e.to_string())?;
    let scan = scan(
        &env.home,
        ScanOptions {
            one_filesystem: true,
            ..Default::default()
        },
    )
    .map_err(|e| e.to_string())?;
    // The tower's sheet ran the command line with `--no-probes`: the same defaults here.
    let opts = ClassifyOptions {
        dormancy_days: 60,
        run_probes: false,
        ..Default::default()
    };
    let classified = classify(&scan, &rules, &env, &opts);
    Ok(crate::report::build(
        &classified,
        started.elapsed().as_millis() as u64,
    ))
}
