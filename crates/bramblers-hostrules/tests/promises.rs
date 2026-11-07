//! The two promises of the crate, checked on its own source: no function writes a file,
//! and no dependency reaches the network.

use std::fs;
use std::path::Path;

fn sources() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    for e in fs::read_dir(dir).unwrap().flatten() {
        let p = e.path();
        if p.extension().is_some_and(|x| x == "rs") {
            // Unit tests build fixtures in a temporary directory; only the crate code counts.
            let text = fs::read_to_string(&p).unwrap();
            let code = match text.find("#[cfg(test)]") {
                Some(i) => text[..i].to_string(),
                None => text,
            };
            out.push((p.display().to_string(), code));
        }
    }
    out
}

#[test]
fn no_source_file_writes_removes_or_renames() {
    for (file, text) in sources() {
        for needle in [
            "fs::write",
            "File::create",
            "OpenOptions",
            "fs::remove",
            "fs::rename",
            "fs::create_dir",
            "write_all",
            "fs::copy",
            "set_permissions",
        ] {
            assert!(
                !text.contains(needle),
                "{file} contains `{needle}`: this crate never writes"
            );
        }
    }
}

#[test]
fn no_network_dependency() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    for needle in ["reqwest", "ureq", "hyper", "tokio", "curl", "http", "net"] {
        assert!(
            !manifest.lines().any(|l| l.trim_start().starts_with(needle)),
            "Cargo.toml depends on `{needle}`"
        );
    }
    for (file, text) in sources() {
        assert!(!text.contains("std::net"), "{file} uses std::net");
    }
}
