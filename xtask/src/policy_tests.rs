use super::*;

fn files(list: &[(&str, &str)]) -> (Vec<String>, std::collections::BTreeMap<String, String>) {
    let paths = list.iter().map(|(p, _)| p.to_string()).collect();
    let map = list
        .iter()
        .map(|(p, t)| (p.to_string(), t.to_string()))
        .collect();
    (paths, map)
}

#[test]
fn every_kind_of_file_is_checked() {
    let (paths, map) = files(&[
        (
            "src/a.rs",
            "#[cfg_attr(all(), allow(clippy::unwrap_used))]\nfn f() {}",
        ),
        ("crates/x/Cargo.toml", "[package]\nname = \"x\"\n"),
        (
            "crates/x/clippy.toml",
            "avoid-breaking-exported-api = false\n",
        ),
        (
            ".cargo/config.toml",
            "[build]\nrustflags = [\"-A\", \"warnings\"]\n",
        ),
        ("README.md", "#[allow(x)] in Text zählt nicht"),
    ]);
    let (sites, problems) = collect(&paths, |p| map.get(p).cloned().unwrap_or_default());
    assert!(
        sites.contains(&("src/a.rs".to_string(), "clippy::unwrap_used".to_string())),
        "{sites:?}"
    );
    assert!(
        sites.contains(&(
            ".cargo/config.toml".to_string(),
            "build.rustflags".to_string()
        )),
        "{sites:?}"
    );
    // Fehlen deny.toml und clippy.toml, fehlen alle Sperren: das sind Stellen ohne Eintrag.
    assert!(
        sites
            .iter()
            .any(|(p, r)| p == "deny.toml" && r == "advisories.yanked=fehlt"),
        "{sites:?}"
    );
    assert!(
        sites
            .iter()
            .any(|(p, r)| p == "clippy.toml" && r.starts_with("disallowed-methods-fehlt=")),
        "{sites:?}"
    );
    assert!(
        problems
            .iter()
            .any(|m| m.starts_with("crates/x/Cargo.toml")),
        "{problems:?}"
    );
    assert!(
        problems
            .iter()
            .any(|m| m.starts_with("crates/x/clippy.toml")),
        "{problems:?}"
    );
    assert!(
        problems.iter().any(|m| m.starts_with("Cargo.toml")),
        "Grundliste der Workspace-Lints fehlt: {problems:?}"
    );
    assert!(!sites.iter().any(|(p, _)| p == "README.md"));
}
