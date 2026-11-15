use crate::config::{
    build_launcher, effective_catalog, embedded_key_is_valid, Config, DEFAULT_CATALOG_URL,
};
use crate::error::CoreError;
use daedyx_update::error::UpdateError;
use std::path::PathBuf;

fn key_file(dir: &std::path::Path) -> PathBuf {
    let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
    let path = dir.join("catalog.pub");
    std::fs::write(&path, kp.pk.to_box().unwrap().to_string()).unwrap();
    path
}

#[test]
fn ohne_datei_ist_die_konfiguration_leer() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(Config::load(dir.path()), Config::default());
}

#[test]
fn konfiguration_wird_gespeichert_und_gelesen_ohne_nebendatei() {
    let dir = tempfile::tempdir().unwrap();
    let c = Config {
        catalog: Some("https://example.org/katalog".into()),
        pubkey: Some(PathBuf::from("C:\\Schluessel\\catalog.pub")),
    };
    c.save(dir.path()).unwrap();
    assert_eq!(Config::load(dir.path()), c);
    let names: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    assert_eq!(names, ["config.json"]);
}

#[test]
fn kaputte_datei_ergibt_leere_konfiguration() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), "kaputt").unwrap();
    assert_eq!(Config::load(dir.path()), Config::default());
}

#[test]
fn startparameter_haben_vorrang_vor_der_datei() {
    let file = Config {
        catalog: Some("aus-datei".into()),
        pubkey: Some(PathBuf::from("datei.pub")),
    };
    let merged = file.with_overrides(Some("aus-parameter".into()), None);
    assert_eq!(merged.catalog.as_deref(), Some("aus-parameter"));
    assert_eq!(merged.pubkey, Some(PathBuf::from("datei.pub")));
}

#[test]
fn gueltige_einstellung_wird_geprueft_und_ergibt_einen_launcher() {
    let dir = tempfile::tempdir().unwrap();
    let catalog = tempfile::tempdir().unwrap();
    let c = Config {
        catalog: Some(catalog.path().display().to_string()),
        pubkey: Some(key_file(dir.path())),
    };
    let (launcher, pubkey) = build_launcher(&c, dir.path().to_path_buf(), "0.1.0").unwrap();
    assert!(pubkey.contains("minisign public key"));
    assert_eq!(launcher.launcher_version, "0.1.0");
}

#[test]
fn unvollstaendige_oder_fehlerhafte_einstellungen_werden_abgelehnt() {
    let dir = tempfile::tempdir().unwrap();
    let key = key_file(dir.path());
    let build = |c: Config| build_launcher(&c, dir.path().to_path_buf(), "0.1.0").err();
    // Ordner gibt es nicht
    assert!(matches!(
        build(Config {
            catalog: Some(dir.path().join("fehlt").display().to_string()),
            pubkey: Some(key.clone())
        }),
        Some(CoreError::CatalogNotFound(_))
    ));
    // http:// nur lokal
    assert!(matches!(
        build(Config {
            catalog: Some("http://example.org/k".into()),
            pubkey: Some(key.clone())
        }),
        Some(CoreError::SourceNotAllowed(_))
    ));
    // Schlüsseldatei fehlt / ist kein Schlüssel
    assert!(matches!(
        build(Config {
            catalog: Some(dir.path().display().to_string()),
            pubkey: Some(dir.path().join("fehlt.pub"))
        }),
        Some(CoreError::Io(_))
    ));
    let junk = dir.path().join("junk.pub");
    std::fs::write(&junk, "nichts").unwrap();
    assert!(matches!(
        build(Config {
            catalog: Some(dir.path().display().to_string()),
            pubkey: Some(junk)
        }),
        Some(CoreError::Update(UpdateError::KeyInvalid))
    ));
}

#[test]
fn ohne_einstellung_gilt_die_eingebaute_katalogadresse() {
    assert_eq!(effective_catalog(&Config::default()), DEFAULT_CATALOG_URL);
    assert!(DEFAULT_CATALOG_URL.starts_with("https://raw.githubusercontent.com/daedyx/releases/"));
    let c = Config {
        catalog: Some("D:\\Katalog".into()),
        pubkey: None,
    };
    assert_eq!(effective_catalog(&c), "D:\\Katalog");
    // leere oder nur aus Leerzeichen bestehende Eingabe zählt als „nichts“
    let c = Config {
        catalog: Some("   ".into()),
        pubkey: None,
    };
    assert_eq!(effective_catalog(&c), DEFAULT_CATALOG_URL);
}

#[test]
fn eigene_schluesseldatei_mit_eingebauter_adresse_ergibt_einen_launcher() {
    let dir = tempfile::tempdir().unwrap();
    let c = Config {
        catalog: None,
        pubkey: Some(key_file(dir.path())),
    };
    let (_, pubkey) = build_launcher(&c, dir.path().to_path_buf(), "0.1.0").unwrap();
    assert!(pubkey.contains("minisign public key"));
}

#[test]
fn ohne_eingebauten_und_ohne_eigenen_schluessel_ist_die_einstellung_unvollstaendig() {
    if embedded_key_is_valid() {
        return; // echter Schlüssel eingebaut: dieser Fall tritt nicht auf
    }
    let dir = tempfile::tempdir().unwrap();
    assert!(matches!(
        build_launcher(&Config::default(), dir.path().to_path_buf(), "0.1.0").err(),
        Some(CoreError::ConfigIncomplete)
    ));
}

/// Wächter für das Setup: Die Pipeline setzt `DAEDYX_REQUIRE_REAL_KEY=1`; dann muss in
/// `assets/catalog.pub` der echte öffentliche Schlüssel stehen, kein Platzhalter.
#[test]
fn fuer_das_setup_ist_ein_echter_schluessel_eingebaut() {
    if std::env::var("DAEDYX_REQUIRE_REAL_KEY").as_deref() == Ok("1") {
        assert!(
            embedded_key_is_valid(),
            "crates/launcher-core/assets/catalog.pub ist noch ein Platzhalter"
        );
    }
}
