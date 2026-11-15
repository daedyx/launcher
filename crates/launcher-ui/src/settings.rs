use crate::i18n::Language;
use serde::{Deserialize, Serialize};
use std::path::Path;

const FILE_NAME: &str = "ui-settings.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

/// Wahl der Sprache: dem System folgen (Voreinstellung) oder fest Deutsch/English.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LanguageChoice {
    #[default]
    System,
    De,
    En,
}

impl LanguageChoice {
    /// Die tatsächliche Sprache; `system` ist die Kennung des Systems (siehe `system_locale`).
    pub fn resolve(self, system: Option<&str>) -> Language {
        match self {
            LanguageChoice::System => Language::from_locale(system),
            LanguageChoice::De => Language::De,
            LanguageChoice::En => Language::En,
        }
    }
}

/// Einstellungen der Oberfläche (Datei `ui-settings.json` im Datenordner des Launchers).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiSettings {
    #[serde(default)]
    pub theme: ThemeChoice,
    #[serde(default)]
    pub language: LanguageChoice,
    /// Neue Version des Launchers von selbst installieren (Voreinstellung: ja)
    #[serde(default = "yes")]
    pub auto_update_launcher: bool,
    /// Beim Anmelden in Windows starten (Voreinstellung: nein)
    #[serde(default)]
    pub autostart: bool,
}

fn yes() -> bool {
    true
}

impl Default for UiSettings {
    fn default() -> Self {
        UiSettings {
            theme: ThemeChoice::default(),
            language: LanguageChoice::default(),
            auto_update_launcher: true,
            autostart: false,
        }
    }
}

impl UiSettings {
    /// Fehlt die Datei oder ist sie unlesbar, gelten die Voreinstellungen.
    pub fn load(dir: &Path) -> UiSettings {
        std::fs::read(dir.join(FILE_NAME))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    /// Schreibt zuerst in eine Nebendatei und benennt sie dann um.
    pub fn save(&self, dir: &Path) -> Result<(), String> {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        let tmp = dir.join(format!("{FILE_NAME}.tmp"));
        let json = serde_json::to_vec_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
        let target = dir.join(FILE_NAME);
        let _ = std::fs::remove_file(&target);
        std::fs::rename(&tmp, &target).map_err(|e| e.to_string())
    }
}
