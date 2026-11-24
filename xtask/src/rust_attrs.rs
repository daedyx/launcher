//! Ausnahme-Stellen in Rust-Quelltext (SICHERHEIT B.4): `allow`/`expect` (auch in `cfg_attr`, mit Leerzeichen und
//! über mehrere Zeilen), `ignore` an Tests und das Schlüsselwort `unsafe`. Kommentare, Zeichenketten und Zeichen
//! zählen nicht. Je Stelle ein Text: die Regeln eines Attributs sortiert und mit Komma getrennt
//! (`clippy::exit,clippy::print_stdout`), oder `ignore` bzw. `unsafe`.

use crate::rust_tokens::{tokens, Tok};

/// Inhalt der Klammer, die bei `open` beginnt (alle Klammerarten zählen für die Tiefe).
fn group(t: &[Tok], open: usize) -> Option<&[Tok]> {
    let mut depth = 0usize;
    for (k, tok) in t.iter().enumerate().skip(open) {
        match tok {
            Tok::Punct('(' | '[' | '{') => depth = depth.saturating_add(1),
            Tok::Punct(')' | ']' | '}') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return t.get(open.saturating_add(1)..k);
                }
            }
            _ => {}
        }
    }
    None
}

/// Teile auf oberster Ebene, getrennt durch Kommas.
fn split_commas(t: &[Tok]) -> Vec<&[Tok]> {
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0usize, 0usize);
    for (k, tok) in t.iter().enumerate() {
        match tok {
            Tok::Punct('(' | '[' | '{') => depth = depth.saturating_add(1),
            Tok::Punct(')' | ']' | '}') => depth = depth.saturating_sub(1),
            Tok::Punct(',') if depth == 0 => {
                parts.push(t.get(start..k).unwrap_or_default());
                start = k.saturating_add(1);
            }
            _ => {}
        }
    }
    parts.push(t.get(start..).unwrap_or_default());
    parts.into_iter().filter(|p| !p.is_empty()).collect()
}

/// Pfad am Anfang (`clippy::x`) und die Anzahl der Teile, die er belegt.
fn path(t: &[Tok]) -> (String, usize) {
    let len = t
        .iter()
        .take_while(|x| matches!(x, Tok::Ident(_) | Tok::Punct(':')))
        .count();
    let text = t
        .get(..len)
        .unwrap_or_default()
        .iter()
        .fold(String::new(), |mut s, x| {
            match x {
                Tok::Ident(w) => s.push_str(w),
                Tok::Punct(p) => s.push(*p),
                Tok::Lit => {}
            }
            s
        });
    (text, len)
}

/// Ein Attribut-Inhalt (`allow(…)`, `cfg_attr(…, …)`, `ignore`, …) → Stellen.
fn meta(t: &[Tok], out: &mut Vec<String>) {
    let (name, len) = path(t);
    let inner = if t.get(len) == Some(&Tok::Punct('(')) {
        group(t, len)
    } else {
        None
    };
    match (name.as_str(), inner) {
        ("allow" | "expect", Some(inner)) => {
            let mut rules: Vec<String> = split_commas(inner)
                .into_iter()
                .filter(|p| !p.contains(&Tok::Punct('=')))
                .map(|p| path(p).0)
                .collect();
            rules.sort();
            out.push(rules.join(","));
        }
        ("cfg_attr", Some(inner)) => split_commas(inner)
            .into_iter()
            .skip(1)
            .for_each(|p| meta(p, out)),
        ("ignore", _) => out.push("ignore".to_string()),
        _ => {}
    }
}

/// Alle Ausnahme-Stellen einer Rust-Datei.
pub fn sites(src: &str) -> Vec<String> {
    let t = tokens(src);
    let mut out = Vec::new();
    for (i, tok) in t.iter().enumerate() {
        match tok {
            Tok::Ident(w) if w == "unsafe" => out.push("unsafe".to_string()),
            Tok::Punct('#') => {
                let bang = usize::from(t.get(i.saturating_add(1)) == Some(&Tok::Punct('!')));
                let open = i.saturating_add(1).saturating_add(bang);
                let inner = if t.get(open) == Some(&Tok::Punct('[')) {
                    group(&t, open)
                } else {
                    None
                };
                if let Some(inner) = inner {
                    meta(inner, &mut out);
                }
            }
            _ => {}
        }
    }
    out
}

#[cfg(test)]
#[path = "rust_attrs_tests.rs"]
mod tests;
