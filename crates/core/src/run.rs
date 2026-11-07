//! Launch one module's command line for its report, with a time limit. The only thing the
//! tower ever executes is `<module cli> <report args>` — a read-only command by contract.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use bramblers_report::Report;

use crate::spec::Spec;

/// Why a report could not be obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// The binary could not be started.
    Spawn(String),
    /// The module ran longer than its sheet allows and was stopped.
    Timeout(u64),
    /// The module exited with an error; the first line of its stderr.
    Exit(i32, String),
    /// The output is not a conforming report; every defect listed.
    NotAReport(Vec<String>),
}

impl std::fmt::Display for RunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunError::Spawn(e) => write!(f, "could not start: {e}"),
            RunError::Timeout(s) => write!(f, "no report after {s} s, stopped"),
            RunError::Exit(code, err) if err.is_empty() => write!(f, "exited with {code}"),
            RunError::Exit(code, err) => write!(f, "exited with {code}: {err}"),
            RunError::NotAReport(errs) => {
                write!(f, "not a bbl-report document: {}", errs.join("; "))
            }
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

/// Run `cli <spec.report>` and check its stdout as a report.
pub fn report(spec: &Spec, cli: &Path) -> Result<Run, RunError> {
    let started = Instant::now();
    let mut child = Command::new(cli)
        .args(&spec.report)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| RunError::Spawn(e.to_string()))?;

    // Drain stdout and stderr on threads so a verbose module cannot block on a full pipe.
    let mut out = child.stdout.take().expect("piped");
    let mut err = child.stderr.take().expect("piped");
    let out_t = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let err_t = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err.read_to_string(&mut s);
        s
    });

    let limit = Duration::from_secs(spec.timeout_s);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() > limit => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(RunError::Timeout(spec.timeout_s));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(RunError::Spawn(e.to_string())),
        }
    };
    let stdout = out_t.join().unwrap_or_default();
    let stderr = err_t.join().unwrap_or_default();
    if !status.success() {
        // The first meaningful line of stderr is usually the reason (`unrecognized
        // subcommand 'report'` for a module older than the contract, say).
        let first = stderr
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("")
            .to_string();
        return Err(RunError::Exit(status.code().unwrap_or(-1), first));
    }
    let report = bramblers_report::conformance::check(&stdout).map_err(RunError::NotAReport)?;
    Ok(Run {
        report,
        json: stdout,
        elapsed: started.elapsed(),
    })
}
