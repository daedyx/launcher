use crate::download::download_verified;
use crate::error::CoreError;
use crate::installer::Installer;
use crate::lock::Lock;
use crate::source::Source;
use crate::state::{InstalledApp, State};
use crate::system::SystemInfo;
use daedyx_update::catalog::{is_valid_app_id, Catalog};
use daedyx_update::error::UpdateError;
use daedyx_update::verify::verified_catalog;
use daedyx_update::version::is_newer;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Launcher {
    pub source: Source,
    pub platform: String,
    pub data_dir: PathBuf,
    pub launcher_version: String,
    pub installer: Box<dyn Installer>,
    pub system: Box<dyn SystemInfo>,
    /// Registry-Namen der Apps aus dem zuletzt geprüften Katalog (App-Kennung zu Schlüsselname).
    pub(crate) registry_keys: Mutex<BTreeMap<String, String>>,
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

impl Launcher {
    pub fn new(
        source: Source,
        platform: String,
        data_dir: PathBuf,
        launcher_version: String,
        installer: Box<dyn Installer>,
        system: Box<dyn SystemInfo>,
    ) -> Launcher {
        Launcher {
            source,
            platform,
            data_dir,
            launcher_version,
            installer,
            system,
            registry_keys: Mutex::new(BTreeMap::new()),
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

    /// Katalog und Signatur holen, die Signatur prüfen, erst dann den Katalog lesen.
    pub fn load_catalog(&self, pubkey_text: &str) -> Result<Catalog, CoreError> {
        let bytes = self.source.fetch_bytes("catalog.json")?;
        let sig = String::from_utf8(self.source.fetch_bytes("catalog.json.minisig")?)
            .map_err(|_| UpdateError::SignatureInvalid)?;
        let catalog = verified_catalog(pubkey_text, &bytes, &sig)?;
        catalog.check_launcher(&self.launcher_version)?;
        let mut keys = self.registry_keys.lock().unwrap();
        keys.clear();
        for app in &catalog.apps {
            if let Some(k) = &app.registry_key {
                keys.insert(app.id.clone(), k.clone());
            }
        }
        Ok(catalog)
    }

    pub fn status(&self, catalog: &Catalog) -> Result<Vec<AppStatus>, CoreError> {
        let mut state = State::load(&self.data_dir)?;
        let mut adopted = false;
        let mut found = Vec::new();
        let apps: Vec<&daedyx_update::catalog::App> = catalog
            .apps
            .iter()
            .filter(|a| a.id != crate::selfupdate::LAUNCHER_APP_ID)
            .collect();
        for app in &apps {
            let loc = self.locate(&state, &app.id);
            if let Some(version) = &loc.version {
                // Eine gefundene Installation (auch von Hand gemacht) wird gemerkt.
                if state.apps.get(&app.id).map(|a| &a.install_dir) != Some(&loc.dir) {
                    state.apps.insert(
                        app.id.clone(),
                        InstalledApp {
                            version: version.clone(),
                            install_dir: loc.dir.clone(),
                        },
                    );
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
        for (i, app) in apps.iter().enumerate() {
            let installed = found[i].version.clone();
            let latest = app.latest()?.map(|r| r.version.clone());
            let update_available = match (&installed, &latest) {
                (Some(i), Some(l)) => is_newer(l, i)?,
                _ => false,
            };
            out.push(AppStatus {
                id: app.id.clone(),
                name: app.name.clone(),
                running: installed.is_some() && running.get(i).copied().unwrap_or(false),
                installed,
                latest,
                update_available,
            });
        }
        Ok(out)
    }

    /// Kennungen der installierten Apps, die gerade laufen.
    pub fn running_apps(&self, catalog: &Catalog) -> Result<Vec<String>, CoreError> {
        let state = State::load(&self.data_dir)?;
        let installed: Vec<(String, PathBuf)> = catalog
            .apps
            .iter()
            .filter(|a| a.id != crate::selfupdate::LAUNCHER_APP_ID)
            .filter_map(|a| {
                let loc = self.locate(&state, &a.id);
                loc.version.map(|_| (a.id.clone(), loc.dir))
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

    /// Installiert die neueste Version oder aktualisiert auf sie. Eine laufende App wird nicht
    /// angefasst (`AppRunning`), und es wird vorher nichts geladen.
    pub fn install(
        &self,
        catalog: &Catalog,
        app_id: &str,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<Outcome, CoreError> {
        let app = catalog.app(app_id)?;
        let release = app.latest()?.ok_or_else(|| UpdateError::NoPackage {
            app: app_id.to_string(),
            platform: self.platform.clone(),
        })?;
        let package = release.package_for(app_id, &self.platform)?;

        let _lock = Lock::acquire(&self.data_dir.join("locks"), app_id)?;
        let mut state = State::load(&self.data_dir)?;
        let loc = self.locate(&state, app_id);
        let installed = loc.version.clone();
        if let Some(v) = &installed {
            if !is_newer(&release.version, v)? {
                return Ok(Outcome::UpToDate { version: v.clone() });
            }
            if self.is_running(&loc.dir) {
                return Err(CoreError::AppRunning(app_id.to_string()));
            }
        }

        let file_name = format!(
            "{}-{}.{}",
            app_id,
            release.version,
            package.kind.file_extension()
        );
        let file = download_verified(
            &self.source,
            package,
            &self.data_dir.join("downloads"),
            &file_name,
            progress,
        )?;
        let dir = loc.dir;
        let result = self.installer.install(&file, package.kind, &dir);
        let _ = std::fs::remove_file(&file);
        result?;

        state.apps.insert(
            app_id.to_string(),
            InstalledApp {
                version: release.version.clone(),
                install_dir: dir,
            },
        );
        state.save(&self.data_dir)?;
        Ok(match installed {
            Some(from) => Outcome::Updated {
                from,
                to: release.version.clone(),
            },
            None => Outcome::Installed {
                version: release.version.clone(),
            },
        })
    }

    pub fn remove(&self, app_id: &str) -> Result<(), CoreError> {
        if !is_valid_app_id(app_id) {
            return Err(UpdateError::UnknownApp(app_id.to_string()).into());
        }
        let _lock = Lock::acquire(&self.data_dir.join("locks"), app_id)?;
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
