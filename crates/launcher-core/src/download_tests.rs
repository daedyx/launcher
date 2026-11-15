use crate::download::download_verified;
use crate::error::CoreError;
use crate::source::Source;
use daedyx_update::catalog::{Package, PackageKind};
use daedyx_update::error::UpdateError;
use daedyx_update::verify::sha256_file;

fn fixture(content: &[u8]) -> (tempfile::TempDir, Source, Package) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("packages")).unwrap();
    let path = dir.path().join("packages/a.exe");
    std::fs::write(&path, content).unwrap();
    let package = Package {
        url: "packages/a.exe".into(),
        sha256: sha256_file(&path).unwrap(),
        size: content.len() as u64,
        kind: PackageKind::NsisSilent,
    };
    let source = Source::Dir(dir.path().to_path_buf());
    (dir, source, package)
}

fn is_checksum_error(r: &Result<std::path::PathBuf, CoreError>) -> bool {
    matches!(r, Err(CoreError::Update(UpdateError::ChecksumMismatch)))
}

#[test]
fn gueltiges_paket_wird_freigegeben() {
    let (_keep, source, package) = fixture(b"paket");
    let out = tempfile::tempdir().unwrap();
    let file = download_verified(
        &source,
        &package,
        out.path(),
        "desk-1.0.0.exe",
        &mut |_, _| {},
    )
    .unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), b"paket");
    assert!(!out.path().join("desk-1.0.0.exe.part").exists());
}

#[test]
fn falsche_pruefsumme_hinterlaesst_keine_datei() {
    let (_keep, source, mut package) = fixture(b"paket");
    package.sha256 = "00".repeat(32);
    let out = tempfile::tempdir().unwrap();
    let r = download_verified(
        &source,
        &package,
        out.path(),
        "desk-1.0.0.exe",
        &mut |_, _| {},
    );
    assert!(is_checksum_error(&r));
    assert_eq!(std::fs::read_dir(out.path()).unwrap().count(), 0);
}

#[test]
fn falsche_groesse_hinterlaesst_keine_datei() {
    let (_keep, source, mut package) = fixture(b"paket");
    package.size += 1;
    let out = tempfile::tempdir().unwrap();
    let r = download_verified(
        &source,
        &package,
        out.path(),
        "desk-1.0.0.exe",
        &mut |_, _| {},
    );
    assert!(is_checksum_error(&r));
    assert_eq!(std::fs::read_dir(out.path()).unwrap().count(), 0);
}

#[test]
fn fehlendes_paket_hinterlaesst_keine_datei() {
    let (_keep, source, mut package) = fixture(b"paket");
    package.url = "packages/gibt-es-nicht.exe".into();
    let out = tempfile::tempdir().unwrap();
    assert!(download_verified(&source, &package, out.path(), "x.exe", &mut |_, _| {}).is_err());
    assert_eq!(std::fs::read_dir(out.path()).unwrap().count(), 0);
}
