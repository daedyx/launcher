use super::*;

const TABLE_MD: &str = "\
| Nr. | Datei | Regel | Grund | Entscheid von | Datum | Ablauf |
|---|---|---|---|---|---|---|
| A-001 | src/a.rs | clippy::expect_used | Grund | Björn | 2026-11-23 | 2027-03-31 |
| A-002 | src/a.rs | clippy::x | Grund | Björn | 2026-11-23 | 2027-03-31 |
| A-003 | deny.toml | advisory | Grund | Björn | 2026-11-23 | 2026-01-01 |
";

fn found(pairs: &[(&str, usize)]) -> BTreeMap<String, usize> {
    pairs.iter().map(|(p, n)| (p.to_string(), *n)).collect()
}

#[test]
fn table_rows_are_parsed_without_header() {
    let e = parse_table(TABLE_MD);
    assert_eq!(e.len(), 3);
    assert_eq!(
        e.first(),
        Some(&Entry {
            path: "src/a.rs".into(),
            rule: "clippy::expect_used".into(),
            expires: "2027-03-31".into()
        })
    );
}

#[test]
fn markers_in_code_are_counted_comments_are_not() {
    let code = [
        concat!("#[", "allow(clippy::x)]"),
        "fn f() {}",
        concat!("// kein #[", "allow(...)]"),
        concat!("#[", "ignore]"),
    ];
    assert_eq!(count_rust(&code.join("\n")), 2);
    assert_eq!(count_rust(concat!("let x = unsafe", " { y };")), 1);
    assert_eq!(
        count_deny(concat!(
            "ignore = [\n  { id = \"RUST",
            "SEC-2024-0001\" },\n  # { id = \"RUST",
            "SEC-x\" }\n]"
        )),
        1
    );
}

#[test]
fn matching_counts_are_green() {
    let p = check(
        &found(&[("src/a.rs", 2)]),
        &parse_table(TABLE_MD),
        "2026-11-23",
    );
    // nur der abgelaufene deny-Eintrag und dessen fehlende Stelle
    assert_eq!(p.len(), 2, "{p:?}");
}

#[test]
fn unlisted_or_surplus_or_expired_are_red() {
    let entries = parse_table(TABLE_MD);
    let too_many = check(
        &found(&[("src/a.rs", 3), ("deny.toml", 1)]),
        &entries,
        "2026-11-23",
    );
    assert!(
        too_many.iter().any(|m| m.starts_with("src/a.rs: 3")),
        "{too_many:?}"
    );
    let unlisted = check(&found(&[("src/b.rs", 1)]), &[], "2026-11-23");
    assert_eq!(unlisted.len(), 1);
    let expired = check(&found(&[("deny.toml", 1)]), &entries[2..], "2026-11-23");
    assert!(
        expired.iter().any(|m| m.contains("abgelaufen")),
        "{expired:?}"
    );
    assert!(check(&BTreeMap::new(), &[], "2026-11-23").is_empty());
}

#[test]
fn broken_dates_are_red() {
    let bad = vec![Entry {
        path: "x.rs".into(),
        rule: "clippy::x".into(),
        expires: "bald".into(),
    }];
    assert!(check(&found(&[("x.rs", 1)]), &bad, "2026-11-23")
        .iter()
        .any(|m| m.contains("Ablaufdatum")));
}

#[test]
fn audit_ignores_must_mirror_deny_ignores() {
    let deny = concat!(
        "ignore = [\n { id = \"RUST",
        "SEC-2026-0001\", reason = \"x\" },\n]"
    );
    let same = concat!("ignore = [\"RUST", "SEC-2026-0001\"]");
    let more = concat!(
        "ignore = [\"RUST",
        "SEC-2026-0001\", \"RUST",
        "SEC-2026-0002\"]"
    );
    assert!(mirror_problems(deny, same).is_empty());
    assert_eq!(mirror_problems(deny, more).len(), 1);
    assert_eq!(mirror_problems(deny, "").len(), 1);
}

#[test]
fn scan_rows_allow_one_rule_in_one_file_until_they_expire() {
    let md = "\
| A-1 | .github/workflows/release.yml | scan:secret-im-workflow | alt | B | 2026-11-23 | 2027-01-31 |
| A-2 | docs/alt.md | scan:mailadresse | alt | B | 2026-11-23 | 2026-01-01 |
";
    let e = parse_table(md);
    assert_eq!(
        e.first().map(|x| x.rule.as_str()),
        Some("scan:secret-im-workflow")
    );
    let allowed = scan_allowances(&e, "2026-11-23");
    assert!(allowed.contains(&(
        ".github/workflows/release.yml".to_string(),
        "secret-im-workflow".to_string()
    )));
    assert_eq!(allowed.len(), 1, "abgelaufene Zeile erlaubt nichts mehr");
    // Scan-Zeilen zählen nicht als Code-Stellen; abgelaufen bleibt rot.
    let p = check(&BTreeMap::new(), &e, "2026-11-23");
    assert_eq!(p.len(), 1, "{p:?}");
}
