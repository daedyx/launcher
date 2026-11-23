use crate::i18n::{Language, Texts};

#[test]
fn texte_werden_geladen_und_platzhalter_ersetzt() {
    let t = Texts::from_json(r#"{"a":"Hallo {name}!","b":"ohne"}"#).unwrap();
    assert_eq!(t.t("b"), "ohne");
    assert_eq!(t.tf("a", &[("name", "Welt")]), "Hallo Welt!");
}

#[test]
fn fehlender_schluessel_ist_sichtbar() {
    let t = Texts::from_json("{}").unwrap();
    assert_eq!(t.t("x.y"), "[[x.y]]");
}

/// egui bricht ab, wenn die eingebaute Schrift ein Zeichen nicht kennt (zum Beispiel „→“).
/// Darum nur Buchstaben, Ziffern, ASCII-Satzzeichen und diese geprüften Zeichen erlauben.
#[test]
fn sprachdatei_enthaelt_nur_zeichen_der_eingebauten_schrift() {
    for lang in Language::ALL {
        let t = Texts::for_language(lang);
        for (key, text) in t.entries() {
            for c in text.chars() {
                let ok = c.is_alphanumeric()
                    || c.is_ascii_punctuation()
                    || c == ' '
                    || "·…„“–".contains(c);
                assert!(
                    ok,
                    "{lang:?} {key}: Zeichen {c:?} (U+{:04X}) nicht erlaubt",
                    c as u32
                );
            }
        }
    }
}

#[test]
fn kaputte_sprachdatei_wird_gemeldet() {
    assert!(Texts::from_json("nein").is_err());
}

#[test]
fn deutsche_sprachdatei_ist_gueltig_und_ihre_platzhalter_sind_ganz() {
    let t = Texts::de();
    for (key, text) in t.entries() {
        assert!(!text.trim().is_empty(), "{key} ist leer");
        let opens = text.matches('{').count();
        let closes = text.matches('}').count();
        assert_eq!(opens, closes, "{key}: Klammern passen nicht");
    }
}

/// Jeder im Code benutzte Textschlüssel muss in der Sprachdatei stehen.
#[test]
fn alle_im_code_benutzten_schluessel_existieren() {
    let t = Texts::de();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".rs") || name.ends_with("_tests.rs") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        for pat in [".t(\"", ".tf(\""] {
            for (i, _) in src.match_indices(pat) {
                let rest = src.get(i + pat.len()..).unwrap_or_default();
                let key = rest.split('"').next().unwrap_or_default();
                assert!(t.has(key), "{name}: Schlüssel {key:?} fehlt in de.json");
            }
        }
    }
}

fn placeholders(text: &str) -> Vec<String> {
    let mut found: Vec<String> = text
        .split('{')
        .skip(1)
        .filter_map(|p| p.split('}').next())
        .map(String::from)
        .collect();
    found.sort();
    found
}

/// Jede Sprache hat genau die Schlüssel der deutschen Datei, mit denselben Platzhaltern.
#[test]
fn alle_sprachdateien_haben_dieselben_schluessel_und_platzhalter() {
    let de = Texts::for_language(Language::De);
    for lang in Language::ALL {
        let other = Texts::for_language(lang);
        for (key, text) in de.entries() {
            assert!(other.has(key), "{lang:?}: Schlüssel {key:?} fehlt");
            assert_eq!(
                placeholders(text),
                placeholders(&other.t(key)),
                "{lang:?}: Platzhalter von {key:?} weichen ab"
            );
        }
        for (key, text) in other.entries() {
            assert!(de.has(key), "{lang:?}: überzähliger Schlüssel {key:?}");
            assert!(!text.trim().is_empty(), "{lang:?}: {key:?} ist leer");
        }
    }
}

#[test]
fn sprachen_haben_eigene_namen_und_sind_nicht_gleich() {
    assert_eq!(Language::De.native_name(), "Deutsch");
    assert_eq!(Language::En.native_name(), "English");
    assert_ne!(
        Texts::for_language(Language::De).t("action.install"),
        Texts::for_language(Language::En).t("action.install")
    );
}

#[test]
fn systemsprache_wird_aus_der_kennung_abgeleitet() {
    assert_eq!(Language::from_locale(Some("de-DE")), Language::De);
    assert_eq!(Language::from_locale(Some("de")), Language::De);
    assert_eq!(Language::from_locale(Some("DE_AT")), Language::De);
    assert_eq!(Language::from_locale(Some("en-US")), Language::En);
    // Andere Sprachen: English als gemeinsame Ausweichsprache; nichts erkannt: Deutsch.
    assert_eq!(Language::from_locale(Some("fr-FR")), Language::En);
    assert_eq!(Language::from_locale(Some("dv-MV")), Language::En);
    assert_eq!(Language::from_locale(None), Language::De);
}
