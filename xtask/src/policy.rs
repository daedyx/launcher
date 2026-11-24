//! Sammelt alle Ausnahme-Stellen und harten Verstöße eines Repos (SICHERHEIT B.4, Review M-2), Datei für Datei:
//! Rust-Quelltext, `deny.toml`, `clippy.toml`, Manifeste und `.cargo/config.toml`. Fehlen `deny.toml` oder
//! `clippy.toml`, werden sie wie leere Dateien geprüft: Dann fehlen alle Sperren, und das ist rot.
use crate::{deny_policy, manifest, rust_attrs};

/// (Datei, Regel) je Ausnahme-Stelle.
pub type Sites = Vec<(String, String)>;

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn add(sites: &mut Sites, path: &str, rules: Vec<String>) {
    sites.extend(rules.into_iter().map(|r| (path.to_string(), r)));
}

/// `paths`: alle versionierten Dateien (mit `/`), `read`: Inhalt als Text.
pub fn collect(paths: &[String], read: impl Fn(&str) -> String) -> (Sites, Vec<String>) {
    let mut sites = Sites::new();
    let mut problems = Vec::new();
    for path in paths {
        let p = path.as_str();
        let name = file_name(p);
        if p.ends_with(".rs") {
            add(&mut sites, p, rust_attrs::sites(&read(p)));
        } else if name == "Cargo.toml" {
            problems.extend(manifest::crate_problems(p, &read(p)));
        } else if (name == "clippy.toml" || name == ".clippy.toml") && p != "clippy.toml" {
            problems.push(format!(
                "{p}: nur eine clippy.toml im Wurzelordner ist erlaubt (Clippy liest auch diese)"
            ));
        } else if p.ends_with(".cargo/config.toml") || p.ends_with(".cargo/config") {
            add(&mut sites, p, manifest::config_sites(&read(p)));
        }
    }
    // Nur versionierte Dateien zählen; eine nicht eingecheckte Datei gilt als fehlend.
    let root = |f: &str| {
        if paths.iter().any(|p| p == f) {
            read(f)
        } else {
            String::new()
        }
    };
    let (ws_problems, ws_sites) = manifest::workspace(&root("Cargo.toml"));
    problems.extend(ws_problems);
    add(&mut sites, "Cargo.toml", ws_sites);
    add(
        &mut sites,
        "deny.toml",
        deny_policy::sites(&root("deny.toml")),
    );
    add(
        &mut sites,
        "clippy.toml",
        manifest::clippy_sites(&root("clippy.toml")),
    );
    add(
        &mut sites,
        ".cargo/audit.toml",
        deny_policy::audit_sites(&root(".cargo/audit.toml")),
    );
    (sites, problems)
}

#[cfg(test)]
#[path = "policy_tests.rs"]
mod tests;
