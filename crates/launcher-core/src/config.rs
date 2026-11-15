//! Dauerhafte Einstellungen des Launchers (`config.json` im Datenordner): Katalogquelle und
//! Schlüsseldatei. Startparameter und Umgebungsvariablen haben Vorrang vor der Datei.
use crate::error::CoreError;
use crate::installer::SystemInstaller;
use crate::service::Launcher;
use crate::source::Source;
use crate::system::RealSystem;
use daedyx_update::catalog::current_platform;
use daedyx_update::verify::check_pubkey;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const FILE_NAME: &str = "config.json";

/// Fest eingebaute Katalog-Adresse: das öffentliche Repository der Veröffentlichungen. Jede
/// Veröffentlichung hängt den vollständigen, signierten `catalog.json` an.
pub const DEFAULT_CATALOG_URL: &str = "https://raw.githubusercontent.com/daedyx/releases/main";

/// Fest eingebauter öffentlicher Katalog-Schlüssel (Datei `assets/catalog.pub`). Öffentlich, darf
/// im Repository stehen. Solange dort ein Platzhalter steht, muss der Schlüssel eingestellt werden.
const EMBEDDED_PUBKEY: &str = include_str!("../assets/catalog.pub");

pub fn embedded_key_is_valid() -> bool {
    check_pubkey(EMBEDDED_PUBKEY).is_ok()
}

/// Die geltende Katalogquelle: Einstellung, sonst die eingebaute Adresse.
pub fn effective_catalog(config: &Config) -> String {
    config
        .catalog
        .as_deref()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .unwrap_or(DEFAULT_CATALOG_URL)
        .to_string()
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Config {
    /// Ordner oder Internetadresse (`https://`, lokal auch `http://`)
    #[serde(default)]
    pub catalog: Option<String>,
    /// Öffentlicher minisign-Schlüssel des Katalogs (Datei)
    #[serde(default)]
    pub pubkey: Option<PathBuf>,
}

impl Config {
    /// Fehlt die Datei oder ist sie unlesbar, ist die Konfiguration leer.
    pub fn load(dir: &Path) -> Config {
        std::fs::read(dir.join(FILE_NAME))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    /// Schreibt zuerst in eine Nebendatei und benennt sie dann um.
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

    /// Startparameter überstimmen die Datei.
    pub fn with_overrides(mut self, catalog: Option<String>, pubkey: Option<PathBuf>) -> Config {
        if catalog.is_some() {
            self.catalog = catalog;
        }
        if pubkey.is_some() {
            self.pubkey = pubkey;
        }
        self
    }
}

/// Prüft die Einstellungen und baut den Launcher mit dem echten System. Katalogadresse und
/// Schlüssel kommen aus der Einstellung, sonst aus dem Eingebauten. Gibt zusätzlich den
/// Schlüsseltext zurück. Fehler: weder eingestellter noch eingebauter Schlüssel, Quelle nicht
/// erlaubt oder Ordner fehlt, Schlüsseldatei nicht lesbar oder kein Schlüssel.
pub fn build_launcher(
    config: &Config,
    data_dir: PathBuf,
    launcher_version: &str,
) -> Result<(Launcher, String), CoreError> {
    let catalog = effective_catalog(config);
    let source = Source::parse(&catalog)?;
    if let Source::Dir(dir) = &source {
        if !dir.is_dir() {
            return Err(CoreError::CatalogNotFound(dir.display().to_string()));
        }
    }
    let pubkey = match &config.pubkey {
        Some(path) => std::fs::read_to_string(path)?,
        None if embedded_key_is_valid() => EMBEDDED_PUBKEY.to_string(),
        None => return Err(CoreError::ConfigIncomplete),
    };
    check_pubkey(&pubkey)?;
    let launcher = Launcher::new(
        source,
        current_platform().to_string(),
        data_dir,
        launcher_version.to_string(),
        Box::new(SystemInstaller),
        Box::new(RealSystem::default()),
    );
    Ok((launcher, pubkey))
}
