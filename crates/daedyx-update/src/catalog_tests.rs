use crate::catalog::*;
use crate::error::UpdateError;

const SAMPLE: &str = r#"{
  "schema": 1,
  "minLauncherVersion": "0.1.0",
  "apps": [{
    "id": "desk",
    "name": "Daedyx Desk",
    "description": "Mail, Kalender, Aufgaben",
    "releases": [
      {"version": "0.9.0", "date": "2026-11-01",
       "packages": {"windows-x64": {"url": "packages/desk-0.9.0.exe", "sha256": "aa", "size": 3, "kind": "nsis-silent"}}},
      {"version": "0.10.0", "date": "2026-11-13", "notes": "neu",
       "packages": {"windows-x64": {"url": "packages/desk-0.10.0.exe", "sha256": "bb", "size": 4, "kind": "nsis-silent"}}}
    ]
  }]
}"#;

#[test]
fn katalog_wird_gelesen() {
    let c = Catalog::parse(SAMPLE.as_bytes()).unwrap();
    assert_eq!(c.apps.len(), 1);
    assert_eq!(c.app("desk").unwrap().name, "Daedyx Desk");
}

#[test]
fn neueste_version_ist_zehn_nicht_neun() {
    let c = Catalog::parse(SAMPLE.as_bytes()).unwrap();
    let latest = c.app("desk").unwrap().latest().unwrap().unwrap();
    assert_eq!(latest.version, "0.10.0");
}

#[test]
fn unbekanntes_schema_wird_abgelehnt() {
    let json = SAMPLE.replace("\"schema\": 1", "\"schema\": 2");
    assert_eq!(
        Catalog::parse(json.as_bytes()),
        Err(UpdateError::SchemaUnsupported {
            found: 2,
            expected: 1
        })
    );
}

#[test]
fn kaputtes_json_wird_abgelehnt() {
    assert!(matches!(
        Catalog::parse(b"{ nicht json"),
        Err(UpdateError::CatalogInvalid(_))
    ));
}

#[test]
fn unbekannte_app_wird_gemeldet() {
    let c = Catalog::parse(SAMPLE.as_bytes()).unwrap();
    assert_eq!(
        c.app("nix").unwrap_err(),
        UpdateError::UnknownApp("nix".into())
    );
}

#[test]
fn paket_fuer_fehlende_plattform_wird_gemeldet() {
    let c = Catalog::parse(SAMPLE.as_bytes()).unwrap();
    let r = c.app("desk").unwrap().latest().unwrap().unwrap();
    assert!(r.package_for("desk", "windows-x64").is_ok());
    assert_eq!(
        r.package_for("desk", "macos-arm64").unwrap_err(),
        UpdateError::NoPackage {
            app: "desk".into(),
            platform: "macos-arm64".into()
        }
    );
}

#[test]
fn gefaehrliche_app_kennung_wird_abgelehnt() {
    for bad in ["..", "../x", "a b", "A", "", "-x", "a/b"] {
        let json = SAMPLE.replace("\"id\": \"desk\"", &format!("\"id\": \"{bad}\""));
        assert!(
            matches!(
                Catalog::parse(json.as_bytes()),
                Err(UpdateError::CatalogInvalid(_))
            ),
            "Kennung {bad:?} hätte abgelehnt werden müssen"
        );
    }
}

#[test]
fn zu_alter_launcher_wird_erkannt() {
    let json = SAMPLE.replace(
        "\"minLauncherVersion\": \"0.1.0\"",
        "\"minLauncherVersion\": \"0.3.0\"",
    );
    let c = Catalog::parse(json.as_bytes()).unwrap();
    assert!(c.check_launcher("0.3.0").is_ok());
    assert_eq!(
        c.check_launcher("0.2.9").unwrap_err(),
        UpdateError::LauncherTooOld {
            need: "0.3.0".into(),
            have: "0.2.9".into()
        }
    );
}

#[test]
fn paketart_bestimmt_die_dateiendung() {
    assert_eq!(PackageKind::NsisSilent.file_extension(), "exe");
    assert_eq!(PackageKind::Dmg.file_extension(), "dmg");
}

#[test]
fn autor_ist_optional() {
    let c = Catalog::parse(SAMPLE.as_bytes()).unwrap();
    assert_eq!(c.app("desk").unwrap().author, "");
    let json = SAMPLE.replace(
        "\"description\": \"Mail, Kalender, Aufgaben\",",
        "\"description\": \"x\", \"author\": \"Beispiel-Studio\",",
    );
    let c = Catalog::parse(json.as_bytes()).unwrap();
    assert_eq!(c.app("desk").unwrap().author, "Beispiel-Studio");
}

#[test]
fn registry_schluessel_ist_optional() {
    let c = Catalog::parse(SAMPLE.as_bytes()).unwrap();
    assert_eq!(c.app("desk").unwrap().registry_key, None);
    let json = SAMPLE.replace(
        "\"description\": \"Mail, Kalender, Aufgaben\",",
        "\"description\": \"x\", \"registryKey\": \"Daedyx Desk\",",
    );
    let c = Catalog::parse(json.as_bytes()).unwrap();
    assert_eq!(
        c.app("desk").unwrap().registry_key.as_deref(),
        Some("Daedyx Desk")
    );
}

#[test]
fn katalog_mit_utf8_bom_wird_gelesen() {
    // Windows-Editoren schreiben gern ein BOM an den Anfang; die Signatur gilt für die Bytes samt BOM.
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice(SAMPLE.as_bytes());
    assert_eq!(Catalog::parse(&bytes).unwrap().apps.len(), 1);
}
