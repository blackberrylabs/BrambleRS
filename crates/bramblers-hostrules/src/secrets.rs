//! Secret detection and masking. A value is classified the moment it is read; the clear
//! text of a secret never leaves this module — `Masked` holds only the shown form.
//!
//! Three kinds of value:
//! - a *reference*: the host expands it itself (`${VAR}`, `${env:NAME}`, `{env:VAR}`,
//!   `${{ secrets.NAME }}`, `$VAR`, `%VAR%`): never a secret, shown as written;
//! - a *secret*: the key name says so (KEY, TOKEN, SECRET, PASSWORD, …) or the value has
//!   the shape of a credential (`sk-…`, `ghp_…`, a JWT, 32+ hex, …): masked;
//! - a *plain* value: shown as written.

use regex::Regex;
use serde::Serialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueKind {
    Plain,
    Reference,
    Secret,
}

/// A value as it may be shown. For a secret, `shown` is the masked form only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Masked {
    pub shown: String,
    pub kind: ValueKind,
}

fn key_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?i)(key|token|secret|passw|credential|auth|bearer|api)").unwrap()
    })
}

/// Credential shapes with a known prefix. Kept in one place: the whole-value check and the
/// search inside free text must agree.
const SHAPES: &str = concat!(
    r"sk-[A-Za-z0-9_-]{16,}",              // OpenAI / Anthropic style
    r"|sk_(?:live|test)_[A-Za-z0-9]{16,}", // Stripe
    r"|rk_(?:live|test)_[A-Za-z0-9]{16,}", // Stripe restricted
    r"|gh[pousr]_[A-Za-z0-9]{20,}",        // GitHub
    r"|github_pat_[A-Za-z0-9_]{20,}",
    r"|xox[abp]-[A-Za-z0-9-]{10,}",                 // Slack
    r"|xapp-[0-9]-[A-Za-z0-9-]{10,}",               // Slack app-level
    r"|AKIA[0-9A-Z]{16}",                           // AWS access key id
    r"|AIza[0-9A-Za-z_-]{30,}",                     // Google API key
    r"|ntn_[A-Za-z0-9]{20,}",                       // Notion
    r"|glpat-[A-Za-z0-9_-]{20,}",                   // GitLab
    r"|shpat_[A-Za-z0-9]{32,}",                     // Shopify
    r"|SG\.[A-Za-z0-9_-]{16,}\.[A-Za-z0-9_-]{16,}", // SendGrid
    r"|SK[0-9a-fA-F]{32}",                          // Twilio
    r"|eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}", // JWT
);

/// A pure hexadecimal blob is a credential when it is the whole value, but not when it
/// appears inside a command: commit hashes and object ids are 40 hex characters.
const HEX_BLOB: &str = r"[0-9a-fA-F]{32,}";

fn shape_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(&format!("^({SHAPES}|{HEX_BLOB})$")).unwrap())
}

fn embedded_shape_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(SHAPES).unwrap())
}

/// `KEY=value`, `KEY: value`, with optional quotes around the key or the value.
fn assignment_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r#"(?i)("?[A-Za-z_][A-Za-z0-9_.-]{0,60}"?)(\s*[=:]\s*)("[^"\n]*"|'[^'\n]*'|[^\s&;|]+)"#,
        )
        .unwrap()
    })
}

/// Names that are a credential even when the value is only digits. `API` and `AUTH` alone
/// are not enough: `API_PORT=8080` is a setting, `PASSWORD=12345678` is a secret.
fn key_is_strongly_secret(key: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(
            r"(?i)(passw|passphrase|secret|token|credential|bearer|private[_-]?key|access[_-]?key|api[_-]?key|apikey)",
        )
        .unwrap()
    })
    .is_match(key)
}

fn reference_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"^\s*(\$\{[^}]*\}|\$\{\{[^}]*\}\}|\{(env|file):[^}]*\}|\$[A-Za-z_][A-Za-z0-9_]*|%[A-Za-z_][A-Za-z0-9_]*%)\s*$").unwrap()
    })
}

/// Does the value contain a reference the host expands (anywhere in the string)?
pub fn contains_reference(value: &str) -> bool {
    value.contains("${")
        || value.contains("{env:")
        || value.contains("{file:")
        || value.contains('%') && value.matches('%').count() >= 2
}

/// Is the whole value a reference?
pub fn is_reference(value: &str) -> bool {
    reference_re().is_match(value)
}

/// Does the key name suggest a credential?
pub fn key_looks_secret(key: &str) -> bool {
    key_re().is_match(key)
}

/// Does the value have the shape of a credential? A `Bearer <token>` header counts.
pub fn value_looks_secret(value: &str) -> bool {
    let v = value.trim();
    let v = v
        .strip_prefix("Bearer ")
        .or_else(|| v.strip_prefix("bearer "))
        .unwrap_or(v)
        .trim();
    if v.len() < 16 {
        return false;
    }
    if shape_re().is_match(v) {
        return true;
    }
    // Long opaque string without spaces or path separators, mixed classes.
    v.len() >= 40
        && !v.contains(' ')
        && !v.contains('/')
        && !v.contains('.')
        && v.chars().any(|c| c.is_ascii_digit())
        && v.chars().any(|c| c.is_ascii_alphabetic())
}

/// Mask a value: first 6 and last 4 characters when long enough, otherwise dots only.
pub fn mask(value: &str) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() >= 16 {
        let head: String = chars[..6].iter().collect();
        let tail: String = chars[chars.len() - 4..].iter().collect();
        format!("{head}…{tail}")
    } else {
        "••••••••".to_string()
    }
}

/// A value that stays readable even under a secret-looking name: a URL, a path, a
/// boolean, or a number under a name that is not strongly secret (`API_PORT=8080`).
fn may_stay_plain(key: &str, value: &str) -> bool {
    let v = value.trim().trim_matches(|c| c == '"' || c == '\'');
    if v.starts_with("http://")
        || v.starts_with("https://")
        || v.starts_with('/')
        || v.starts_with("~/")
        || v.eq_ignore_ascii_case("true")
        || v.eq_ignore_ascii_case("false")
    {
        return true;
    }
    v.chars().all(|c| c.is_ascii_digit()) && !key_is_strongly_secret(key)
}

/// Classify a key/value pair and produce what may be shown.
pub fn classify(key: &str, value: &str) -> Masked {
    if is_reference(value) || (contains_reference(value) && !value_looks_secret(value)) {
        return Masked {
            shown: value.to_string(),
            kind: ValueKind::Reference,
        };
    }
    if key_looks_secret(key) && !value.is_empty() || value_looks_secret(value) {
        // A URL, a path or a flag under a secret-looking name is not a secret
        // (APIPRICEINDEX_URL). A numeric password under PASSWORD is one.
        if key_looks_secret(key) && !value_looks_secret(value) && may_stay_plain(key, value) {
            return Masked {
                shown: value.to_string(),
                kind: ValueKind::Plain,
            };
        }
        return Masked {
            shown: mask(value),
            kind: ValueKind::Secret,
        };
    }
    Masked {
        shown: value.to_string(),
        kind: ValueKind::Plain,
    }
}

/// The quote around an assignment value, if any.
fn quote_parts(value: &str) -> (&str, &str) {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        (&value[..1], &value[1..value.len() - 1])
    } else {
        ("", value)
    }
}

/// A free-text value (argument, URL, shell line): mask every credential shape, every
/// secret-looking assignment, and every secret query parameter, wherever they sit.
pub fn scrub_text(text: &str) -> Masked {
    let mut changed = false;

    // Known shapes first: `sk-…`, `ghp_…`, a JWT, a Stripe or Twilio key, anywhere.
    let mut shown = embedded_shape_re()
        .replace_all(text, |c: &regex::Captures| {
            if is_reference(&c[0]) {
                return c[0].to_string();
            }
            changed = true;
            mask(&c[0])
        })
        .into_owned();

    // `KEY=value` and `KEY: value`, with optional quotes.
    shown = assignment_re()
        .replace_all(&shown, |c: &regex::Captures| {
            let key = &c[1];
            let value = &c[3];
            if value.is_empty()
                || value.contains('…')
                || is_reference(value)
                || !key_looks_secret(key)
                || may_stay_plain(key, value)
            {
                return c[0].to_string();
            }
            let (quote, inner) = quote_parts(value);
            changed = true;
            format!("{}{}{quote}{}{quote}", &c[1], &c[2], mask(inner))
        })
        .into_owned();

    // Query parameters with a secret-looking name.
    if let Some(q) = shown.find('?') {
        let base = shown[..q].to_string();
        let query = shown[q + 1..].to_string();
        let mut qchanged = false;
        let parts: Vec<String> = query
            .split('&')
            .map(|kv| match kv.split_once('=') {
                Some((k, v))
                    if key_looks_secret(k)
                        && !v.is_empty()
                        && !v.contains('…')
                        && !is_reference(v) =>
                {
                    qchanged = true;
                    format!("{k}={}", mask(v))
                }
                _ => kv.to_string(),
            })
            .collect();
        if qchanged {
            changed = true;
            shown = format!("{base}?{}", parts.join("&"));
        }
    }

    if changed {
        return Masked {
            shown,
            kind: ValueKind::Secret,
        };
    }
    if value_looks_secret(&shown) && !is_reference(&shown) {
        return Masked {
            shown: mask(&shown),
            kind: ValueKind::Secret,
        };
    }
    Masked {
        shown,
        kind: ValueKind::Plain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn references_are_never_secrets() {
        for v in [
            "${OPENAI_API_KEY}",
            "${env:TOKEN}",
            "{env:MY_API_KEY}",
            "${{ secrets.X }}",
            "$TOKEN",
            "%API_KEY%",
            "${VAR:-default}",
        ] {
            let m = classify("API_KEY", v);
            assert_eq!(m.kind, ValueKind::Reference, "{v}");
            assert_eq!(m.shown, v);
        }
        let m = classify("Authorization", "Bearer ${env:MY_SERVICE_TOKEN}");
        assert_eq!(m.kind, ValueKind::Reference);
    }

    #[test]
    fn literal_secrets_are_masked_by_name_and_by_shape() {
        let m = classify(
            "OPENAI_API_KEY",
            "sk-proj-abcdefghijklmnopqrstuvwxyz0123456789",
        );
        assert_eq!(m.kind, ValueKind::Secret);
        assert!(!m.shown.contains("abcdefghijklmnop"));
        assert_eq!(m.shown, "sk-pro…6789");
        let m = classify("SOMETHING", "ghp_abcdefghijklmnopqrstuvwxyz0123456789");
        assert_eq!(m.kind, ValueKind::Secret);
        let m = classify(
            "Authorization",
            "Bearer eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.abcdefghijklmnopqrst",
        );
        assert_eq!(m.kind, ValueKind::Secret);
        let m = classify("PASSWORD", "hunter2");
        assert_eq!(m.kind, ValueKind::Secret);
        assert_eq!(m.shown, "••••••••");
    }

    #[test]
    fn urls_and_flags_under_secret_like_names_stay_plain() {
        assert_eq!(
            classify("APIPRICEINDEX_URL", "https://apipriceindex.com").kind,
            ValueKind::Plain
        );
        assert_eq!(classify("AUTH_ENABLED", "true").kind, ValueKind::Plain);
        assert_eq!(classify("NODE_ENV", "production").kind, ValueKind::Plain);
    }

    #[test]
    fn numeric_passwords_are_masked_but_ports_are_not() {
        let m = classify("PASSWORD", "12345678");
        assert_eq!(m.kind, ValueKind::Secret);
        assert_eq!(m.shown, "••••••••");
        let m = classify("DB_PASSWORD", "1234");
        assert_eq!(m.kind, ValueKind::Secret);
        assert_eq!(classify("API_PORT", "8080").kind, ValueKind::Plain);
        assert_eq!(classify("AUTH_ENABLED", "true").kind, ValueKind::Plain);
    }

    #[test]
    fn assignments_in_a_shell_line_are_masked() {
        let m = scrub_text("export ANTHROPIC_API_KEY=sk-ant-api03-abcdefghijklmnopqrstuvwxyz");
        assert_eq!(m.kind, ValueKind::Secret);
        assert!(!m.shown.contains("abcdefghijklmnop"));
        assert!(m.shown.contains("ANTHROPIC_API_KEY=sk-ant…"));

        let token = "AbCdEfGhIjKlMnOpQrStUvWxYz012345";
        let m = scrub_text(&format!("TOKEN={token}"));
        assert_eq!(m.kind, ValueKind::Secret);
        assert!(!m.shown.contains(token));
        assert!(m.shown.starts_with("TOKEN=AbCdEf…"));
    }

    #[test]
    fn provider_shapes_are_masked_inside_text() {
        // Assembled at run time: a test fixture must never have the exact shape of a real
        // key, or GitHub's push protection blocks the commit (seen on 2026-11-07).
        let stripe = ["sk_", "live_", "51AbCdEfGhIjKlMnOpQrStUv"].concat();
        let twilio = ["SK", "0123456789abcdef", "0123456789abcdef"].concat();
        let sendgrid = ["SG", ".abcdefghijklmnop", ".qrstuvwxyz012345"].concat();
        let shopify = ["shpat_", "0123456789abcdef", "0123456789abcdef"].concat();
        for value in [stripe, twilio, sendgrid, shopify] {
            let m = scrub_text(&format!("run with {value} now"));
            assert_eq!(m.kind, ValueKind::Secret, "{value}");
            assert!(!m.shown.contains(&value), "{value} came out");
        }
    }

    #[test]
    fn url_query_tokens_are_scrubbed() {
        let m = scrub_text("https://mcp.example.com/sse?api_key=0123456789abcdef0123456789abcdef");
        assert_eq!(m.kind, ValueKind::Secret);
        assert!(m
            .shown
            .starts_with("https://mcp.example.com/sse?api_key=012345…"));
        assert_eq!(scrub_text("/home/u/project").kind, ValueKind::Plain);
    }
}
