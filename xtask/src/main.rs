//! Eigene Prüfwerkzeuge des Repos (SICHERHEIT Anhang D), nur Standardbibliothek, kein Netz:
//! - `cargo xtask scan` prüft alle versionierten Dateien, `--staged` nur die vorgemerkten (Pre-Commit-Haken).
//! - `cargo xtask exceptions` vergleicht Ausnahmen im Code mit `docs/SICHERHEIT-AUSNAHMEN.md`.
mod cargo_config;
mod date;
mod deny_policy;
mod exceptions;
mod files;
mod manifest;
mod patterns;
mod policy;
mod rust_attrs;
mod rust_tokens;
mod scan;
mod toml_lite;
mod wordlist;

use std::process::ExitCode;

fn run_scan(staged: bool) -> Result<String, Vec<String>> {
    let (words, with_list) = wordlist::load();
    let table =
        exceptions::parse_table(&std::fs::read_to_string(exceptions::TABLE).unwrap_or_default());
    let allowed = exceptions::scan_allowances(&table, &date::today());
    let paths = if staged {
        files::staged()
    } else {
        files::tracked()
    }
    .map_err(|e| vec![e])?;
    let mut findings = Vec::new();
    for path in &paths {
        let content = if staged {
            Some(files::staged_content(path).map_err(|e| vec![e])?)
        } else {
            files::worktree_content(path)
        };
        let found = scan::scan_file(path, &content.unwrap_or_default(), &words);
        findings.extend(
            found
                .into_iter()
                .filter(|f| !allowed.contains(&(f.path.clone(), f.rule.to_string()))),
        );
    }
    if findings.is_empty() {
        let list = if with_list {
            "mit lokaler Wortliste"
        } else {
            "ohne lokale Wortliste"
        };
        return Ok(format!(
            "scan: {} Datei(en) geprüft, keine Funde ({list}).",
            paths.len()
        ));
    }
    let mut out: Vec<String> = findings
        .iter()
        .map(|f| format!("{}:{}: {}", f.path, f.line, f.rule))
        .collect();
    out.push(format!(
        "scan: {} Fund(e). Echte Daten und Geheimnisse gehören nicht ins Repository.",
        findings.len()
    ));
    Err(out)
}

fn run_exceptions() -> Result<String, Vec<String>> {
    let paths = files::tracked().map_err(|e| vec![e])?;
    let read = |p: &str| scan::decode_text(&files::worktree_content(p).unwrap_or_default());
    let (sites, mut problems) = policy::collect(&paths, read);
    let table = read(exceptions::TABLE);
    problems.extend(exceptions::check(
        &sites,
        &exceptions::parse_table(&table),
        &date::today(),
    ));
    problems.extend(exceptions::mirror_problems(
        &read("deny.toml"),
        &read(".cargo/audit.toml"),
    ));
    if problems.is_empty() {
        Ok(format!(
            "exceptions: {} Ausnahme-Stelle(n), alle eingetragen und gültig.",
            sites.len()
        ))
    } else {
        Err(problems)
    }
}

/// Ausgabe auf die Konsole ist die Aufgabe dieses Werkzeugs (läuft nie in einer App).
#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "SICHERHEIT-AUSNAHMEN A-015"
)]
fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("scan") => run_scan(args.iter().any(|a| a == "--staged")),
        Some("exceptions") => run_exceptions(),
        _ => Err(vec![
            "Aufruf: cargo xtask scan [--staged] | cargo xtask exceptions".to_string(),
        ]),
    };
    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(problems) => {
            for p in problems {
                eprintln!("{p}");
            }
            ExitCode::FAILURE
        }
    }
}
