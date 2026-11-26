//! Installieren, Aktualisieren und Entfernen einer App (aus `service.rs` gelöst, Review U3 F-U3-9). Geladen wird
//! unter der gemeinsamen Sperre der App in ihren eigenen Ordner `downloads\<Eintrags-Kennung>` (N-U3-1); vor dem
//! Setup wird die Datei über einen gesperrten Griff noch einmal geprüft und bis zum Ende festgehalten (M-U3-1).
use crate::apps::KnownApp;
use crate::error::CoreError;
use crate::held::{Held, HeldError};
use crate::lineup::Lineup;
use crate::service::{known, newer, Launcher, Outcome};
use crate::state::{InstalledApp, State};
use daedyx_update::reason::{BlockKind, Blocked};
use std::path::PathBuf;

/// Gesperrte Zweitprüfung gescheitert: getauscht heißt „nicht sicher“, sonst ein Dateifehler.
pub(crate) fn held_error(app_id: &str, e: HeldError) -> CoreError {
    let kind = if e == HeldError::Changed {
        BlockKind::NotTrusted
    } else {
        BlockKind::LocalFile
    };
    CoreError::Blocked {
        app: app_id.to_string(),
        blocked: Blocked {
            kind,
            detail: format!("held: {e:?}"),
        },
    }
}

impl Launcher {
    /// Eigener Download-Ordner einer App; aufgeräumt wird er nur unter ihrer Sperre.
    pub(crate) fn app_downloads(&self, app: &KnownApp) -> PathBuf {
        self.downloads_dir().join(app.update_app)
    }

    /// Installiert die neueste geprüfte Version oder aktualisiert auf sie. Eine laufende App wird nicht angefasst
    /// (`AppRunning`), und es wird vorher nichts geladen. Geladen und installiert wird unter der gemeinsamen Sperre.
    pub fn install(
        &self,
        lineup: &Lineup,
        app_id: &str,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<Outcome, CoreError> {
        let app = known(app_id)?;
        let listed = lineup
            .get(app_id)
            .ok_or_else(|| CoreError::UnknownApp(app_id.to_string()))?;
        let _lock = self.lock(app)?;
        let mut state = State::load(&self.data_dir)?;
        let loc = self.locate(&state, app_id);
        let installed = loc.version.clone();
        let Some(offer) = listed.offer() else {
            return match (&installed, listed.blocked()) {
                (_, Some(b)) => Err(CoreError::Blocked {
                    app: app_id.to_string(),
                    blocked: b.clone(),
                }),
                (Some(v), None) => Ok(Outcome::UpToDate { version: v.clone() }),
                (None, None) => Err(CoreError::NotOffered(app_id.to_string())),
            };
        };
        let to = offer.entry().version.to_string();
        if let Some(v) = &installed {
            if !newer(&to, v) {
                return Ok(Outcome::UpToDate { version: v.clone() });
            }
            if self.is_running(&loc.dir) {
                return Err(CoreError::AppRunning(app_id.to_string()));
            }
        }
        self.check_launcher_version(offer.entry().min_launcher_version.as_ref())?;
        let policy = app
            .policy()
            .ok_or_else(|| CoreError::PlatformUnsupported(app_id.to_string()))?;
        let file = self
            .remote
            .fetch(&policy, offer, &self.app_downloads(app), &mut |d, t| {
                progress(d, Some(t))
            })
            .map_err(|blocked| CoreError::Blocked {
                app: app_id.to_string(),
                blocked,
            })?;
        let dir = loc.dir;
        // Zweite Prüfung über einen gesperrten Griff, gehalten bis das Setup fertig ist (M-U3-1).
        let result = match Held::open(&file, &offer.entry().package.sha256) {
            Ok(held) => {
                let r = self
                    .installer
                    .install(&file, offer.entry().package.kind, &dir);
                drop(held);
                r
            }
            Err(e) => Err(held_error(app_id, e)),
        };
        let _ = std::fs::remove_file(&file);
        result?;
        state.apps.insert(
            app_id.to_string(),
            InstalledApp {
                version: to.clone(),
                install_dir: dir,
            },
        );
        state.save(&self.data_dir)?;
        Ok(match installed {
            Some(from) => Outcome::Updated { from, to },
            None => Outcome::Installed { version: to },
        })
    }

    pub fn remove(&self, app_id: &str) -> Result<(), CoreError> {
        let app = known(app_id)?;
        let _lock = self.lock(app)?;
        let mut state = State::load(&self.data_dir)?;
        let loc = self.locate(&state, app_id);
        if loc.version.is_none() && !state.apps.contains_key(app_id) {
            return Err(CoreError::NotInstalled(app_id.to_string()));
        }
        if !loc.dir.starts_with(self.apps_dir()) {
            return Err(CoreError::Io(format!(
                "Der Ordner liegt außerhalb des Programmordners: {}",
                loc.dir.display()
            )));
        }
        if self.is_running(&loc.dir) {
            return Err(CoreError::AppRunning(app_id.to_string()));
        }
        self.installer.uninstall(&loc.dir)?;
        state.apps.remove(app_id);
        state.save(&self.data_dir)
    }
}
