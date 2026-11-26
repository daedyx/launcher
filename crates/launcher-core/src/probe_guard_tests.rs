//! Wächter (Review U3 F-U3-2): Der Weg ohne Nachweis steht nur in `probe_local.rs` (Merkmal `probe`) und in den zwei
//! Probe-Befehlen der Kommandozeile; das Release baut ihn nie.
use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel))
        .unwrap()
        .replace("\r\n", "\n")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn the_path_without_attestation_lives_only_in_the_probe() {
    let mut files = Vec::new();
    rust_files(&repo().join("crates"), &mut files);
    let allowed = [
        "probe_local.rs",
        "probe_local_tests.rs",
        "probe_guard_tests.rs",
        "main.rs",
    ];
    for f in &files {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(f).unwrap();
        if text.contains("install_local") || text.contains("self_update_local") {
            assert!(allowed.contains(&name.as_str()), "{}", f.display());
        }
    }
    let lib = read("crates/launcher-core/src/lib.rs");
    assert!(lib.contains("#[cfg(feature = \"probe\")]\npub mod probe_local;"));
    let cli = read("crates/launcher-cli/src/main.rs");
    assert_eq!(cli.matches("install_local").count(), 1);
    assert_eq!(
        cli.matches("#[cfg(feature = \"probe\")]").count(),
        4,
        "zwei Befehle, zwei Zweige"
    );
}

#[test]
fn the_release_never_builds_the_probe() {
    let release = read(".github/workflows/release.yml");
    assert!(
        !release.contains("probe") && !release.contains("Probe"),
        "release.yml ohne Probe"
    );
    let build = read("scripts/build-setup.ps1");
    assert!(build.contains("if ($Probe)"), "Merkmal nur mit -Probe");
    for toml in [
        "crates/launcher-core/Cargo.toml",
        "crates/launcher-cli/Cargo.toml",
    ] {
        assert!(
            !read(toml).contains("default = ["),
            "{toml}: keine Standard-Merkmale"
        );
    }
}
