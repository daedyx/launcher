//! Ausnahmenzähler (SICHERHEIT Anhang B.4): Stellen im Code gegen `docs/SICHERHEIT-AUSNAHMEN.md`.
//! Rot, wenn eine Datei mehr oder weniger Stellen hat als Einträge, oder ein Eintrag abgelaufen ist.
use std::collections::BTreeMap;

pub const TABLE: &str = "docs/SICHERHEIT-AUSNAHMEN.md";

/// Muster werden zusammengesetzt, damit diese Datei sich nicht selbst zählt.
const RUST_MARKERS: [&str; 8] = [
    concat!("#[", "allow("),
    concat!("#![", "allow("),
    concat!("#[", "expect("),
    concat!("#![", "expect("),
    concat!("#[", "ignore"),
    concat!("unsafe", " {"),
    concat!("unsafe", " fn"),
    concat!("unsafe", " impl"),
];

/// Ein Eintrag der Tabelle: Datei, Regel und Ablaufdatum `JJJJ-MM-TT`. Regeln `scan:<regel>` erlauben einen
/// Scan-Fund in genau dieser Datei (z. B. eine Altlast, deren Ablösung beschlossen ist) und zählen nicht als Code-Stelle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub path: String,
    pub rule: String,
    pub expires: String,
}

fn valid_until(e: &Entry, today: &str) -> bool {
    e.expires.len() == 10
        && e.expires.bytes().filter(|c| *c == b'-').count() == 2
        && e.expires.as_str() >= today
}

/// Erlaubte Scan-Funde (Datei, Regel) aus gültigen `scan:`-Zeilen.
pub fn scan_allowances(
    entries: &[Entry],
    today: &str,
) -> std::collections::BTreeSet<(String, String)> {
    entries
        .iter()
        .filter(|e| valid_until(e, today))
        .filter_map(|e| Some((e.path.clone(), e.rule.strip_prefix("scan:")?.to_string())))
        .collect()
}

/// Zeilen `| Nr. | Datei | Regel | Grund | Entscheid | Datum | Ablauf |`; Kopf- und Trennzeilen zählen nicht.
pub fn parse_table(md: &str) -> Vec<Entry> {
    md.lines()
        .filter(|l| l.trim_start().starts_with('|'))
        .filter_map(|l| {
            let cells: Vec<&str> = l.split('|').map(str::trim).collect();
            let nr = cells.get(1)?;
            if nr.is_empty() || nr.starts_with("Nr") || nr.starts_with("---") {
                return None;
            }
            Some(Entry {
                path: cells.get(2)?.to_string(),
                rule: cells.get(3)?.to_string(),
                expires: cells.get(7)?.to_string(),
            })
        })
        .collect()
}

/// Stellen in Rust-Quelltext; Kommentarzeilen zählen nicht.
pub fn count_rust(text: &str) -> usize {
    text.lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with("//"))
        .map(|l| {
            RUST_MARKERS
                .iter()
                .map(|m| l.matches(m).count())
                .sum::<usize>()
        })
        .sum()
}

/// Ausgenommene Meldungen in `deny.toml` (`id = "RUSTSEC-…"`, nicht auskommentiert).
pub fn count_deny(text: &str) -> usize {
    text.lines()
        .map(str::trim_start)
        .filter(|l| !l.starts_with('#') && l.contains(concat!("\"RUST", "SEC-")))
        .count()
}

/// Alle Meldungs-Kennungen `RUSTSEC-JJJJ-NNNN` in nicht auskommentierten Zeilen.
fn advisory_ids(text: &str) -> std::collections::BTreeSet<String> {
    let prefix = concat!("RUST", "SEC-");
    text.lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .flat_map(|l| {
            l.match_indices(prefix)
                .map(move |(i, _)| l.get(i..).unwrap_or_default())
        })
        .map(|rest| {
            rest.chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                .collect()
        })
        .collect()
}

/// `cargo audit` liest seine Ausnahmen aus `.cargo/audit.toml`; sie müssen genau denen in `deny.toml` entsprechen,
/// damit es nur eine Liste (mit Einträgen in der Tabelle) gibt.
pub fn mirror_problems(deny: &str, audit: &str) -> Vec<String> {
    if advisory_ids(deny) == advisory_ids(audit) {
        Vec::new()
    } else {
        vec!["Ausnahmen in .cargo/audit.toml und deny.toml unterscheiden sich".to_string()]
    }
}

/// Probleme (leer = grün). `found`: Datei → Anzahl Stellen (nur Dateien mit mindestens einer Stelle).
pub fn check(found: &BTreeMap<String, usize>, entries: &[Entry], today: &str) -> Vec<String> {
    let mut listed: BTreeMap<&str, usize> = BTreeMap::new();
    let mut problems = Vec::new();
    for e in entries {
        if !e.rule.starts_with("scan:") {
            let n = listed.entry(e.path.as_str()).or_insert(0);
            *n = n.saturating_add(1);
        }
        let valid_date =
            e.expires.len() == 10 && e.expires.bytes().filter(|c| *c == b'-').count() == 2;
        if !valid_date {
            problems.push(format!(
                "{}: Ablaufdatum fehlt oder ist kein JJJJ-MM-TT",
                e.path
            ));
        } else if e.expires.as_str() < today {
            problems.push(format!(
                "{}: Ausnahme ist am {} abgelaufen",
                e.path, e.expires
            ));
        }
    }
    let paths: std::collections::BTreeSet<&str> = found
        .keys()
        .map(String::as_str)
        .chain(listed.keys().copied())
        .collect();
    for p in paths {
        let (have, allowed) = (
            found.get(p).copied().unwrap_or(0),
            listed.get(p).copied().unwrap_or(0),
        );
        if have != allowed {
            problems.push(format!(
                "{p}: {have} Ausnahme-Stelle(n) im Code, {allowed} Eintrag/Einträge in {TABLE}"
            ));
        }
    }
    problems
}

#[cfg(test)]
#[path = "exceptions_tests.rs"]
mod tests;
