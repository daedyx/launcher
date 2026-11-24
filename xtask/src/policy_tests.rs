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
        problems.iter().any(|m| m.starts_with(".cargo/config.toml")),
        "{problems:?}"
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

#[test]
fn cargo_settings_and_macro_attributes_stop_the_check() {
    let (paths, map) = files(&[
        (".cargo/config.toml", "[alias]\nxtask = \"run --quiet --package xtask --\"\n[target.x86_64-pc-windows-msvc]\nrunner = \"ok.cmd\"\n"),
        ("crates/a/.cargo/config.toml", "[alias]\nxtask = \"run --quiet --package xtask --\"\n"),
        ("rust-toolchain.toml", "[toolchain]\nchannel = \"1.99.1\"\n"),
        ("src/m.rs", "macro_rules! w { ($m:meta) => { #[$m] fn f() {} }; }"),
        ("deny.toml", "a = \"\"\"\nx\"\"\"\n"),
        (".github/workflows/ci.yml", "env:\n  RUSTC_WRAPPER: w.cmd\n"),
    ]);
    let (sites, problems) = collect(&paths, |p| map.get(p).cloned().unwrap_or_default());
    for p in [
        ".cargo/config.toml",
        "crates/a/.cargo/config.toml",
        "rust-toolchain.toml",
        "src/m.rs",
        "deny.toml",
    ] {
        assert!(
            problems.iter().any(|m| m.starts_with(p)),
            "{p}: {problems:?}"
        );
    }
    assert!(
        sites.contains(&(
            ".github/workflows/ci.yml".to_string(),
            "workflow:rustc_wrapper".to_string()
        )),
        "{sites:?}"
    );
}

#[test]
fn build_scripts_and_crate_settings_are_sites() {
    let (paths, map) = files(&[
        ("crates/a/build.rs", "fn main() {}"),
        (
            "crates/a/Cargo.toml",
            "[package]\nname = \"a\"\n[lib]\ntest = false\n[lints]\nworkspace = true\n",
        ),
    ]);
    let (sites, _) = collect(&paths, |p| map.get(p).cloned().unwrap_or_default());
    assert!(
        sites.contains(&("crates/a/build.rs".to_string(), "build.rs".to_string())),
        "{sites:?}"
    );
    assert!(
        sites.contains(&(
            "crates/a/Cargo.toml".to_string(),
            "manifest:lib.test".to_string()
        )),
        "{sites:?}"
    );
}
