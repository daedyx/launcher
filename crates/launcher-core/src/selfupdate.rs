//! Selbst-Update des Launchers (U3): Das Setup kommt aus dem eigenen Repo `daedyx/launcher` und wird wie jedes Paket
//! geprüft (GitHub-Nachweis, Eintrag, Größe, SHA-256), dann still gestartet; danach beendet der Aufrufer den Launcher,
//! und das Setup ersetzt ihn und startet ihn neu. Nur eine echt höhere Version, so bleibt eine Update-Schleife aus.
use crate::apps::{LAUNCHER, LAUNCHER_ID};
use crate::error::CoreError;
use crate::lineup::Lineup;
use crate::service::Launcher;
use serde::Serialize;

/// Kennung des Launchers in Fortschrittsmeldungen (wie früher im Katalog).
pub const LAUNCHER_APP_ID: &str = LAUNCHER_ID;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelfUpdate {
    pub version: String,
    pub notes: String,
}

impl Launcher {
    /// Gibt es eine geprüfte, höhere Version des Launchers selbst?
    pub fn self_update_available(&self, lineup: &Lineup) -> Option<SelfUpdate> {
        let latest = lineup.launcher.latest()?;
        let newer = crate::service::newer(&latest.version, &self.launcher_version);
        newer.then_some(SelfUpdate {
            version: latest.version,
            notes: latest.notes,
        })
    }

    /// Lädt das Setup der neueren Version, prüft es und startet es still (`/S /UPDATE`). Gibt `None` zurück, wenn es
    /// kein Update gibt. Das Setup bleibt liegen, bis es läuft; es wird beim nächsten Start aufgeräumt. Danach muss
    /// der Aufrufer den Launcher beenden.
    pub fn start_self_update(
        &self,
        lineup: &Lineup,
        relaunch: bool,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<Option<SelfUpdate>, CoreError> {
        let (Some(update), Some(offer)) =
            (self.self_update_available(lineup), lineup.launcher.offer())
        else {
            return Ok(None);
        };
        self.check_launcher_version(offer.entry().min_launcher_version.as_ref())?;
        let policy = LAUNCHER
            .policy()
            .ok_or_else(|| CoreError::PlatformUnsupported(LAUNCHER_ID.into()))?;
        let _lock = self.lock(&LAUNCHER)?;
        let setup = self
            .remote
            .fetch(&policy, offer, &self.downloads_dir(), &mut |d, t| {
                progress(d, Some(t))
            })
            .map_err(|blocked| CoreError::Blocked {
                app: LAUNCHER_ID.to_string(),
                blocked,
            })?;
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

    pub(crate) fn downloads_dir(&self) -> std::path::PathBuf {
        self.data_dir.join("downloads")
    }

    /// Löscht übrig gebliebene Setups und abgebrochene Downloads (beim Start aufrufen). Nur unter der Sperre des
    /// Launchers, damit kein laufender Download eines zweiten Launchers verschwindet; Verknüpfungen bleiben unberührt.
    pub fn clean_self_update_leftovers(&self) {
        if let Ok(_lock) = self.lock(&LAUNCHER) {
            let _ = daedyx_update::download::clean_leftovers(&self.downloads_dir());
        }
    }
}
