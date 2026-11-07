//! Opening a file or a link in the user's desktop session, with one policy for every view.
//!
//! `xdg-open` receives its target as one argument, never through a shell. Only http and
//! https are accepted as links: a project file must not send the user to `file:`, `smb:`
//! or a custom handler. Files keep the desktop's default application.

/// Open an http or https URL; any other scheme is refused.
pub fn url(target: &str) -> bool {
    if !url_allowed(target) {
        return false;
    }
    spawn(target)
}

/// Open a local file with the application the desktop associates with it.
pub fn file(target: &str) -> bool {
    spawn(target)
}

fn url_allowed(target: &str) -> bool {
    let lower = target.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

fn spawn(target: &str) -> bool {
    std::process::Command::new("xdg-open")
        .arg(target)
        .spawn()
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_http_and_https_links_are_opened() {
        assert!(url_allowed("https://github.com/blackberrylabs"));
        assert!(url_allowed("HTTP://EXAMPLE.COM"));
        for refused in [
            "file:///etc/passwd",
            "smb://host/share",
            "javascript:alert(1)",
            "ftp://example.com",
            "mailto:a@b",
        ] {
            assert!(!url_allowed(refused), "{refused}");
        }
    }
}
