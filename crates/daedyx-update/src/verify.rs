use crate::catalog::Catalog;
use crate::error::UpdateError;
use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

pub fn sha256_file(path: &Path) -> Result<String, UpdateError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(buf.get(..n).unwrap_or_default());
    }
    Ok(hex::encode(hasher.finalize()))
}

/// Vergleicht die SHA-256-Prüfsumme der Datei mit dem erwarteten Wert (Groß-/Kleinschreibung egal).
pub fn verify_sha256(path: &Path, expected: &str) -> Result<(), UpdateError> {
    let actual = sha256_file(path)?;
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(UpdateError::ChecksumMismatch)
    }
}

/// Prüft eine minisign-Signatur (`pubkey_text` und `sig_text` im Dateiformat von minisign).
pub fn verify_signature(pubkey_text: &str, data: &[u8], sig_text: &str) -> Result<(), UpdateError> {
    let key = minisign_verify::PublicKey::decode(pubkey_text)
        .map_err(|_| UpdateError::SignatureInvalid)?;
    let sig =
        minisign_verify::Signature::decode(sig_text).map_err(|_| UpdateError::SignatureInvalid)?;
    key.verify(data, &sig, false)
        .map_err(|_| UpdateError::SignatureInvalid)
}

/// Erst die Signatur prüfen, dann den Katalog lesen. Ein unsignierter Katalog wird nie ausgewertet.
pub fn verified_catalog(
    pubkey_text: &str,
    catalog: &[u8],
    sig_text: &str,
) -> Result<Catalog, UpdateError> {
    verify_signature(pubkey_text, catalog, sig_text)?;
    Catalog::parse(catalog)
}

/// Prüft, ob der Text ein öffentlicher minisign-Schlüssel ist (Dateiinhalt einer `.pub`-Datei).
pub fn check_pubkey(pubkey_text: &str) -> Result<(), UpdateError> {
    minisign_verify::PublicKey::decode(pubkey_text)
        .map(|_| ())
        .map_err(|_| UpdateError::KeyInvalid)
}
