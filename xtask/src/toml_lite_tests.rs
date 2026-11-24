use super::*;

#[test]
fn sections_keys_comments_and_multi_line_values() {
    let text = "a = 1 # Kommentar\n[sec]\n# ganz\nb = \"x # kein Kommentar\"\nc = [\n  \"p\", # eins\n  { n = \"q\" },\n]\n[\"t\".u]\nd = true\n";
    let got: Vec<(String, String)> = items(text)
        .into_iter()
        .map(|i| (i.key, compact(&i.value)))
        .collect();
    let want = [
        ("a", "1"),
        ("sec.b", "\"x#keinKommentar\""),
        ("sec.c", "[\"p\",{n=\"q\"},]"),
        ("t.u.d", "true"),
    ];
    let want: Vec<(String, String)> = want
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn values_are_split_into_strings_elements_and_fields() {
    let v = "[ \"a\", { crate = \"b\", allow = [\"x\", \"y\"] }, 'c' ]";
    assert_eq!(strings(v), vec!["a", "b", "x", "y", "c"]);
    let e = elements(v);
    assert_eq!(e.len(), 3);
    assert_eq!(
        field(&e[1], "crate").map(|f| unquote(&f)),
        Some("b".to_string())
    );
    assert_eq!(field(&e[1], "fehlt"), None);
    assert_eq!(unquote(" \"deny\" "), "deny");
    assert_eq!(unquote(" true "), "true");
}
