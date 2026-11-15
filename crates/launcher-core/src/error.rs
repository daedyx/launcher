use daedyx_update::error::UpdateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error(transparent)]
    Update(#[from] UpdateError),
    #[error("Diese Quelle ist nicht erlaubt: {0}")]
    SourceNotAllowed(String),
    #[error("Der Download ist fehlgeschlagen: {0}")]
    Download(String),
    #[error("Der Installer ist fehlgeschlagen (Code {0})")]
    InstallerFailed(i32),
    #[error("Diese Plattform wird nicht unterstützt: {0}")]
    PlatformUnsupported(String),
    #[error("Für {0} läuft gerade eine Installation oder ein Update")]
    Busy(String),
    #[error("{0} läuft gerade. Schließe die App und versuche es noch einmal")]
    AppRunning(String),
    #[error("Der Katalogordner wurde nicht gefunden: {0}")]
    CatalogNotFound(String),
    #[error("Es ist noch kein Katalog-Schlüssel eingebaut oder eingestellt (daedyx config set --pubkey <Datei>)")]
    ConfigIncomplete,
    #[error("{0} ist nicht installiert")]
    NotInstalled(String),
    #[error("Die App lässt sich nicht starten: {0}")]
    NotLaunchable(String),
    #[error("Dateifehler: {0}")]
    Io(String),
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e.to_string())
    }
}
