//! Aufbau des Launchers mit dem echten System (U3). Es gibt nichts mehr einzustellen: Die Update-Quellen stehen fest
//! im Programm (`apps`), geprüft wird über den GitHub-Nachweis, ohne Schlüsseldatei und ohne Katalogadresse. Eine
//! alte `config.json` (Katalog, Schlüssel) wird nicht mehr gelesen.
use crate::installer::SystemInstaller;
use crate::remote::Remote;
use crate::service::Launcher;
use crate::system::RealSystem;
use daedyx_update::lock::shared_lock_dir;
use std::path::{Path, PathBuf};

/// Gemeinsamer Sperrordner mit den Apps: `<lokal>\daedyx\locks` (Windows `%LOCALAPPDATA%`). Ohne lokalen Ordner
/// unter dem Datenordner des Launchers (dann teilt er die Sperre nicht, sperrt aber zwei Launcher gegeneinander).
pub fn lock_dir(local: Option<&Path>, data_dir: &Path) -> PathBuf {
    shared_lock_dir(local.unwrap_or(data_dir))
}

/// Baut den Launcher mit echtem Netz, echtem Nachweis und dem echten System.
pub fn build_launcher(data_dir: PathBuf, launcher_version: &str) -> Launcher {
    let local = dirs::data_local_dir();
    let locks = lock_dir(local.as_deref(), &data_dir);
    Launcher::new(
        Remote::real(),
        data_dir,
        locks,
        launcher_version.to_string(),
        Box::new(SystemInstaller),
        Box::new(RealSystem::default()),
    )
}
