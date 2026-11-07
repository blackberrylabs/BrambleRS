//! Run one module in-process: call its `run` function on a thread, wait for the report,
//! and stop waiting at its time limit. Nothing is launched, no PATH is read.

use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant};

use bramblers_report::Report;

use crate::registry::Provider;

/// Why a report could not be obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// The module ran longer than its record allows and BrambleRS stopped waiting. The
    /// thread cannot be killed safely; it is detached and its result is ignored.
    Timeout(u64),
    /// The module itself refused: it says why.
    Failed(String),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::Timeout(s) => write!(f, "no report after {s} s, stopped waiting"),
            RunError::Failed(e) => write!(f, "{e}"),
        }
    }
}

/// A report obtained from a module, and how long it took.
#[derive(Debug, Clone)]
pub struct Run {
    pub report: Report,
    pub json: String,
    pub elapsed: Duration,
}

/// Call the module's `run` function and wait at most `provider.timeout_s` for it.
pub fn report(provider: &Provider) -> Result<Run, RunError> {
    let started = Instant::now();
    let run = provider.run;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(run());
    });
    match rx.recv_timeout(Duration::from_secs(provider.timeout_s)) {
        Ok(Ok(report)) => {
            let json = report.to_json();
            Ok(Run {
                report,
                json,
                elapsed: started.elapsed(),
            })
        }
        Ok(Err(e)) => Err(RunError::Failed(e)),
        Err(RecvTimeoutError::Timeout) => Err(RunError::Timeout(provider.timeout_s)),
        Err(RecvTimeoutError::Disconnected) => Err(RunError::Failed(
            "the run thread stopped without a report".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok() -> Result<Report, String> {
        Ok(bramblers_report::conformance::example())
    }

    fn refused() -> Result<Report, String> {
        Err("the host rules could not be read".into())
    }

    fn slow() -> Result<Report, String> {
        std::thread::sleep(Duration::from_secs(3));
        Ok(bramblers_report::conformance::example())
    }

    fn provider(run: fn() -> Result<Report, String>, timeout_s: u64) -> Provider {
        Provider {
            id: "test",
            name: "Test",
            verb: "test",
            tagline: "",
            description: "",
            timeout_s,
            run,
        }
    }

    #[test]
    fn a_report_comes_back_with_its_json() {
        let run = report(&provider(ok, 5)).unwrap();
        assert_eq!(run.report.module.id, "permitt");
        assert!(run.json.contains("\"schema\""));
    }

    #[test]
    fn a_refusal_says_why() {
        assert!(matches!(
            report(&provider(refused, 5)),
            Err(RunError::Failed(ref e)) if e == "the host rules could not be read"
        ));
    }

    #[test]
    fn a_slow_module_times_out() {
        assert!(matches!(
            report(&provider(slow, 1)),
            Err(RunError::Timeout(1))
        ));
    }
}
