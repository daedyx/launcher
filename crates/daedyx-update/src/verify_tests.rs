use crate::error::UpdateError;
use crate::verify::*;

const ABC_SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

fn sign(data: &[u8]) -> (String, String) {
    let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
    let sig = minisign::sign(
        Some(&kp.pk),
        &kp.sk,
        std::io::Cursor::new(data),
        Some("trusted"),
        Some("untrusted"),
    )
    .unwrap();
    (kp.pk.to_box().unwrap().to_string(), sig.to_string())
}

#[test]
fn pruefsumme_stimmt() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("a.bin");
    std::fs::write(&p, b"abc").unwrap();
    assert_eq!(sha256_file(&p).unwrap(), ABC_SHA256);
    assert!(verify_sha256(&p, ABC_SHA256).is_ok());
    assert!(verify_sha256(&p, &ABC_SHA256.to_uppercase()).is_ok());
}

#[test]
fn falsche_pruefsumme_wird_abgelehnt() {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("a.bin");
    std::fs::write(&p, b"abd").unwrap();
    assert_eq!(
        verify_sha256(&p, ABC_SHA256),
        Err(UpdateError::ChecksumMismatch)
    );
}

#[test]
fn gueltige_signatur_wird_akzeptiert() {
    let (pk, sig) = sign(b"daten");
    assert!(verify_signature(&pk, b"daten", &sig).is_ok());
}

#[test]
fn veraenderte_daten_werden_abgelehnt() {
    let (pk, sig) = sign(b"daten");
    assert_eq!(
        verify_signature(&pk, b"daten!", &sig),
        Err(UpdateError::SignatureInvalid)
    );
}

#[test]
fn fremder_schluessel_wird_abgelehnt() {
    let (_, sig) = sign(b"daten");
    let (other_pk, _) = sign(b"egal");
    assert_eq!(
        verify_signature(&other_pk, b"daten", &sig),
        Err(UpdateError::SignatureInvalid)
    );
}

#[test]
fn kaputte_signaturdatei_wird_abgelehnt() {
    let (pk, _) = sign(b"daten");
    assert_eq!(
        verify_signature(&pk, b"daten", "kaputt"),
        Err(UpdateError::SignatureInvalid)
    );
}

#[test]
fn unsignierter_katalog_wird_nie_gelesen() {
    let json = br#"{"schema":1,"minLauncherVersion":"0.1.0","apps":[]}"#;
    let (pk, sig) = sign(json);
    assert!(verified_catalog(&pk, json, &sig).is_ok());
    let tampered = br#"{"schema":1,"minLauncherVersion":"0.1.0","apps":[ ]}"#;
    assert_eq!(
        verified_catalog(&pk, tampered, &sig),
        Err(UpdateError::SignatureInvalid)
    );
}

#[test]
fn oeffentlicher_schluessel_wird_geprueft() {
    let (pubkey, _) = sign(b"x");
    assert!(check_pubkey(&pubkey).is_ok());
    assert_eq!(check_pubkey("kein Schlüssel"), Err(UpdateError::KeyInvalid));
    assert_eq!(check_pubkey(""), Err(UpdateError::KeyInvalid));
}
