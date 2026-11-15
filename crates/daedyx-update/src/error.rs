use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum UpdateError {
    #[error("Der Katalog ist ungültig: {0}")]
    CatalogInvalid(String),
    #[error("Die Katalogversion {found} wird nicht unterstützt (erwartet {expected})")]
    SchemaUnsupported { found: u32, expected: u32 },
    #[error("Der Launcher ist zu alt (nötig: {need}, vorhanden: {have})")]
    LauncherTooOld { need: String, have: String },
    #[error("Die Signatur ist ungültig")]
    SignatureInvalid,
    #[error("Der öffentliche Schlüssel ist ungültig")]
    KeyInvalid,
    #[error("Die Prüfsumme stimmt nicht überein")]
    ChecksumMismatch,
    #[error("Für {app} gibt es kein Paket für {platform}")]
    NoPackage { app: String, platform: String },
    #[error("Unbekannte App: {0}")]
    UnknownApp(String),
    #[error("Ungültige Versionsnummer: {0}")]
    BadVersion(String),
    #[error("Dateifehler: {0}")]
    Io(String),
}

impl From<std::io::Error> for UpdateError {
    fn from(e: std::io::Error) -> Self {
        UpdateError::Io(e.to_string())
    }
}
