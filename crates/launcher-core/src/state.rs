use crate::error::CoreError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstalledApp {
    pub version: String,
    pub install_dir: PathBuf,
}

/// Was der Launcher über installierte Apps festgehalten hat (`installed.json`).
/// Die tatsächliche Version fragt der Launcher bei der App selbst ab (siehe `probe`).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct State {
    #[serde(default)]
    pub apps: BTreeMap<String, InstalledApp>,
}

const FILE_NAME: &str = "installed.json";

impl State {
    pub fn load(dir: &Path) -> Result<State, CoreError> {
        match std::fs::read(dir.join(FILE_NAME)) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| CoreError::Io(format!("{FILE_NAME} ist beschädigt: {e}"))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(State::default()),
            Err(e) => Err(e.into()),
        }
    }

    /// Schreibt zuerst in eine Nebendatei und benennt sie dann um, damit nie eine halbe Datei bleibt.
    pub fn save(&self, dir: &Path) -> Result<(), CoreError> {
        std::fs::create_dir_all(dir)?;
        let tmp = dir.join(format!("{FILE_NAME}.tmp"));
        let json = serde_json::to_vec_pretty(self).map_err(|e| CoreError::Io(e.to_string()))?;
        std::fs::write(&tmp, json)?;
        let target = dir.join(FILE_NAME);
        let _ = std::fs::remove_file(&target);
        std::fs::rename(&tmp, &target)?;
        Ok(())
    }
}
