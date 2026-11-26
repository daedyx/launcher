//! Was die Suche je App gefunden hat (ersetzt den signierten Katalog, U3): für jede fest eingetragene App
//! ([`crate::apps`]) ein geprüftes Angebot, „aktuell“ oder ein Sperrgrund. Ohne Netz bleiben die Apps sichtbar;
//! nur das Angebot fehlt. Die höchste je geprüft gesehene Version merkt sich der Launcher je App in seinem eigenen
//! Datenordner (`seen\<app>\updates.json`), nie im Ordner der App (kein zweiter Schreiber, Review U1-R H-U1-7).
use crate::apps::{KnownApp, APPS, LAUNCHER};
use crate::service::Launcher;
use daedyx_update::check::{Offer, Outcome};
use daedyx_update::reason::{BlockKind, Blocked};
use daedyx_update::state::UpdateState;
use daedyx_update::version::{self, Version};
use std::path::PathBuf;

/// Ergebnis der Suche für eine App.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Found {
    /// Geprüft und neuer als installiert (nicht installiert zählt als 0.0.0).
    Offer(Box<Offer>),
    /// Geprüft, nichts Neueres.
    UpToDate(Version),
    Blocked(Blocked),
}

/// Eine App mit dem Ergebnis ihrer Suche.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listed {
    pub app: &'static KnownApp,
    pub found: Found,
}

/// Neueste geprüfte Version mit Angaben aus dem Eintrag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Latest {
    pub version: String,
    pub date: String,
    pub notes: String,
    pub important: bool,
}

impl Listed {
    /// Das geprüfte Angebot, falls es eines gibt.
    pub fn offer(&self) -> Option<&Offer> {
        match &self.found {
            Found::Offer(o) => Some(o),
            _ => None,
        }
    }

    /// Neueste geprüfte Version (Angebot oder „aktuell“).
    pub fn latest_version(&self) -> Option<String> {
        match &self.found {
            Found::Offer(o) => Some(o.entry().version.to_string()),
            Found::UpToDate(v) => Some(v.to_string()),
            Found::Blocked(_) => None,
        }
    }

    /// Angaben des Angebots (Datum und Notiz gibt es nur mit Angebot).
    pub fn latest(&self) -> Option<Latest> {
        self.offer().map(|o| {
            let e = o.entry();
            Latest {
                version: e.version.to_string(),
                date: e.date.clone(),
                notes: e.notes.clone(),
                important: e.important,
            }
        })
    }

    pub fn blocked(&self) -> Option<&Blocked> {
        match &self.found {
            Found::Blocked(b) => Some(b),
            _ => None,
        }
    }
}

/// Ergebnis der Suche für alle Apps und den Launcher selbst.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lineup {
    pub apps: Vec<Listed>,
    pub launcher: Listed,
}

impl Lineup {
    pub fn get(&self, id: &str) -> Option<&Listed> {
        self.apps
            .iter()
            .chain(std::iter::once(&self.launcher))
            .find(|l| l.app.id == id)
    }

    /// Ein Grund, der für die ganze Suche gilt, z. B. kein Netz: der erste Sperrgrund einer App, Offline zuerst.
    pub fn problem(&self) -> Option<BlockKind> {
        let kinds: Vec<BlockKind> = self
            .apps
            .iter()
            .filter_map(|l| l.blocked().map(|b| b.kind))
            .collect();
        kinds
            .iter()
            .copied()
            .find(|k| *k == BlockKind::Offline)
            .or_else(|| kinds.first().copied())
    }
}

impl Launcher {
    fn seen_dir(&self, app: &KnownApp) -> PathBuf {
        self.data_dir.join("seen").join(app.id)
    }

    fn search(&self, app: &'static KnownApp, installed: Option<&str>) -> Listed {
        let Some(policy) = app.policy() else {
            return Listed {
                app,
                found: Found::Blocked(Blocked {
                    kind: BlockKind::NotTrusted,
                    detail: "platform".into(),
                }),
            };
        };
        let installed = installed
            .and_then(|v| version::parse(v).ok())
            .unwrap_or(Version::new(0, 0, 0));
        let mut state = UpdateState::load_or_default(&self.seen_dir(app));
        let outcome = self
            .remote
            .check(&policy, &installed, state.highest_seen().as_ref());
        if let Some(latest) = outcome.verified_latest() {
            state.note_seen(latest);
            // Merken ist eine Sicherung gegen Zurückrollen; scheitert es, gilt die Suche trotzdem.
            let _ = state.save(&self.seen_dir(app));
        }
        let found = match outcome {
            Outcome::Available(o) => Found::Offer(o),
            Outcome::UpToDate { latest } => Found::UpToDate(latest),
            Outcome::Blocked(b) => Found::Blocked(b),
        };
        Listed { app, found }
    }

    /// Sucht für alle Apps und den Launcher selbst. Installierte Versionen kommen aus Zustand, `app.json` und Registry.
    pub fn load_lineup(&self) -> Lineup {
        let state = crate::state::State::load(&self.data_dir).unwrap_or_default();
        let apps = APPS
            .iter()
            .map(|a| {
                let installed = self.locate(&state, a.id).version;
                self.search(a, installed.as_deref())
            })
            .collect();
        let launcher = self.search(&LAUNCHER, Some(&self.launcher_version));
        Lineup { apps, launcher }
    }
}
