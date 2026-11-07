//! The BrambleRS control tower, core.
//!
//! One screen that says what is wrong on a machine on the AI side — most serious first —
//! and opens the BrambleRS module that deals with it. The tower computes nothing: it
//! launches the installed modules, reads their `bbl-report/1` documents, assembles them into
//! one list of findings sorted by severity, and knows how to open a module at a finding.
//!
//! Rules (BRIEF, R1–R6): the tower reads no host file and scans nothing; it writes nothing
//! outside `~/.config/blackberrylabs/` (interface preferences, the app's) and
//! `~/.local/state/blackberrylabs/<module>/report.json` (the saved reports, written here);
//! no network; actions are shown, never run; a module plugs in through the contract and its
//! declarative sheet only.

#![forbid(unsafe_code)]

pub mod cache;
pub mod dashboard;
pub mod discover;
pub mod packages;
pub mod run;
pub mod spec;

pub use cache::Cache;
pub use dashboard::{Dashboard, Line, ModuleStatus, ReportState};
pub use spec::{Modules, Spec};
