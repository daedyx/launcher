//! Wo eine App installiert ist und welche Version dort steht (Zustand, Registry, Programmordner).
use crate::probe::read_installed_version;
use crate::service::Launcher;
use crate::state::State;
use std::path::PathBuf;

/// Wo eine App liegt und welche Version dort steht (`None`: nicht installiert).
pub(crate) struct Located {
    pub dir: PathBuf,
    pub version: Option<String>,
}

impl Launcher {
    /// Findet die Installation: erst der gemerkte Ordner, dann der Uninstall-Eintrag der App
    /// (so wird auch eine von Hand installierte App erkannt), sonst der eigene Programmordner.
    /// Die Version steht in `app.json` (vom Setup bei jeder Installation geschrieben), sonst
    /// gilt `DisplayVersion` aus der Registry.
    pub(crate) fn locate(&self, state: &State, id: &str) -> Located {
        let remembered = state.apps.get(id).map(|a| a.install_dir.clone());
        if let Some(dir) = &remembered {
            if let Some(v) = read_installed_version(dir) {
                return Located {
                    dir: dir.clone(),
                    version: Some(v),
                };
            }
        }
        let key = crate::sync::lock_or_recover(&self.registry_keys)
            .get(id)
            .cloned();
        if let Some(entry) = key.and_then(|k| self.system.uninstall_entry(&k)) {
            if let Some(loc) = entry.install_location.filter(|l| l.is_dir()) {
                let version = read_installed_version(&loc).or(entry.display_version);
                if version.is_some() {
                    return Located { dir: loc, version };
                }
            }
        }
        let dir = remembered.unwrap_or_else(|| self.apps_dir().join(id));
        let version = read_installed_version(&dir);
        Located { dir, version }
    }
}
