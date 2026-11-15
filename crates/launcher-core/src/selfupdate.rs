//! Selbst-Update des Launchers: Er steht als App `launcher` im Katalog. Das Setup wird wie jedes
//! Paket geprüft (Katalog-Signatur, Größe, SHA-256), dann still gestartet; danach beendet der
//! Aufrufer den Launcher, und das Setup ersetzt ihn und startet ihn neu.
use crate::download::download_verified;
use crate::error::CoreError;
use crate::lock::Lock;
use crate::service::Launcher;
use daedyx_update::catalog::Catalog;
use daedyx_update::error::UpdateError;
use daedyx_update::version::is_newer;
use serde::Serialize;

/// Kennung des Launchers im Katalog.
pub const LAUNCHER_APP_ID: &str = "launcher";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelfUpdate {
    pub version: String,
    pub notes: String,
}

impl Launcher {
    /// Gibt es im Katalog eine neuere Version des Launchers selbst?
    pub fn self_update_available(
        &self,
        catalog: &Catalog,
    ) -> Result<Option<SelfUpdate>, CoreError> {
        let Ok(app) = catalog.app(LAUNCHER_APP_ID) else {
            return Ok(None);
        };
        let Some(release) = app.latest()? else {
            return Ok(None);
        };
        // Nur anbieten, wenn es ein Paket für diese Plattform gibt.
        if release
            .package_for(LAUNCHER_APP_ID, &self.platform)
            .is_err()
        {
            return Ok(None);
        }
        if !is_newer(&release.version, &self.launcher_version)? {
            return Ok(None);
        }
        Ok(Some(SelfUpdate {
            version: release.version.clone(),
            notes: release.notes.clone(),
        }))
    }

    /// Lädt das Setup der neueren Version, prüft es und startet es still (`/S /UPDATE`). Gibt
    /// `None` zurück, wenn es kein Update gibt. Das Setup bleibt liegen, bis es läuft; es wird
    /// beim nächsten Start aufgeräumt. Danach muss der Aufrufer den Launcher beenden.
    pub fn start_self_update(
        &self,
        catalog: &Catalog,
        relaunch: bool,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<Option<SelfUpdate>, CoreError> {
        let Some(update) = self.self_update_available(catalog)? else {
            return Ok(None);
        };
        let app = catalog.app(LAUNCHER_APP_ID)?;
        let release = app.latest()?.ok_or_else(|| UpdateError::NoPackage {
            app: LAUNCHER_APP_ID.to_string(),
            platform: self.platform.clone(),
        })?;
        let package = release.package_for(LAUNCHER_APP_ID, &self.platform)?;
        let _lock = Lock::acquire(&self.data_dir.join("locks"), LAUNCHER_APP_ID)?;
        let file_name = format!(
            "{}-{}.{}",
            LAUNCHER_APP_ID,
            release.version,
            package.kind.file_extension()
        );
        let setup = download_verified(
            &self.source,
            package,
            &self.data_dir.join("downloads"),
            &file_name,
            progress,
        )?;
        // `/NOLAUNCH`: das Setup startet den Launcher nach dem Update nicht neu (Kommandozeile, Tests).
        let args: &[&str] = if relaunch {
            &["/S", "/UPDATE"]
        } else {
            &["/S", "/UPDATE", "/NOLAUNCH"]
        };
        if let Err(e) = self.system.start_detached(&setup, args) {
            let _ = std::fs::remove_file(&setup);
            return Err(CoreError::Io(e));
        }
        Ok(Some(update))
    }

    /// Löscht übrig gebliebene Setup-Dateien früherer Selbst-Updates (beim Start aufrufen).
    pub fn clean_self_update_leftovers(&self) {
        let Ok(entries) = std::fs::read_dir(self.data_dir.join("downloads")) else {
            return;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&format!("{LAUNCHER_APP_ID}-")) {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
}
