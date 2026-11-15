use crate::error::CoreError;
use crate::lock::Lock;
use std::time::{Duration, SystemTime};

#[test]
fn zweite_sperre_fuer_dieselbe_app_wird_abgelehnt() {
    let dir = tempfile::tempdir().unwrap();
    let _first = Lock::acquire(dir.path(), "desk").unwrap();
    assert!(matches!(
        Lock::acquire(dir.path(), "desk"),
        Err(CoreError::Busy(_))
    ));
}

#[test]
fn andere_app_ist_nicht_gesperrt() {
    let dir = tempfile::tempdir().unwrap();
    let _first = Lock::acquire(dir.path(), "desk").unwrap();
    assert!(Lock::acquire(dir.path(), "board").is_ok());
}

#[test]
fn sperre_wird_beim_loslassen_freigegeben() {
    let dir = tempfile::tempdir().unwrap();
    drop(Lock::acquire(dir.path(), "desk").unwrap());
    assert!(Lock::acquire(dir.path(), "desk").is_ok());
}

#[test]
fn uebrig_gebliebene_sperre_wird_uebernommen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("desk.lock");
    std::fs::write(&path, b"").unwrap();
    let old = SystemTime::now() - Duration::from_secs(2 * 60 * 60);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&path)
        .unwrap()
        .set_modified(old)
        .unwrap();
    assert!(Lock::acquire(dir.path(), "desk").is_ok());
}
