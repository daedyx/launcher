//! Erlaubnisliste für Manifeste (Review Prüflücken M-5, N-8). Jeder Schlüssel, der hier nicht steht, ist eine
//! Ausnahme-Stelle `manifest:<schlüssel>`. Gemeint sind vor allem Einstellungen, die Tests oder Prüfungen still
//! abschalten (`[lib] test = false`, `harness`, `doctest`, `autotests`), ein eigenes Build-Skript (`build`) und
//! Ersatz-Abhängigkeiten (`[patch]`, `[replace]`). Unbekanntes ist immer ein Fund.
use crate::toml_lite::{items, unquote, Item};

/// Erlaubte Felder in `[package]` (auch als `<feld>.workspace = true`).
const PACKAGE_KEYS: [&str; 15] = [
    "name",
    "version",
    "edition",
    "description",
    "license",
    "license-file",
    "publish",
    "authors",
    "repository",
    "readme",
    "homepage",
    "documentation",
    "keywords",
    "categories",
    "rust-version",
];
/// Abhängigkeits- und Merkmal-Tabellen: ihr Inhalt ist frei (die Lieferkette prüft `cargo deny`).
const FREE_TABLES: [&str; 4] = [
    "dependencies",
    "dev-dependencies",
    "build-dependencies",
    "features",
];
/// Erlaubte Felder je Ziel (`[lib]`, `[[bin]]`, `[[test]]`, `[[example]]`, `[[bench]]`).
const TARGET_KEYS: [&str; 3] = ["name", "path", "required-features"];
const TARGETS: [&str; 5] = ["lib", "bin", "test", "example", "bench"];
/// Feste Werte im Wurzel-Manifest (`[profile.release]`).
const ROOT_FIXED: [(&str, &str); 3] = [
    ("profile.release.overflow-checks", "true"),
    ("profile.release.debug", "false"),
    ("profile.release.strip", "symbols"),
];

fn crate_key_allowed(key: &str) -> bool {
    let (head, rest) = key.split_once('.').unwrap_or((key, ""));
    match head {
        "package" => {
            let (field, tail) = rest.split_once('.').unwrap_or((rest, ""));
            PACKAGE_KEYS.contains(&field) && (tail.is_empty() || tail == "workspace")
        }
        "lints" => true, // genau `workspace = true`, hart geprüft in `manifest::crate_problems`
        "target" => FREE_TABLES.iter().any(|t| key.contains(&format!(".{t}."))),
        _ if FREE_TABLES.contains(&head) => !rest.is_empty(),
        _ => {
            let kind = head.split('#').next().unwrap_or(head);
            TARGETS.contains(&kind) && TARGET_KEYS.contains(&rest)
        }
    }
}

/// Ausnahme-Stellen eines Crate-Manifests (nur mit `[package]`).
pub fn crate_sites(text: &str) -> Vec<String> {
    let all = items(text);
    if !all.iter().any(|i| i.key.starts_with("package.")) {
        return Vec::new();
    }
    all.iter()
        .filter(|i| !crate_key_allowed(&i.key))
        .map(|i| format!("manifest:{}", i.key))
        .collect()
}

/// Ausnahme-Stellen des Wurzel-Manifests außerhalb der Lints (die prüft `manifest::workspace`).
pub fn root_sites(all: &[Item]) -> Vec<String> {
    let mut out = Vec::new();
    for i in all {
        let k = i.key.as_str();
        let head = k.split('.').next().unwrap_or(k);
        let crate_part =
            matches!(head, "package" | "lints" | "target") || FREE_TABLES.contains(&head) || {
                let kind = head.split('#').next().unwrap_or(head);
                TARGETS.contains(&kind)
            };
        if crate_part || k.starts_with("workspace.lints.") {
            continue; // Crate-Teil prüft `crate_sites`, Lints prüft `manifest::workspace`
        }
        if let Some((_, want)) = ROOT_FIXED.iter().find(|(f, _)| *f == k) {
            let v = unquote(&i.value);
            if v != *want {
                out.push(format!("manifest:{k}={v}"));
            }
            continue;
        }
        let allowed = matches!(k, "workspace.resolver" | "workspace.members")
            || k.starts_with("workspace.package.")
            || k.starts_with("workspace.dependencies.");
        if !allowed {
            out.push(format!("manifest:{k}"));
        }
    }
    out
}
