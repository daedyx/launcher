use crate::catalog::{Catalog, Package, PackageKind, Release};
use crate::catalog_edit::{add_release, AppMeta};
use crate::error::UpdateError;
use std::collections::BTreeMap;

fn empty() -> Catalog {
    Catalog {
        schema: 1,
        min_launcher_version: "0.1.0".into(),
        apps: vec![],
    }
}

fn release(version: &str) -> Release {
    let mut packages = BTreeMap::new();
    packages.insert(
        "windows-x64".to_string(),
        Package {
            url: format!("https://example.org/desk-{version}.exe"),
            sha256: "ab".repeat(32),
            size: 10,
            kind: PackageKind::NsisSilent,
        },
    );
    Release {
        version: version.into(),
        date: "2026-11-14".into(),
        notes: String::new(),
        packages,
    }
}

fn meta(name: &str) -> AppMeta {
    AppMeta {
        name: Some(name.into()),
        registry_key: Some("Daedyx Desk".into()),
        ..Default::default()
    }
}

#[test]
fn neue_app_wird_mit_ihren_angaben_angelegt() {
    let mut c = empty();
    add_release(&mut c, "desk", release("0.1.0"), meta("Daedyx Desk"), false).unwrap();
    let app = c.app("desk").unwrap();
    assert_eq!(app.name, "Daedyx Desk");
    assert_eq!(app.registry_key.as_deref(), Some("Daedyx Desk"));
    assert_eq!(app.releases.len(), 1);
}

#[test]
fn weitere_version_kommt_dazu_und_angaben_bleiben() {
    let mut c = empty();
    add_release(&mut c, "desk", release("0.1.0"), meta("Daedyx Desk"), false).unwrap();
    add_release(&mut c, "desk", release("0.2.0"), AppMeta::default(), false).unwrap();
    let app = c.app("desk").unwrap();
    assert_eq!(app.releases.len(), 2);
    assert_eq!(app.name, "Daedyx Desk");
    assert_eq!(app.latest().unwrap().unwrap().version, "0.2.0");
}

#[test]
fn angegebene_felder_aendern_den_eintrag_nur_dort() {
    let mut c = empty();
    add_release(&mut c, "desk", release("0.1.0"), meta("Daedyx Desk"), false).unwrap();
    let change = AppMeta {
        author: Some("Daedyx".into()),
        ..Default::default()
    };
    add_release(&mut c, "desk", release("0.2.0"), change, false).unwrap();
    let app = c.app("desk").unwrap();
    assert_eq!(app.author, "Daedyx");
    assert_eq!(app.registry_key.as_deref(), Some("Daedyx Desk"));
}

#[test]
fn doppelte_version_wird_abgelehnt_ausser_mit_ersetzen() {
    let mut c = empty();
    add_release(&mut c, "desk", release("0.1.0"), meta("Daedyx Desk"), false).unwrap();
    let err = add_release(&mut c, "desk", release("0.1.0"), AppMeta::default(), false);
    assert!(matches!(err, Err(UpdateError::CatalogInvalid(_))));
    assert_eq!(c.app("desk").unwrap().releases.len(), 1);
    let mut again = release("0.1.0");
    again.notes = "neu".into();
    add_release(&mut c, "desk", again, AppMeta::default(), true).unwrap();
    let app = c.app("desk").unwrap();
    assert_eq!(app.releases.len(), 1);
    assert_eq!(app.releases[0].notes, "neu");
}

#[test]
fn neue_app_ohne_namen_und_ungueltige_angaben_werden_abgelehnt() {
    let mut c = empty();
    assert!(matches!(
        add_release(&mut c, "desk", release("0.1.0"), AppMeta::default(), false),
        Err(UpdateError::CatalogInvalid(_))
    ));
    assert!(matches!(
        add_release(&mut c, "../x", release("0.1.0"), meta("X"), false),
        Err(UpdateError::CatalogInvalid(_))
    ));
    assert!(matches!(
        add_release(&mut c, "desk", release("keine-version"), meta("X"), false),
        Err(UpdateError::BadVersion(_))
    ));
    assert!(c.apps.is_empty());
}

#[test]
fn ergebnis_ist_ein_gueltiger_katalog() {
    let mut c = empty();
    add_release(&mut c, "desk", release("0.1.0"), meta("Daedyx Desk"), false).unwrap();
    let json = serde_json::to_vec_pretty(&c).unwrap();
    let parsed = Catalog::parse(&json).unwrap();
    assert_eq!(parsed, c);
    assert!(String::from_utf8(json).unwrap().contains("registryKey"));
}
