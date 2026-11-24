//! Sammelt alle Ausnahme-Stellen und harten Verstöße eines Repos (SICHERHEIT B.4, Review M-2), Datei für Datei:
//! Rust-Quelltext, `deny.toml`, `clippy.toml`, Manifeste und `.cargo/config.toml`. Fehlen `deny.toml` oder
//! `clippy.toml`, werden sie wie leere Dateien geprüft: Dann fehlen alle Sperren, und das ist rot.
use crate::{cargo_config, deny_policy, manifest, rust_attrs, toml_check};

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
        if name == "build.rs" {
            // Ein Build-Skript führt beim Bauen eigenen Code aus, auch vor Clippy und Tests: nur mit Eintrag.
            add(&mut sites, p, vec!["build.rs".to_string()]);
        }
        if p.ends_with(".rs") {
            let text = read(p);
            add(&mut sites, p, rust_attrs::sites(&text));
            if rust_attrs::macro_attributes(&text) > 0 {
                problems.push(format!(
                    "{p}: Attribut aus einem Makro-Platzhalter (`#[$…]`) ist nicht erlaubt"
                ));
            }
            continue;
        }
        if name.ends_with(".toml") {
            problems.extend(
                toml_check::problems(&read(p))
                    .into_iter()
                    .map(|m| format!("{p}: {m}")),
            );
        }
        if name == "Cargo.toml" {
            let text = read(p);
            problems.extend(manifest::crate_problems(p, &text));
            add(&mut sites, p, manifest::crate_sites(&text));
        } else if (name == "clippy.toml" || name == ".clippy.toml") && p != "clippy.toml" {
            problems.push(format!(
                "{p}: nur eine clippy.toml im Wurzelordner ist erlaubt (Clippy liest auch diese)"
            ));
        } else if cargo_config::is_config_file(p) {
            problems.extend(cargo_config::config_problems(p, &read(p)));
        } else if cargo_config::is_toolchain_file(p) {
            problems.push(format!(
                "{p}: eigene Toolchain-Datei ist nicht erlaubt (die CI nagelt die Version fest)"
            ));
        } else if p.starts_with(".github/workflows/") {
            add(&mut sites, p, cargo_config::workflow_sites(&read(p)));
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
