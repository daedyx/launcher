use crate::config::{build_launcher, lock_dir};
use std::path::Path;

#[test]
fn the_lock_is_shared_with_the_apps_under_the_local_folder() {
    let local = Path::new("C:\\Benutzer\\Test\\AppData\\Local");
    let data = Path::new("C:\\Daten\\Launcher");
    assert_eq!(
        lock_dir(Some(local), data),
        local.join("daedyx").join("locks")
    );
    assert_eq!(lock_dir(None, data), data.join("daedyx").join("locks"));
}

#[test]
fn the_launcher_is_built_without_key_file_or_catalog_address() {
    let dir = tempfile::tempdir().unwrap();
    let l = build_launcher(dir.path().to_path_buf(), "0.2.0");
    assert_eq!(l.launcher_version, "0.2.0");
    assert!(l.lock_dir.ends_with(Path::new("daedyx").join("locks")));
    // Bauen fragt nichts an und legt nichts an.
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
}
