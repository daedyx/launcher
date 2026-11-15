use crate::i18n::Language;
use crate::settings::{LanguageChoice, ThemeChoice, UiSettings};

#[test]
fn ohne_datei_gelten_die_voreinstellungen() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(UiSettings::load(dir.path()), UiSettings::default());
    assert_eq!(UiSettings::default().theme, ThemeChoice::System);
    assert_eq!(UiSettings::default().language, LanguageChoice::System);
}

#[test]
fn einstellungen_werden_gespeichert_und_wieder_gelesen() {
    let dir = tempfile::tempdir().unwrap();
    let s = UiSettings {
        theme: ThemeChoice::Dark,
        language: LanguageChoice::En,
        ..UiSettings::default()
    };
    s.save(dir.path()).unwrap();
    assert_eq!(UiSettings::load(dir.path()), s);
}

#[test]
fn kaputte_oder_unbekannte_datei_faellt_auf_voreinstellung_zurueck() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("ui-settings.json"), "kaputt").unwrap();
    assert_eq!(UiSettings::load(dir.path()), UiSettings::default());
    std::fs::write(dir.path().join("ui-settings.json"), r#"{"theme":"bunt"}"#).unwrap();
    assert_eq!(UiSettings::load(dir.path()), UiSettings::default());
}

#[test]
fn speichern_hinterlaesst_keine_nebendatei() {
    let dir = tempfile::tempdir().unwrap();
    UiSettings::default().save(dir.path()).unwrap();
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(names, ["ui-settings.json"]);
}

#[test]
fn sprachwahl_system_folgt_dem_system_sonst_gilt_die_wahl() {
    assert_eq!(LanguageChoice::System.resolve(Some("en-GB")), Language::En);
    assert_eq!(LanguageChoice::System.resolve(Some("de-DE")), Language::De);
    assert_eq!(LanguageChoice::De.resolve(Some("en-GB")), Language::De);
    assert_eq!(LanguageChoice::En.resolve(Some("de-DE")), Language::En);
}

#[test]
fn alte_einstellungsdatei_mit_fester_sprache_bleibt_gueltig() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("ui-settings.json"),
        r#"{"theme":"dark","language":"en"}"#,
    )
    .unwrap();
    let s = UiSettings::load(dir.path());
    assert_eq!(s.theme, ThemeChoice::Dark);
    assert_eq!(s.language, LanguageChoice::En);
}

#[test]
fn neue_felder_haben_sinnvolle_voreinstellungen_auch_bei_alten_dateien() {
    let d = UiSettings::default();
    assert!(d.auto_update_launcher);
    assert!(!d.autostart);
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("ui-settings.json"), r#"{"theme":"dark"}"#).unwrap();
    let s = UiSettings::load(dir.path());
    assert_eq!(s.theme, ThemeChoice::Dark);
    assert!(s.auto_update_launcher);
    assert!(!s.autostart);
    let off = UiSettings {
        auto_update_launcher: false,
        autostart: true,
        ..UiSettings::default()
    };
    off.save(dir.path()).unwrap();
    assert_eq!(UiSettings::load(dir.path()), off);
}
