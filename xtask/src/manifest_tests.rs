use super::*;

const CRATE: &str =
    "[package]\nname = \"a\"\n\n[dependencies]\nx = \"1\"\n\n[lints]\nworkspace = true\n";

#[test]
fn crate_manifest_must_use_the_workspace_lints_only() {
    assert!(crate_problems("crates/a/Cargo.toml", CRATE).is_empty());
    assert!(crate_problems(
        "crates/a/Cargo.toml",
        "lints = { workspace = true }\n[package]\nname = \"a\"\n"
    )
    .is_empty());
    assert!(
        crate_problems("Cargo.toml", "[workspace]\nmembers = []\n").is_empty(),
        "ohne [package] nichts zu prüfen"
    );
    let missing = crate_problems("crates/a/Cargo.toml", "[package]\nname = \"a\"\n");
    assert_eq!(missing.len(), 1, "{missing:?}");
    let own = format!("{CRATE}\n[lints.clippy]\nunwrap_used = \"allow\"\n");
    assert_eq!(crate_problems("crates/a/Cargo.toml", &own).len(), 1);
    let replaced = CRATE.replace("workspace = true", "workspace = false");
    assert_eq!(crate_problems("crates/a/Cargo.toml", &replaced).len(), 1);
    let only_own = CRATE.replace(
        "[lints]\nworkspace = true",
        "[lints.rust]\nunsafe_code = \"allow\"",
    );
    assert!(!crate_problems("crates/a/Cargo.toml", &only_own).is_empty());
}

const WORKSPACE: &str = r#"[workspace]
members = ["crates/*"]

[workspace.lints.rust]
unsafe_code = "forbid"
unused_must_use = "deny"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
unreachable = "deny"
todo = "deny"
unimplemented = "deny"
indexing_slicing = "deny"
string_slice = "deny"
get_unwrap = "deny"
print_stdout = "deny"
print_stderr = "deny"
dbg_macro = "deny"
undocumented_unsafe_blocks = "deny"
multiple_unsafe_ops_per_block = "deny"
cast_possible_truncation = "warn"
cast_sign_loss = "warn"
arithmetic_side_effects = "warn"
mem_forget = "deny"
exit = "deny"
"#;

#[test]
fn workspace_lints_keep_the_base_list() {
    let (p, s) = workspace(WORKSPACE);
    assert!(p.is_empty() && s.is_empty(), "{p:?} {s:?}");
    let (p, _) = workspace(&WORKSPACE.replace("unwrap_used = \"deny\"", "unwrap_used = \"allow\""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) = workspace(&WORKSPACE.replace("expect_used = \"deny\"\n", ""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) = workspace(&WORKSPACE.replace(
        "arithmetic_side_effects = \"warn\"",
        "arithmetic_side_effects = \"allow\"",
    ));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) =
        workspace(&WORKSPACE.replace("unsafe_code = \"forbid\"", "unsafe_code = \"deny\""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, _) = workspace(&WORKSPACE.replace("level = \"deny\"", "level = \"allow\""));
    assert_eq!(p.len(), 1, "{p:?}");
    let (p, s) = workspace(&format!("{WORKSPACE}too_many_lines = \"allow\"\n"));
    assert!(p.is_empty());
    assert_eq!(s, vec!["workspace.lints.clippy.too_many_lines=allow"]);
}

const CLIPPY: &str = r#"allow-unwrap-in-tests = true
allow-expect-in-tests = true
allow-panic-in-tests = true
allow-indexing-slicing-in-tests = true
disallowed-methods = [
  { path = "std::process::Command::new", reason = "x" },
  { path = "std::env::temp_dir", reason = "x" },
  { path = "std::net::TcpStream::connect", reason = "x" },
]
"#;

#[test]
fn clippy_toml_loosening_counts() {
    assert!(clippy_sites(CLIPPY).is_empty());
    let more = clippy_sites(&format!(
        "{CLIPPY}too-many-arguments-threshold = 99\nallow-print-in-tests = true\n"
    ));
    assert_eq!(
        more,
        vec![
            "too-many-arguments-threshold=99",
            "allow-print-in-tests=true"
        ]
    );
    let fewer =
        clippy_sites(&CLIPPY.replace("  { path = \"std::env::temp_dir\", reason = \"x\" },\n", ""));
    assert_eq!(fewer, vec!["disallowed-methods-fehlt=std::env::temp_dir"]);
    assert_eq!(
        clippy_sites("").len(),
        3,
        "fehlende clippy.toml = alle Sperren fehlen"
    );
}
