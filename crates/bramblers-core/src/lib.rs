//! BrambleRS core.
//!
//! One screen that says what is wrong on a machine on the AI side — most serious first.
//! The six modules are crates linked into this binary: the core calls each module's `run`
//! function in-process, reads the report it returns, assembles one list of findings sorted
//! by severity, and saves the reports it may reread.
//!
//! Rules (BRIEF, R1–R6): the core has no code specific to a module beyond the registry
//! record; it launches nothing and reads no PATH; it writes nothing outside
//! `~/.config/bramblers/` (interface preferences, the app's) and
//! `~/.local/state/bramblers/<module>/report.json` (the saved reports, written here);
//! no network; actions are shown, never run.

#![forbid(unsafe_code)]

pub mod cache;
pub mod dashboard;
pub mod registry;
pub mod runner;

pub use cache::Cache;
pub use dashboard::{Dashboard, Line, ModuleStatus, ReportState};
pub use registry::{Provider, PROVIDERS};
pub use runner::{Run, RunError};
