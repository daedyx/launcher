//! Dateien aus Git lesen: alle versionierten (CI) oder nur die vorgemerkten (Pre-Commit-Haken).
use std::process::Command;

/// Einzige Stelle, an der das Werkzeug ein Programm startet: `git` mit festen Argumenten.
#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-001")]
fn git(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("git")
        .args(args)
        .output()
        .map_err(|_| "git lässt sich nicht starten".to_string())?;
    if !out.status.success() {
        return Err(format!(
            "git {} ist fehlgeschlagen",
            args.first().unwrap_or(&"")
        ));
    }
    Ok(out.stdout)
}

/// Pfadliste aus `git … -z` (durch NUL getrennt). Nicht als UTF-8 lesbare Namen werden gemeldet.
pub fn split_names(raw: &[u8]) -> Result<Vec<String>, String> {
    raw.split(|c| *c == 0)
        .filter(|s| !s.is_empty())
        .map(|s| String::from_utf8(s.to_vec()).map_err(|_| "Dateiname ist kein UTF-8".to_string()))
        .collect()
}

pub fn tracked() -> Result<Vec<String>, String> {
    split_names(&git(&["ls-files", "-z"])?)
}

pub fn staged() -> Result<Vec<String>, String> {
    split_names(&git(&[
        "diff",
        "--cached",
        "--name-only",
        "-z",
        "--diff-filter=ACMR",
    ])?)
}

/// Inhalt so, wie er committet würde (aus dem Index, nicht aus dem Arbeitsordner).
pub fn staged_content(path: &str) -> Result<Vec<u8>, String> {
    git(&["show", &format!(":{path}")])
}

/// Inhalt im Arbeitsordner; `None`, wenn die Datei dort gelöscht ist.
pub fn worktree_content(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}
