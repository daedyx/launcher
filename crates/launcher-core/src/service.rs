//! Dienste des Launchers: Stand der Apps, Installieren, Aktualisieren, Entfernen. Seit U3 über die Update-Bibliothek
//! `daedyx-update` (GitHub-Nachweis statt signiertem Katalog) und mit derselben Sperre wie die Apps.
use crate::apps::{self, KnownApp};
use crate::error::CoreError;
use crate::installer::Installer;
use crate::lineup::Lineup;
use crate::remote::Remote;
use crate::state::{InstalledApp, State};
use crate::system::SystemInfo;
use daedyx_update::lock::{Lock, LockError};
use daedyx_update::version::{self, Version};
use serde::Serialize;
use std::path::PathBuf;

pub struct Launcher {
    pub remote: Remote,
    pub data_dir: PathBuf,
    /// Gemeinsamer Sperrordner mit den Apps (`<lokal>\daedyx\locks`, `daedyx_update::lock::shared_lock_dir`).
    pub lock_dir: PathBuf,
    pub launcher_version: String,
    pub installer: Box<dyn Installer>,
    pub system: Box<dyn SystemInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AppStatus {
    pub id: String,
    pub name: String,
    pub installed: Option<String>,
    pub latest: Option<String>,
    pub update_available: bool,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "result", rename_all = "kebab-case")]
pub enum Outcome {
    Installed { version: String },
    Updated { from: String, to: String },
    UpToDate { version: String },
}

/// `candidate` ist echt höher als `installed` (ungültige Angaben: nein).
pub(crate) fn newer(candidate: &str, installed: &str) -> bool {
    match (version::parse(candidate), version::parse(installed)) {
        (Ok(c), Ok(i)) => version::is_newer(&c, &i),
        _ => false,
    }
}

fn known(app_id: &str) -> Result<&'static KnownApp, CoreError> {
    apps::APPS
        .iter()
        .find(|a| a.id == app_id)
        .ok_or_else(|| CoreError::UnknownApp(app_id.to_string()))
}

impl Launcher {
    pub fn new(
        remote: Remote,
        data_dir: PathBuf,
        lock_dir: PathBuf,
        launcher_version: String,
        installer: Box<dyn Installer>,
        system: Box<dyn SystemInfo>,
    ) -> Launcher {
        Launcher {
            remote,
            data_dir,
            lock_dir,
            launcher_version,
            installer,
            system,
        }
    }

    pub(crate) fn apps_dir(&self) -> PathBuf {
        self.data_dir.join("apps")
    }

    fn is_running(&self, dir: &PathBuf) -> bool {
        self.system
            .running(std::slice::from_ref(dir))
            .first()
            .copied()
            .unwrap_or(false)
    }

    /// Die gemeinsame Sperre einer App: dieselbe, die die App bei ihrem eigenen Update holt.
    pub(crate) fn lock(&self, app: &KnownApp) -> Result<Lock, CoreError> {
        Lock::acquire(&self.lock_dir, app.update_app).map_err(|e| match e {
            LockError::Busy => CoreError::Busy(app.id.to_string()),
            other => CoreError::Io(other.to_string()),
        })
    }

    /// Verlangt ein Eintrag eine neuere Launcher-Version (`minLauncherVersion`), wird nichts geladen.
    pub(crate) fn check_launcher_version(&self, need: Option<&Version>) -> Result<(), CoreError> {
        let have = version::parse(&self.launcher_version).unwrap_or(Version::new(0, 0, 0));
        match need {
            Some(n) if have < *n => Err(CoreError::LauncherTooOld {
                need: n.to_string(),
                have: self.launcher_version.clone(),
            }),
            _ => Ok(()),
        }
    }

    pub fn status(&self, lineup: &Lineup) -> Result<Vec<AppStatus>, CoreError> {
        let mut state = State::load(&self.data_dir)?;
        let mut adopted = false;
        let mut found = Vec::new();
        for listed in &lineup.apps {
            let loc = self.locate(&state, listed.app.id);
            if let Some(version) = &loc.version {
                // Eine gefundene Installation (auch von Hand gemacht) wird gemerkt.
                if state.apps.get(listed.app.id).map(|a| &a.install_dir) != Some(&loc.dir) {
                    let app = InstalledApp {
                        version: version.clone(),
                        install_dir: loc.dir.clone(),
                    };
                    state.apps.insert(listed.app.id.to_string(), app);
                    adopted = true;
                }
            }
            found.push(loc);
        }
        if adopted {
            state.save(&self.data_dir)?;
        }
        let dirs: Vec<PathBuf> = found.iter().map(|l| l.dir.clone()).collect();
        let running = self.system.running(&dirs);
        let mut out = Vec::new();
        for (i, listed) in lineup.apps.iter().enumerate() {
            let installed = found.get(i).and_then(|l| l.version.clone());
            let latest = listed.latest_version();
            let update_available = match (&installed, &latest) {
                (Some(i), Some(l)) => newer(l, i),
                _ => false,
            };
            out.push(AppStatus {
                id: listed.app.id.to_string(),
                name: listed.app.name.to_string(),
                running: installed.is_some() && running.get(i).copied().unwrap_or(false),
                installed,
                latest,
                update_available,
            });
        }
        Ok(out)
    }

    /// Kennungen der installierten Apps, die gerade laufen.
    pub fn running_apps(&self) -> Result<Vec<String>, CoreError> {
        let state = State::load(&self.data_dir)?;
        let installed: Vec<(String, PathBuf)> = apps::APPS
            .iter()
            .filter_map(|a| {
                let loc = self.locate(&state, a.id);
                loc.version.map(|_| (a.id.to_string(), loc.dir))
            })
            .collect();
        let dirs: Vec<PathBuf> = installed.iter().map(|(_, d)| d.clone()).collect();
        let flags = self.system.running(&dirs);
        Ok(installed
            .into_iter()
            .zip(flags)
            .filter(|(_, running)| *running)
            .map(|((id, _), _)| id)
            .collect())
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
            .fetch(
                &policy,
                offer,
                &self.data_dir.join("downloads"),
                &mut |d, t| progress(d, Some(t)),
            )
            .map_err(|blocked| CoreError::Blocked {
                app: app_id.to_string(),
                blocked,
            })?;
        let dir = loc.dir;
        let result = self
            .installer
            .install(&file, offer.entry().package.kind, &dir);
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
