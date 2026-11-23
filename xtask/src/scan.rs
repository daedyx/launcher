//! Scan eines Dateisatzes (SICHERHEIT 3.2): Dateinamen, Größe und jede Textzeile.
use crate::patterns;

pub const MAX_FILE_BYTES: usize = 1024 * 1024;
const FORBIDDEN_EXT: [&str; 13] = [
    "db", "sqlite", "sqlite3", "eml", "msg", "mbox", "pst", "ics", "vcf", "p12", "pfx", "pem",
    "key",
];
/// Erfundene Beispieldateien dürfen diese Endungen haben.
const FIXTURE_DIR: &str = "testdata/erfunden/";

/// Ein Fund; `line` 0 = betrifft die ganze Datei. Der gefundene Text selbst wird nie mitgegeben.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub path: String,
    pub line: usize,
    pub rule: &'static str,
}

/// Geheimnis-Verweise in Workflows: nur `secrets.GITHUB_TOKEN` ist erlaubt (plan-ohne-secrets).
fn has_workflow_secret(line: &str) -> bool {
    line.match_indices("secrets.").any(|(i, p)| {
        let rest = line.get(i.saturating_add(p.len())..).unwrap_or_default();
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        name != "GITHUB_TOKEN"
    })
}

fn line_rules(line: &str, words: &[String], workflow: bool) -> Vec<&'static str> {
    let checks = [
        (patterns::has_secret(line), "geheimnis"),
        (patterns::has_real_mail(line), "mailadresse"),
        (patterns::has_user_path(line), "benutzerpfad"),
        (patterns::has_private_ip(line), "private-ip"),
        (patterns::has_org_name(line), "organisation"),
        (patterns::has_listed_word(line, words), "lokale-wortliste"),
        (workflow && has_workflow_secret(line), "secret-im-workflow"),
    ];
    checks
        .into_iter()
        .filter(|(hit, _)| *hit)
        .map(|(_, rule)| rule)
        .collect()
}

/// Prüft eine Datei. `path` mit `/` als Trenner, relativ zum Repository.
pub fn scan_file(path: &str, content: &[u8], words: &[String]) -> Vec<Finding> {
    let mut rules: Vec<(usize, &'static str)> = Vec::new();
    let name = path.rsplit('/').next().unwrap_or(path);
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());
    if name == "CLAUDE.md" {
        rules.push((0, "claude-md-im-git"));
    }
    if name == ".env" || name.starts_with(".env.") {
        rules.push((0, "env-datei"));
    }
    if ext.is_some_and(|e| FORBIDDEN_EXT.contains(&e.as_str())) && !path.contains(FIXTURE_DIR) {
        rules.push((0, "verbotener-dateityp"));
    }
    if content.len() > MAX_FILE_BYTES {
        rules.push((0, "datei-ueber-1-mb"));
    }
    // Binärdateien (Bilder, Symbole) haben keine Textzeilen.
    let text = if content.contains(&0) {
        None
    } else {
        std::str::from_utf8(content).ok()
    };
    let workflow = path.starts_with(".github/workflows/");
    for (i, line) in text.unwrap_or_default().lines().enumerate() {
        for rule in line_rules(line, words, workflow) {
            rules.push((i.saturating_add(1), rule));
        }
    }
    rules
        .into_iter()
        .map(|(line, rule)| Finding {
            path: path.to_string(),
            line,
            rule,
        })
        .collect()
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod tests;
