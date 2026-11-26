//! Zweite Prüfung über einen gesperrten Griff (Review U3 M-U3-1, wie `daedyx-sys::start_verified` in den Apps).
use crate::held::{Held, HeldError};
use sha2::{Digest, Sha256};

fn sha(b: &[u8]) -> [u8; 32] {
    Sha256::digest(b).into()
}

#[test]
fn the_unchanged_file_is_held() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("setup.exe");
    std::fs::write(&f, b"geprueft").unwrap();
    assert!(Held::open(&f, &sha(b"geprueft")).is_ok());
}

#[test]
fn a_changed_or_missing_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("setup.exe");
    std::fs::write(&f, b"getauscht").unwrap();
    assert_eq!(
        Held::open(&f, &sha(b"geprueft")).unwrap_err(),
        HeldError::Changed
    );
    assert_eq!(
        Held::open(&dir.path().join("fehlt.exe"), &sha(b"x")).unwrap_err(),
        HeldError::Missing
    );
}

#[cfg(windows)]
#[test]
fn while_held_nobody_can_write_rename_or_delete_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("setup.exe");
    std::fs::write(&f, b"geprueft").unwrap();
    let held = Held::open(&f, &sha(b"geprueft")).unwrap();
    assert!(
        std::fs::OpenOptions::new().write(true).open(&f).is_err(),
        "Schreiben gesperrt"
    );
    assert!(
        std::fs::rename(&f, dir.path().join("x.exe")).is_err(),
        "Umbenennen gesperrt"
    );
    assert!(std::fs::remove_file(&f).is_err(), "Löschen gesperrt");
    assert!(
        std::fs::read(&f).is_ok(),
        "Lesen (für den Start) bleibt erlaubt"
    );
    drop(held);
    assert!(std::fs::remove_file(&f).is_ok());
}
