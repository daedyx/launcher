use super::*;

fn s(src: &str) -> Vec<String> {
    sites(src)
}

#[test]
fn plain_and_spaced_attributes_are_found() {
    assert_eq!(
        s("#[allow(clippy::unwrap_used)]\nfn f() {}"),
        vec!["clippy::unwrap_used"]
    );
    assert_eq!(
        s("# [ allow ( clippy :: unwrap_used ) ]\nfn f() {}"),
        vec!["clippy::unwrap_used"]
    );
    assert_eq!(
        s("#! [expect(clippy::panic, reason = \"x\")]"),
        vec!["clippy::panic"]
    );
}

#[test]
fn cfg_attr_hides_nothing() {
    assert_eq!(
        s("#[cfg_attr(all(), allow(clippy::unwrap_used))]\nfn f() {}"),
        vec!["clippy::unwrap_used"]
    );
    let nested = "#[cfg_attr(test, cfg_attr(all(), expect(clippy::panic, reason = \"x\")), allow(dead_code))]";
    assert_eq!(s(nested), vec!["clippy::panic", "dead_code"]);
    assert_eq!(
        s("#[cfg_attr(windows, ignore)]\n#[test]\nfn t() {}"),
        vec!["ignore"]
    );
}

#[test]
fn multi_line_attributes_list_all_rules_sorted() {
    let src = "#![\n  allow(\n    clippy::print_stdout,\n    clippy::exit,\n    reason = \"A-001\"\n  )\n]\n";
    assert_eq!(s(src), vec!["clippy::exit,clippy::print_stdout"]);
}

#[test]
fn comments_and_text_do_not_count() {
    let src = r##"
// #[allow(clippy::a)]
/* #[allow(clippy::b)] /* verschachtelt */ #[allow(clippy::c)] */
/// Beispiel: #[allow(clippy::d)]
let a = "#[allow(clippy::e)] \" #[allow(clippy::f)]";
let b = r#"#[allow(clippy::g)] " unsafe { }"#;
let c = '"';
let d = b'#';
fn g<'a>(x: &'a str) -> &'a str { x }
#[allow(clippy::echt)]
fn h() {}
"##;
    assert_eq!(s(src), vec!["clippy::echt"]);
}

#[test]
fn ignore_and_unsafe_are_found_in_every_spelling() {
    assert_eq!(
        s("#[ignore]\n#[ignore = \"langsam\"]\n#[ ignore ]"),
        vec!["ignore", "ignore", "ignore"]
    );
    let src =
        "unsafe {}\nunsafe\n{}\nunsafe fn a() {}\nunsafe impl X for Y {}\nunsafe extern \"C\" {}\n";
    assert_eq!(s(src).len(), 5);
    assert!(s("#![forbid(unsafe_code)]\nlet unsafe_x = 1;").is_empty());
}

#[test]
fn other_attributes_are_no_exceptions() {
    assert!(
        s("#[derive(Debug)]\n#[cfg(test)]\n#[test]\n#[deny(clippy::x)]\n#[warn(clippy::y)]")
            .is_empty()
    );
    assert!(s("macro_rules! m { ($(#[$a:meta])*) => {} }").is_empty());
}
