//! The six modules of BrambleRS, statically: one record per module, in family order.
//! No discovery, no PATH lookup, no subprocess. A module is a crate linked into the
//! binary; its `run` function is called in-process on a thread.

use std::fmt;

use bramblers_report::Report;
use serde::Serialize;

/// One module, as BrambleRS knows it.
#[derive(Clone, Copy, Serialize)]
pub struct Provider {
    /// Stable slug, the same as `module.id` in the module's reports.
    pub id: &'static str,
    pub name: &'static str,
    /// What the module does, one word: `clean`, `inventory`, `inspect`.
    pub verb: &'static str,
    pub tagline: &'static str,
    /// A short paragraph for the module card and its detail.
    pub description: &'static str,
    /// How long BrambleRS waits for the module's report before giving up.
    pub timeout_s: u64,
    /// Produce the module's report. The module crates own this function; the registry
    /// only points at it.
    #[serde(skip)]
    pub run: fn() -> Result<Report, String>,
}

impl fmt::Debug for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Provider")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// The six modules, in family order. Adding a module means adding a crate, one record
/// here, and nothing else.
pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "billfold",
        name: "Billfold",
        verb: "tally",
        tagline: "What this machine spends on AI: accounts, keys, tokens",
        description: "The accounts each AI host is logged into and how each pays, the provider keys the machine holds and who can read them, the tokens each host burned in the last 30 days — priced at API rates from an embedded snapshot, never fetched. Twenty-one hosts, twenty-two providers, secrets masked.",
        timeout_s: 120,
        run: bramblers_billfold_core::run::report,
    },
    Provider {
        id: "cleanator",
        name: "Cleanator",
        verb: "clean",
        tagline: "Disk cleaner for developers who run AI on Linux — never deletes anything",
        description: "Scans a home directory, recognises what AI tooling leaves behind (model stores, caches, build outputs, abandoned environments) and proposes what could go — as a script you read and run yourself.",
        timeout_s: 900,
        run: bramblers_cleanator_core::run::report,
    },
    Provider {
        id: "lastrun",
        name: "Lastrun",
        verb: "replay",
        tagline: "What your AI agents actually did on this machine — commands, files, packages — session by session",
        description: "For each AI host that records its actions, the sessions of the last 30 days and what they did — commands run, files created, edited and deleted, packages installed, MCP calls, refusals — read back from the transcripts the hosts already write, never their text. Twenty-one hosts, every rule sourced.",
        // 128 s in a debug build on 201 MB of transcripts; the release build reads the same
        // machine in 8 s. 300 s leaves room for a slow disk without hiding a real hang.
        timeout_s: 300,
        run: bramblers_lastrun_core::run::report,
    },
    Provider {
        id: "linstruct",
        name: "Linstruct",
        verb: "lint",
        tagline: "Your AI instruction files, as each agent will actually read them",
        description: "For each AI host on the machine and each project, the stack of instruction files the host would load (CLAUDE.md, AGENTS.md, .cursor/rules…) — in its order, references expanded, budget simulated — and what it would cut, ignore or shadow. Nineteen hosts, every rule sourced.",
        timeout_s: 120,
        run: bramblers_linstruct_core::run::report,
    },
    Provider {
        id: "mcproster",
        name: "MCProster",
        verb: "inventory",
        tagline: "Every MCP server declared on your machine, in one list",
        description: "Reads the MCP configuration of every AI host on the machine and lists the servers once, with the hosts and files that declare them, duplicates, secrets in clear, missing commands and skipped confirmations.",
        timeout_s: 60,
        run: bramblers_mcproster_core::run::report,
    },
    Provider {
        id: "permitt",
        name: "Permitt",
        verb: "inspect",
        tagline: "What your AI agents may do on this machine without asking",
        description: "One row per AI assistant, one column per capability (commands, files, network, MCP): what runs without asking, which setting says so, and the bypass flags hidden in shell start-up files.",
        timeout_s: 60,
        run: bramblers_permitt_core::run::report,
    },
];

/// Every module, in family order.
pub fn all() -> &'static [Provider] {
    PROVIDERS
}

/// One module by its stable slug.
pub fn get(id: &str) -> Option<&'static Provider> {
    PROVIDERS.iter().find(|p| p.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn six_modules_in_family_order_with_unique_ids() {
        let ids: Vec<&str> = all().iter().map(|p| p.id).collect();
        assert_eq!(
            ids,
            [
                "billfold",
                "cleanator",
                "lastrun",
                "linstruct",
                "mcproster",
                "permitt"
            ]
        );
        let mut seen = std::collections::BTreeSet::new();
        for p in all() {
            assert!(seen.insert(p.id), "{} appears twice", p.id);
            assert!(p.timeout_s > 0, "{}: no timeout", p.id);
        }
    }
}
