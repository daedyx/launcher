//! Schlüssel des Katalogs: erzeugen, laden, signieren. Das Passwort wird nie als Parameter
//! übergeben, sondern über eine Abfrage geholt (im Programm: verdeckte Eingabe im Terminal).
use minisign::{KeyPair, PublicKey, PublicKeyBox, SecretKey, SecretKeyBox};

const TRUSTED_COMMENT: &str = "Daedyx Katalog";

/// Erzeugt ein Schlüsselpaar und gibt (öffentlicher Text, geheimer Text) zurück.
/// Mit `password: None` bleibt der geheime Schlüssel unverschlüsselt (nur für Tests).
pub fn generate(password: Option<String>) -> Result<(String, String), String> {
    let pair = match password {
        Some(pw) => KeyPair::generate_encrypted_keypair(Some(pw)),
        None => KeyPair::generate_unencrypted_keypair(),
    }
    .map_err(|e| e.to_string())?;
    let public = pair.pk.to_box().map_err(|e| e.to_string())?.to_string();
    let secret = pair
        .sk
        .to_box(Some("Daedyx Katalog (geheim)"))
        .map_err(|e| e.to_string())?
        .to_string();
    Ok((public, secret))
}

/// Lädt den geheimen Schlüssel. Ist er verschlüsselt, wird `ask` einmal nach dem Passwort gefragt.
pub fn load_secret(
    text: &str,
    ask: impl FnOnce() -> Result<String, String>,
) -> Result<SecretKey, String> {
    let boxed = SecretKeyBox::from_string(text).map_err(|_| "Das ist keine Schlüsseldatei")?;
    if let Ok(key) = boxed.clone().into_unencrypted_secret_key() {
        return Ok(key);
    }
    let password = ask()?;
    boxed
        .into_secret_key(Some(password))
        .map_err(|_| "Das Passwort stimmt nicht".to_string())
}

pub fn load_public(text: &str) -> Result<PublicKey, String> {
    PublicKeyBox::from_string(text)
        .and_then(|b| b.into_public_key())
        .map_err(|_| "Das ist keine Datei mit einem öffentlichen Schlüssel".to_string())
}

/// Signiert die Bytes und gibt den Inhalt der `.minisig`-Datei zurück.
pub fn sign(secret: &SecretKey, public: Option<&PublicKey>, data: &[u8]) -> Result<String, String> {
    minisign::sign(
        public,
        secret,
        std::io::Cursor::new(data),
        Some(TRUSTED_COMMENT),
        Some("Signatur des Daedyx Katalogs"),
    )
    .map(|s| s.to_string())
    .map_err(|e| e.to_string())
}
