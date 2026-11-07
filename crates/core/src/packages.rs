//! Which package a machine takes: `.deb`, `.rpm` or AppImage, from `/etc/os-release`.
//!
//! Read once, never written. `/etc/os-release` is the system's own identity file, not a host
//! file (rule R1 is about the AI hosts' files); the family only ships x86_64 packages, so any
//! other architecture is sent to the releases page instead.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Deb,
    Rpm,
    AppImage,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Deb => ".deb",
            Kind::Rpm => ".rpm",
            Kind::AppImage => "AppImage",
        })
    }
}

/// The package kind for this machine, or None when the family ships nothing for it.
pub fn detect() -> Option<Kind> {
    if std::env::consts::ARCH != "x86_64" {
        return None;
    }
    let text = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
    Some(from_os_release(&text))
}

/// `ID` and `ID_LIKE` decide: the Debian family takes `.deb`, the RPM family `.rpm`, anyone
/// else the AppImage, which needs no package manager.
pub fn from_os_release(text: &str) -> Kind {
    let mut ids = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        for key in ["ID=", "ID_LIKE="] {
            if let Some(v) = line.strip_prefix(key) {
                ids.extend(
                    v.trim_matches('"')
                        .split_whitespace()
                        .map(|s| s.to_lowercase()),
                );
            }
        }
    }
    let any = |names: &[&str]| ids.iter().any(|i| names.contains(&i.as_str()));
    if any(&[
        "debian",
        "ubuntu",
        "linuxmint",
        "pop",
        "elementary",
        "zorin",
        "kali",
        "raspbian",
    ]) {
        Kind::Deb
    } else if any(&[
        "fedora",
        "rhel",
        "centos",
        "rocky",
        "almalinux",
        "suse",
        "opensuse",
        "opensuse-tumbleweed",
        "opensuse-leap",
        "sles",
        "mageia",
    ]) {
        Kind::Rpm
    } else {
        Kind::AppImage
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_family_of_the_system_picks_the_package() {
        assert_eq!(from_os_release("ID=ubuntu\nID_LIKE=debian\n"), Kind::Deb);
        assert_eq!(
            from_os_release("ID=linuxmint\nID_LIKE=\"ubuntu debian\"\n"),
            Kind::Deb
        );
        assert_eq!(from_os_release("ID=fedora\n"), Kind::Rpm);
        assert_eq!(
            from_os_release("ID=\"opensuse-tumbleweed\"\nID_LIKE=\"suse opensuse\"\n"),
            Kind::Rpm
        );
        assert_eq!(from_os_release("ID=arch\n"), Kind::AppImage);
        assert_eq!(from_os_release(""), Kind::AppImage);
    }

    #[test]
    fn a_sheet_names_the_exact_release_asset() {
        let mut s = crate::spec::Modules::embedded()
            .specs
            .into_iter()
            .find(|s| s.id == "permitt")
            .unwrap();
        s.latest = Some("0.2.0".into());
        assert_eq!(s.package_url(Kind::Deb).unwrap(), "https://github.com/blackberrylabs/permitt/releases/download/v0.2.0/permitt_0.2.0-1_amd64.deb");
        assert_eq!(s.package_url(Kind::Rpm).unwrap(), "https://github.com/blackberrylabs/permitt/releases/download/v0.2.0/permitt-0.2.0-1.x86_64.rpm");
        assert_eq!(s.package_url(Kind::AppImage).unwrap(), "https://github.com/blackberrylabs/permitt/releases/download/v0.2.0/Permitt-0.2.0-x86_64.AppImage");
        s.latest = None;
        assert!(s.package_url(Kind::Deb).is_none());
    }
}
