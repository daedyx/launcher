use daedyx_update::reason::Blocked;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    /// Suche oder Download ohne Ergebnis: Netz, Nachweis, Eintrag oder Datei (Grund für den Nutzertext in `kind`).
    #[error("{app}: {blocked}")]
    Blocked { app: String, blocked: Blocked },
    /// Für diese App gibt es (noch) kein geprüftes Angebot.
    #[error("Für {0} gibt es keine geprüfte Version")]
    NotOffered(String),
    #[error("Dieser Launcher ist zu alt (nötig: {need}, vorhanden: {have})")]
    LauncherTooOld { need: String, have: String },
    #[error("Die App {0} kennt dieser Launcher nicht")]
    UnknownApp(String),
    #[error("Der Installer ist fehlgeschlagen (Code {0})")]
    InstallerFailed(i32),
    #[error("Diese Plattform wird nicht unterstützt: {0}")]
    PlatformUnsupported(String),
    #[error("Für {0} läuft gerade eine Installation oder ein Update")]
    Busy(String),
    #[error("{0} läuft gerade. Schließe die App und versuche es noch einmal")]
    AppRunning(String),
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
