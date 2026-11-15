use crate::state::{InstalledApp, State};

#[test]
fn fehlende_datei_ergibt_leeren_zustand() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(State::load(dir.path()).unwrap(), State::default());
}

#[test]
fn zustand_wird_gespeichert_und_gelesen() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = State::default();
    s.apps.insert(
        "desk".into(),
        InstalledApp {
            version: "1.0.0".into(),
            install_dir: dir.path().join("apps").join("desk"),
        },
    );
    s.save(dir.path()).unwrap();
    assert_eq!(State::load(dir.path()).unwrap(), s);
    assert!(!dir.path().join("installed.json.tmp").exists());
}

#[test]
fn speichern_ueberschreibt_den_alten_stand() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = State::default();
    s.save(dir.path()).unwrap();
    s.apps.insert(
        "a".into(),
        InstalledApp {
            version: "2.0.0".into(),
            install_dir: dir.path().join("a"),
        },
    );
    s.save(dir.path()).unwrap();
    assert_eq!(State::load(dir.path()).unwrap().apps.len(), 1);
}

#[test]
fn beschaedigte_datei_wird_gemeldet() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("installed.json"), b"{ kaputt").unwrap();
    assert!(State::load(dir.path()).is_err());
}
