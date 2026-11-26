//! Nur mit dem Merkmal `probe` übersetzt (nie im Setup aus `scripts/build-setup.ps1` ohne `-Probe`): Installationsprobe
//! des Launchers (Review U3 F-U3-2). Ein **lokales** Setup wird ohne GitHub-Nachweis wie ein geprüftes behandelt: in den
//! eigenen Download-Ordner kopiert, über einen gesperrten Griff gehasht und bis zum Ende festgehalten, dann mit den echten
//! Schaltern gestartet. So prüft die Probe den NSIS-Weg (`/S /D=`, `uninstall.exe`, `/S /UPDATE`) ohne Netz. Die
//! Prüfung selbst testen der Kern und `service_tests`.
use crate::apps::{KnownApp, LAUNCHER};
use crate::error::CoreError;
use crate::held::Held;
use crate::install::held_error;
use crate::service::{known, Launcher, Outcome};
use crate::state::{InstalledApp, State};
use daedyx_update::entry::PackageKind;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn copy_in(setup: &Path, dir: &Path, name: &str) -> Result<(PathBuf, [u8; 32]), CoreError> {
    std::fs::create_dir_all(dir)?;
    let target = dir.join(name);
    std::fs::copy(setup, &target)?;
    let sha: [u8; 32] = Sha256::digest(std::fs::read(&target)?).into();
    Ok((target, sha))
}

impl Launcher {
    /// Installiert `setup` als App `app_id` (ohne Nachweis, nur Probe).
    pub fn install_local(&self, app_id: &str, setup: &Path) -> Result<Outcome, CoreError> {
        let app: &KnownApp = known(app_id)?;
        let _lock = self.lock(app)?;
        let mut state = State::load(&self.data_dir)?;
        let loc = self.locate(&state, app_id);
        let name = format!("{}-probe-setup.exe", app.update_app);
        let (file, sha) = copy_in(setup, &self.app_downloads(app), &name)?;
        let result = match Held::open(&file, &sha) {
            Ok(held) => {
                let r = self
                    .installer
                    .install(&file, PackageKind::NsisSilent, &loc.dir);
                drop(held);
                r
            }
            Err(e) => Err(held_error(app_id, e)),
        };
        let _ = std::fs::remove_file(&file);
        result?;
        let version = self
            .locate(&state, app_id)
            .version
            .ok_or_else(|| CoreError::NotInstalled(app_id.to_string()))?;
        let app_entry = InstalledApp {
            version: version.clone(),
            install_dir: loc.dir,
        };
        state.apps.insert(app_id.to_string(), app_entry);
        state.save(&self.data_dir)?;
        Ok(match loc.version {
            Some(from) => Outcome::Updated { from, to: version },
            None => Outcome::Installed { version },
        })
    }

    /// Startet `setup` als Selbst-Update (`/S /UPDATE /NOLAUNCH`, ohne Nachweis, nur Probe).
    pub fn self_update_local(&self, setup: &Path) -> Result<(), CoreError> {
        let _lock = self.lock(&LAUNCHER)?;
        let (file, sha) = copy_in(
            setup,
            &self.app_downloads(&LAUNCHER),
            "daedyx-launcher-probe-setup.exe",
        )?;
        let held = Held::open(&file, &sha).map_err(|e| held_error(LAUNCHER.id, e))?;
        let started = self
            .system
            .start_detached(&file, &["/S", "/UPDATE", "/NOLAUNCH"]);
        drop(held);
        started.map_err(CoreError::Io)
    }
}
