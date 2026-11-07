//! The probe: start a stdio server exactly as its host would, ask `initialize` then
//! `tools/list`, and stop it. On demand only; nothing else is called; a deadline bounds
//! the whole exchange. Remote servers (HTTP, SSE) are not probed: no network (BRIEF R2).
//!
//! The clear environment values the server needs are passed to the child process and
//! are not kept: a result holds the server's name and version, its tool names and sizes,
//! and a masked error. It never holds the command line or the server's stderr, which can
//! both contain secrets after reference expansion.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use bramblers_hostrules::secrets;

pub const PROTOCOL_VERSION: &str = "2025-06-18";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// At most this many bytes of stdout are read from a server. A malicious or broken server
/// can otherwise send a line of several gigabytes and take the application down.
const MAX_STDOUT: u64 = 32 * 1024 * 1024;

/// The variables a server may need to start, copied from the parent. Everything else is
/// dropped: a server declared in a project file must not see the desktop's secrets.
const KEEP_ENV: &[&str] = &[
    "PATH",
    "HOME",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "USER",
    "LOGNAME",
    "XDG_RUNTIME_DIR",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    /// First line of the description, shown live; never written to disk (BRIEF D9).
    #[serde(skip)]
    pub description: Option<String>,
    /// Estimated tokens of this tool's definition as sent to the model (name, description,
    /// input schema…): characters of its JSON / 4, rounded up.
    #[serde(default)]
    pub tokens: u64,
}

/// Rough tokens for a JSON text: four characters per token, rounded up. Hosts serialise
/// tool definitions their own way, so this is an estimate (±20 %), always shown with "~".
pub fn estimate_tokens(json_chars: usize) -> u64 {
    json_chars.div_ceil(4) as u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    pub at: String,
    pub ok: bool,
    pub error: Option<String>,
    pub server_name: Option<String>,
    pub server_version: Option<String>,
    pub protocol_version: Option<String>,
    pub tools: Vec<ToolInfo>,
    /// Size of all tool definitions (JSON) returned by `tools/list`.
    #[serde(default)]
    pub schema_bytes: u64,
    /// Sum of the tools' estimated tokens.
    #[serde(default)]
    pub estimated_tokens: u64,
    pub duration_ms: u128,
}

/// Expand the references a host would expand before launching: `${VAR}`, `${VAR:-default}`,
/// `${env:VAR}`, `{env:VAR}`. Unknown variables are left as written.
pub fn expand_refs(value: &str, vars: &BTreeMap<String, String>) -> String {
    let mut s = value.to_string();
    for (open, close) in [("${env:", "}"), ("{env:", "}"), ("${", "}")] {
        let mut from = 0;
        while let Some(start) = s[from..].find(open).map(|i| i + from) {
            let Some(end) = s[start..].find(close).map(|i| i + start) else {
                break;
            };
            let inner = &s[start + open.len()..end];
            let (name, default) = match inner.split_once(":-") {
                Some((n, d)) => (n, Some(d)),
                None => (inner, None),
            };
            match vars.get(name).map(String::as_str).or(default) {
                Some(v) => {
                    let v = v.to_string();
                    s.replace_range(start..end + close.len(), &v);
                    from = start + v.len();
                }
                None => from = end + close.len(),
            }
        }
    }
    s
}

/// Probe a stdio server. `vars` are the variables used to expand references (normally the
/// process environment); `env` is the server's own environment from its declaration.
pub fn probe_stdio(
    command: &str,
    args: &[String],
    env: &BTreeMap<String, String>,
    cwd: Option<&Path>,
    vars: &BTreeMap<String, String>,
    timeout: Duration,
) -> ProbeResult {
    let started = Instant::now();
    let at = chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let command_raw = command.to_string();
    let command = expand_refs(command, vars);
    let args: Vec<String> = args.iter().map(|a| expand_refs(a, vars)).collect();
    let mut result = ProbeResult {
        at,
        ok: false,
        error: None,
        server_name: None,
        server_version: None,
        protocol_version: None,
        tools: Vec::new(),
        schema_bytes: 0,
        estimated_tokens: 0,
        duration_ms: 0,
    };

    let mut cmd = Command::new(&command);
    cmd.args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    cmd.env_clear();
    for key in KEEP_ENV {
        if let Some(value) = vars.get(*key) {
            cmd.env(key, value);
        }
    }
    for (k, v) in env {
        cmd.env(k, expand_refs(v, vars));
    }
    if let Some(dir) = cwd.filter(|d| d.is_dir()) {
        cmd.current_dir(dir);
    }
    #[cfg(unix)]
    cmd.process_group(0);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let shown = secrets::scrub_text(&command_raw).shown;
            result.error = Some(secrets::scrub_text(&format!("cannot start `{shown}`: {e}")).shown);
            result.duration_ms = started.elapsed().as_millis();
            return result;
        }
    };

    // A reader thread for stdout lines, capped in total size. Stderr is discarded: a
    // server can print its environment there, and nothing in the result needs it.
    let (tx, rx) = mpsc::channel::<String>();
    if let Some(out) = child.stdout.take() {
        let tx = tx.clone();
        std::thread::spawn(move || {
            let out = out.take(MAX_STDOUT);
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
    }
    drop(tx);

    let deadline = started + timeout;
    let outcome = (|| -> Result<(), String> {
        let stdin = child.stdin.as_mut().ok_or("no stdin")?;
        let init = json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {"name": "mcproster", "version": env!("CARGO_PKG_VERSION")}
        }});
        send(stdin, &init)?;
        let reply = wait_for(&rx, 1, deadline)?;
        if let Some(err) = reply.get("error") {
            return Err(format!("initialize refused: {err}"));
        }
        let r = reply.get("result").cloned().unwrap_or(Value::Null);
        result.protocol_version = r
            .get("protocolVersion")
            .and_then(Value::as_str)
            .map(str::to_string);
        result.server_name = r
            .pointer("/serverInfo/name")
            .and_then(Value::as_str)
            .map(str::to_string);
        // An empty name or version is reported as absent, not as an empty string.
        result.server_version = r
            .pointer("/serverInfo/version")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string);
        result.server_name = result.server_name.take().filter(|n| !n.trim().is_empty());
        send(
            stdin,
            &json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        )?;
        send(
            stdin,
            &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
        )?;
        let reply = wait_for(&rx, 2, deadline)?;
        if let Some(err) = reply.get("error") {
            return Err(format!("tools/list refused: {err}"));
        }
        if let Some(tools) = reply.pointer("/result/tools").and_then(Value::as_array) {
            for t in tools {
                let name = t
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("?")
                    .to_string();
                let description = t.get("description").and_then(Value::as_str).map(|d| {
                    let d = d.lines().next().unwrap_or("").trim();
                    if d.chars().count() > 120 {
                        format!("{}…", d.chars().take(119).collect::<String>())
                    } else {
                        d.to_string()
                    }
                });
                let json_len = serde_json::to_string(t)
                    .map(|j| j.chars().count())
                    .unwrap_or(0);
                let tokens = estimate_tokens(json_len);
                result.schema_bytes += json_len as u64;
                result.estimated_tokens += tokens;
                result.tools.push(ToolInfo {
                    name,
                    description,
                    tokens,
                });
            }
        }
        Ok(())
    })();

    stop(&mut child);
    result.duration_ms = started.elapsed().as_millis();
    match outcome {
        Ok(()) => result.ok = true,
        Err(e) => result.error = Some(secrets::scrub_text(&e).shown),
    }
    result
}

/// Stop the server and everything it started. The child is its own process group leader,
/// so a group kill also stops `npx`, `uvx` or a wrapper shell. `kill` is used instead of
/// libc so the crate keeps `#![forbid(unsafe_code)]`.
fn stop(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let pid = child.id();
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{pid}")])
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

fn send(stdin: &mut std::process::ChildStdin, msg: &Value) -> Result<(), String> {
    let mut line = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    line.push('\n');
    stdin
        .write_all(line.as_bytes())
        .map_err(|e| format!("server closed its input: {e}"))?;
    stdin.flush().map_err(|e| e.to_string())
}

/// Wait for the JSON-RPC response with the given id, skipping notifications and other
/// output, until the deadline.
fn wait_for(rx: &mpsc::Receiver<String>, id: u64, deadline: Instant) -> Result<Value, String> {
    loop {
        let now = Instant::now();
        if now >= deadline {
            return Err(format!(
                "no answer within the time limit (waiting for response {id})"
            ));
        }
        match rx.recv_timeout(deadline - now) {
            Ok(line) => {
                let Ok(v) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                if v.get("id").and_then(Value::as_u64) == Some(id) {
                    return Ok(v);
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                return Err(format!(
                    "no answer within the time limit (waiting for response {id})"
                ));
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err("server exited before answering".to_string());
            }
        }
    }
}

/// Where probe results are kept: the only directory MCProster writes to (BRIEF R8).
pub fn results_file(home: &Path) -> PathBuf {
    home.join(".config/mcproster/probes.json")
}

/// Load saved results, keyed by server identity.
pub fn load_results(home: &Path) -> BTreeMap<String, ProbeResult> {
    load_results_at(&results_file(home))
}

/// Save results. The file holds dates, names, versions, tool names, sizes and masked
/// errors. It never holds the command line, the server's stderr, or a clear secret: the
/// command line can contain a value expanded from the process environment.
pub fn save_results(home: &Path, results: &BTreeMap<String, ProbeResult>) -> std::io::Result<()> {
    save_results_at(&results_file(home), results)
}

/// The same reader under an explicit path, for the hosts that keep their state elsewhere.
pub fn load_results_at(path: &Path) -> BTreeMap<String, ProbeResult> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// The same writer under an explicit path; creates the parent directory.
pub fn save_results_at(
    path: &Path,
    results: &BTreeMap<String, ProbeResult>,
) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(
        path,
        serde_json::to_string_pretty(results).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_expand_from_the_given_variables_only() {
        let mut vars = BTreeMap::new();
        vars.insert("TOKEN".to_string(), "abc".to_string());
        assert_eq!(expand_refs("Bearer ${TOKEN}", &vars), "Bearer abc");
        assert_eq!(expand_refs("${env:TOKEN}/x", &vars), "abc/x");
        assert_eq!(expand_refs("{env:TOKEN}", &vars), "abc");
        assert_eq!(expand_refs("${MISSING:-dflt}", &vars), "dflt");
        assert_eq!(expand_refs("${MISSING}", &vars), "${MISSING}");
    }

    #[test]
    fn results_can_live_at_an_explicit_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/probes.json");
        let empty = BTreeMap::new();
        save_results_at(&path, &empty).unwrap();
        assert!(path.is_file());
        assert!(load_results_at(&path).is_empty());
    }
}
