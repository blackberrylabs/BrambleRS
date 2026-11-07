//! Assemble: the six modules with their report status, and one list of findings across
//! modules, most serious first. BrambleRS sorts; it never reinterprets.

use std::time::Duration;

use bramblers_report::{Finding, Report, Severity, Summary};
use serde::Serialize;

use crate::cache::Cache;
use crate::registry::{self, Provider};
use crate::runner::{self, Run, RunError};

/// Where a module's report comes from, or why there is none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum ReportState {
    /// Obtained in this run.
    Fresh { elapsed_ms: u64 },
    /// Read from the saved reports.
    Saved { age_s: u64 },
    /// Never run yet.
    NoReport,
    /// The run or the saved file failed; the reason.
    Failed { error: String },
}

/// One module, as BrambleRS sees it.
#[derive(Debug, Clone, Serialize)]
pub struct ModuleStatus {
    pub provider: Provider,
    pub state: ReportState,
    pub report: Option<Report>,
}

/// One finding in the common list, with the module it comes from.
#[derive(Debug, Clone, Serialize)]
pub struct Line {
    pub module: String,
    pub module_name: String,
    pub finding: Finding,
}

/// The whole picture.
#[derive(Debug, Clone, Serialize)]
pub struct Dashboard {
    pub modules: Vec<ModuleStatus>,
    /// Every finding of every report, most serious first; ties by family order, then title.
    pub lines: Vec<Line>,
}

impl Dashboard {
    /// The six modules and their saved reports — nothing is run.
    pub fn from_saved(cache: &Cache) -> Dashboard {
        let statuses = registry::all()
            .iter()
            .map(|provider| {
                let (state, report) = match cache.read(provider.id) {
                    Some(Ok(saved)) => (
                        ReportState::Saved {
                            age_s: saved.age.as_secs(),
                        },
                        Some(saved.report),
                    ),
                    Some(Err(e)) => (ReportState::Failed { error: e }, None),
                    None => (ReportState::NoReport, None),
                };
                ModuleStatus {
                    provider: *provider,
                    state,
                    report,
                }
            })
            .collect();
        Dashboard::assemble(statuses)
    }

    /// Run one module now, save its report, and update its status. A failure keeps the
    /// previous report if there was one.
    pub fn refresh(&mut self, id: &str, cache: &Cache) -> Result<Run, RunError> {
        let provider = match self
            .modules
            .iter()
            .find(|m| m.provider.id == id)
            .map(|m| m.provider)
        {
            Some(p) => p,
            None => return Err(RunError::Failed(format!("unknown module {id:?}"))),
        };
        let result = runner::report(&provider);
        self.apply(id, &result, cache);
        result
    }

    /// Take a run's outcome into account: save the report, set the state, re-sort.
    pub fn apply(&mut self, id: &str, result: &Result<Run, RunError>, cache: &Cache) {
        if let Some(m) = self.modules.iter_mut().find(|m| m.provider.id == id) {
            match result {
                Ok(run) => {
                    m.state = match cache.write(id, &run.json) {
                        Ok(_) => ReportState::Fresh {
                            elapsed_ms: run.elapsed.as_millis() as u64,
                        },
                        Err(e) => ReportState::Failed { error: e },
                    };
                    m.report = Some(run.report.clone());
                }
                Err(e) => {
                    m.state = ReportState::Failed {
                        error: e.to_string(),
                    }
                }
            }
        }
        let statuses = std::mem::take(&mut self.modules);
        *self = Dashboard::assemble(statuses);
    }

    /// Run every module, in family order.
    pub fn refresh_all(&mut self, cache: &Cache) -> Vec<(String, Result<Duration, RunError>)> {
        let ids: Vec<String> = self
            .modules
            .iter()
            .map(|m| m.provider.id.to_string())
            .collect();
        ids.into_iter()
            .map(|id| {
                let r = self.refresh(&id, cache).map(|run| run.elapsed);
                (id, r)
            })
            .collect()
    }

    fn assemble(modules: Vec<ModuleStatus>) -> Dashboard {
        let mut lines: Vec<(usize, Line)> = Vec::new();
        for (order, m) in modules.iter().enumerate() {
            if let Some(r) = &m.report {
                for f in &r.findings {
                    lines.push((
                        order,
                        Line {
                            module: m.provider.id.to_string(),
                            module_name: m.provider.name.to_string(),
                            finding: f.clone(),
                        },
                    ));
                }
            }
        }
        lines.sort_by(|(oa, a), (ob, b)| {
            a.finding
                .severity
                .cmp(&b.finding.severity)
                .then(oa.cmp(ob))
                .then_with(|| a.finding.title.cmp(&b.finding.title))
        });
        Dashboard {
            modules,
            lines: lines.into_iter().map(|(_, l)| l).collect(),
        }
    }

    /// How many findings at each severity, strongest first.
    pub fn counts(&self) -> [(Severity, usize); 4] {
        Severity::ALL.map(|s| {
            (
                s,
                self.lines
                    .iter()
                    .filter(|l| l.finding.severity == s)
                    .count(),
            )
        })
    }

    /// The figures of every module that has a report, in family order.
    pub fn summaries(&self) -> Vec<(&ModuleStatus, &Summary)> {
        self.modules
            .iter()
            .flat_map(|m| {
                m.report
                    .iter()
                    .flat_map(move |r| r.summary.iter().map(move |s| (m, s)))
            })
            .collect()
    }

    /// The lines at or above a severity (1 = act now only … 4 = everything).
    pub fn at_least(&self, min: Severity) -> impl Iterator<Item = &Line> {
        self.lines.iter().filter(move |l| l.finding.severity <= min)
    }

    pub fn module(&self, id: &str) -> Option<&ModuleStatus> {
        self.modules.iter().find(|m| m.provider.id == id)
    }

    pub fn to_json(&self) -> String {
        let mut s = serde_json::to_string_pretty(self).expect("a dashboard serialises");
        s.push('\n');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_report_is_not_a_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let dash = Dashboard::from_saved(&Cache::at(tmp.path()));
        assert_eq!(dash.modules.len(), 6);
        assert!(dash
            .modules
            .iter()
            .all(|m| m.state == ReportState::NoReport && m.report.is_none()));
        assert!(dash.lines.is_empty());
    }

    #[test]
    fn saved_reports_are_sorted_by_severity() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = Cache::at(tmp.path());
        cache
            .write(
                "permitt",
                bramblers_report::conformance::example().to_json().as_str(),
            )
            .unwrap();
        let dash = Dashboard::from_saved(&cache);
        assert!(!dash.lines.is_empty());
        assert!(dash
            .lines
            .windows(2)
            .all(|w| w[0].finding.severity <= w[1].finding.severity));
        assert!(dash.lines.iter().all(|l| l.module == "permitt"));
    }
}
