//! Eigene Prüfwerkzeuge des Repos (SICHERHEIT Anhang D), nur Standardbibliothek, kein Netz:
//! - `cargo xtask scan` prüft alle versionierten Dateien, `--staged` nur die vorgemerkten (Pre-Commit-Haken).
//! - `cargo xtask exceptions` vergleicht Ausnahmen im Code mit `docs/SICHERHEIT-AUSNAHMEN.md`.
//! - `cargo xtask testzahl <protokoll>` prüft die Mindestzahl der Tests je Testprogramm.
//! - `--head` liest den Commit statt des Arbeitsordners (CI).
mod cargo_config;
mod cfg_eval;
mod date;
mod deny_policy;
mod exceptions;
mod files;
mod manifest;
mod manifest_allow;
mod patterns;
mod policy;
mod rust_attrs;
mod rust_tokens;
mod scan;
mod testcount;
mod toml_check;
mod toml_lite;
mod wordlist;

use std::process::ExitCode;

fn run_scan(src: files::Source) -> Result<String, Vec<String>> {
    let (words, with_list) = wordlist::load();
    let table_text = scan::decode_text(&src.content(exceptions::TABLE).unwrap_or_default());
    let allowed =
        exceptions::scan_allowances(&exceptions::parse_table(&table_text), &date::today());
    let paths = src.paths().map_err(|e| vec![e])?;
    let mut findings = Vec::new();
    for path in &paths {
        let content = src.content(path).map_err(|e| vec![e])?;
        let found = scan::scan_file(path, &content, &words);
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

fn run_exceptions(src: files::Source) -> Result<String, Vec<String>> {
    let paths = src.paths().map_err(|e| vec![e])?;
    let read = |p: &str| scan::decode_text(&src.content(p).unwrap_or_default());
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

/// `testzahl <protokoll>`: Protokoll von `cargo test` gegen die Mindestzahlen (aus `HEAD` mit `--head`).
fn run_testcount(args: &[String], src: files::Source) -> Result<String, Vec<String>> {
    let log_path = args
        .iter()
        .skip(1)
        .find(|a| !a.starts_with("--"))
        .ok_or_else(|| vec!["testzahl: Protokoll-Datei fehlt".to_string()])?;
    let log = scan::decode_text(
        &std::fs::read(log_path).map_err(|_| vec![format!("testzahl: {log_path} nicht lesbar")])?,
    );
    let floor = scan::decode_text(&src.content(testcount::FLOOR).unwrap_or_default());
    let problems = testcount::check(&log, &floor);
    if problems.is_empty() {
        let n: u64 = testcount::counts(&log)
            .values()
            .map(|o| o.passed)
            .fold(0, u64::saturating_add);
        Ok(format!(
            "testzahl: {n} Tests bestanden, keine Mindestzahl unterschritten."
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
        Some("scan") => run_scan(files::Source::from_args(&args)),
        Some("exceptions") => run_exceptions(files::Source::from_args(&args)),
        Some("testzahl") => run_testcount(&args, files::Source::from_args(&args)),
        _ => {
            Err(vec!["Aufruf: xtask scan [--staged|--head] | exceptions [--head] | testzahl <protokoll> [--head]"
                .to_string()])
        }
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
