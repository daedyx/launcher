use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Sprachen der Oberfläche. Eine neue Sprache braucht nur eine Datei `lang/<kürzel>.json`
/// und einen Eintrag hier; ein Test verlangt, dass sie alle Schlüssel hat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Language {
    #[default]
    De,
    En,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::De, Language::En];

    /// Sprache aus einer Systemkennung wie „de-DE“ oder „en_US“. Andere Sprachen fallen auf
    /// English zurück (gemeinsame Ausweichsprache); wird nichts erkannt, gilt Deutsch.
    pub fn from_locale(locale: Option<&str>) -> Language {
        let Some(locale) = locale else {
            return Language::De;
        };
        let l = locale.to_lowercase();
        if l == "de" || l.starts_with("de-") || l.starts_with("de_") {
            Language::De
        } else {
            Language::En
        }
    }

    /// Name der Sprache in ihr selbst; wird nicht übersetzt.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::De => "Deutsch",
            Language::En => "English",
        }
    }
}

#[derive(Clone)]
/// Alle Texte der Oberfläche stehen in `lang/de.json` (flache Schlüssel, `{name}`-Platzhalter).
pub struct Texts {
    map: BTreeMap<String, String>,
}

impl Texts {
    pub fn from_json(json: &str) -> Result<Texts, String> {
        let map: BTreeMap<String, String> =
            serde_json::from_str(json).map_err(|e| format!("Sprachdatei ungültig: {e}"))?;
        Ok(Texts { map })
    }

    pub fn for_language(language: Language) -> Texts {
        let json = match language {
            Language::De => include_str!("../lang/de.json"),
            Language::En => include_str!("../lang/en.json"),
        };
        Texts::from_json(json).expect("Sprachdatei ist gültig")
    }

    pub fn de() -> Texts {
        Texts::for_language(Language::De)
    }

    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &String)> {
        self.map.iter()
    }

    /// Fehlt ein Schlüssel, steht er sichtbar in `[[…]]`, damit der Fehler im Test und im Fenster auffällt.
    pub fn t(&self, key: &str) -> String {
        self.map
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("[[{key}]]"))
    }

    pub fn tf(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut text = self.t(key);
        for (name, value) in args {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
}

/// Sprache des Systems (unter Windows die Anzeigesprache), falls erkennbar.
pub fn system_locale() -> Option<String> {
    sys_locale::get_locale()
}
