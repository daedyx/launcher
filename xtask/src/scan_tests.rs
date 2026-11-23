use super::*;

fn rules(path: &str, content: &str) -> Vec<(usize, &'static str)> {
    scan_file(path, content.as_bytes(), &[])
        .into_iter()
        .map(|f| (f.line, f.rule))
        .collect()
}

#[test]
fn clean_code_has_no_findings() {
    assert!(rules("src/lib.rs", "fn main() {}\n// Test mit anna@example.com\n").is_empty());
}

#[test]
fn findings_name_line_and_rule_but_not_the_text() {
    let content = concat!("ok\n", "mail = \"anna", "@", "beispiel.de\"\n");
    let f = scan_file("a.rs", content.as_bytes(), &[]);
    assert_eq!(
        f,
        vec![Finding {
            path: "a.rs".into(),
            line: 2,
            rule: "mailadresse"
        }]
    );
}

#[test]
fn forbidden_files_are_found_except_invented_fixtures() {
    assert_eq!(
        rules("daten/postfach.mbox", ""),
        vec![(0, "verbotener-dateityp")]
    );
    assert_eq!(
        rules("export/KONTAKTE.VCF", ""),
        vec![(0, "verbotener-dateityp")]
    );
    assert!(rules("crates/x/testdata/erfunden/termin.ics", "").is_empty());
    assert_eq!(rules("CLAUDE.md", ""), vec![(0, "claude-md-im-git")]);
    assert_eq!(rules("app/.env.local", ""), vec![(0, "env-datei")]);
}

#[test]
fn large_and_binary_files() {
    let big = vec![b'a'; MAX_FILE_BYTES + 1];
    assert_eq!(scan_file("gross.txt", &big, &[]).len(), 1);
    let binary = [0u8, 1, 2, b'@', b'x'];
    assert!(scan_file("bild.png", &binary, &[]).is_empty());
}

#[test]
fn workflows_may_only_use_the_github_token() {
    let ok = "token: ${{ secrets.GITHUB_TOKEN }}";
    let bad = concat!("key: ${{ secrets.", "SIGNING_KEY }}");
    assert!(rules(".github/workflows/ci.yml", ok).is_empty());
    assert_eq!(
        rules(".github/workflows/ci.yml", bad),
        vec![(1, "secret-im-workflow")]
    );
    assert!(rules("docs/x.md", bad).is_empty());
}

#[test]
fn local_words_are_checked() {
    let words = vec!["musterstadt".to_string()];
    let f = scan_file("README.md", "Wir in Musterstadt".as_bytes(), &words);
    assert_eq!(f.len(), 1);
    assert_eq!(f.first().map(|x| x.rule), Some("lokale-wortliste"));
}
