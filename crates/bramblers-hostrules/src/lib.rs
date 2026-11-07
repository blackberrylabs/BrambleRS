//! `bbl-hostrules`: where each AI assistant keeps its files on a machine, and how to read them
//! without writing — with the source of every claim.
//!
//! A *host rule* is a TOML file: the host's identity, the locations of its files per
//! operating system and scope (user / project / system), the file format, and — for every
//! location — the **source** it was established from (documentation, source code, or
//! observed on a real machine; URL, date, quote). A rule whose location has no source is
//! refused at load time.
//!
//! The crate is shared by the Blackberrylabs tools (MCProster, Permitt). Each tool brings
//! its own rule files and its own extensions: an [`rules::Extension`] for the fields a
//! location needs (which key holds the MCP servers) and a [`rules::HostExtension`] for what
//! the tool knows about the host as a whole (its defaults, the meaning of its permission
//! settings). This crate owns what they have in common:
//!
//! - [`rules`]: the rule model, loading and validation;
//! - [`mod@env`]: the environment a run is made against (home, OS, variables, `PATH`, project
//!   roots), with path expansion (`~`, `${VAR}`, `%VAR%`, `$VAR`) and `which`;
//! - [`resolve`]: a location → the concrete files it designates on this machine;
//! - [`read`]: JSON / JSONC / TOML / YAML → `serde_json::Value`, with key paths
//!   (`a.b.*.c`, `\.` for a literal dot) and line numbers;
//! - [`secrets`]: classify and mask a value the moment it is read.
//!
//! Two promises, enforced by the API surface: there is no function that writes a file,
//! and there is no network code or dependency.
#![forbid(unsafe_code)]

pub mod env;
pub mod read;
pub mod resolve;
pub mod rules;
pub mod secrets;

pub use env::{Env, Os};
pub use rules::{
    Extension, Format, HostExtension, HostRule, Location, NoExt, Rules, Scope, Source, SourceKind,
};
