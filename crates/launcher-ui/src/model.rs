use crate::errors::error_message;
use crate::i18n::Texts;
use launcher_core::daedyx_update::reason::BlockKind;
use launcher_core::jobs::Event;
use launcher_core::lineup::Lineup;
use launcher_core::selfupdate::{SelfUpdate, LAUNCHER_APP_ID};
use launcher_core::service::{AppStatus, Outcome};
use std::collections::BTreeMap;
use std::time::Instant;

/// Lizenz aller Daedyx-Apps.
const LICENSE: &str = "MIT";

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardState {
    NotInstalled,
    Installed,
    UpdateAvailable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: Option<String>,
    pub latest: Option<String>,
    pub state: CardState,
    /// Veröffentlichungsdatum der neuesten geprüften Version (aus dem Eintrag mit GitHub-Nachweis).
    pub date: Option<String>,
    pub license: String,
    pub author: String,
    pub running: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Busy {
    Downloading { done: u64, total: Option<u64> },
    Installing,
    Removing,
}

#[derive(Debug, PartialEq)]
pub enum Load {
    Loading,
    Ready,
    Failed(String),
}

#[derive(Debug, PartialEq)]
pub enum Notice {
    Info(String),
    Error(String),
}

/// Was die Oberfläche zeigt. Kennt egui nicht und wird nur durch Ereignisse des Kerns verändert.
pub struct Model {
    pub load: Load,
    pub cards: Vec<Card>,
    pub busy: BTreeMap<String, Busy>,
    pub notice: Option<Notice>,
    /// Geprüfte neuere Version des Launchers selbst
    pub self_update: Option<SelfUpdate>,
    /// Fortschritt (0 bis 1) des Downloads des Launcher-Setups
    pub self_progress: Option<f32>,
    /// Das Setup läuft: der Launcher soll sich beenden
    pub quit_requested: bool,
    /// Letztes Suchergebnis und die Beschreibungen der Apps in der Sprache der Oberfläche.
    lineup: Option<(Lineup, Vec<(String, String)>)>,
    checked_at: Option<Instant>,
}

impl Default for Model {
    fn default() -> Self {
        Model::new()
    }
}

impl Model {
    pub fn new() -> Model {
        Model {
            load: Load::Loading,
            cards: Vec::new(),
            busy: BTreeMap::new(),
            notice: None,
            self_update: None,
            self_progress: None,
            quit_requested: false,
            lineup: None,
            checked_at: None,
        }
    }

    /// Anzahl der Apps mit verfügbarem Update.
    pub fn app_updates(&self) -> usize {
        self.cards
            .iter()
            .filter(|c| c.state == CardState::UpdateAvailable)
            .count()
    }

    /// Minuten seit der letzten Suche.
    pub fn minutes_since_check(&self) -> Option<u64> {
        self.checked_at.map(|t| t.elapsed().as_secs() / 60)
    }

    /// Vor dem Auftrag an den Arbeiter aufrufen.
    pub fn begin(&mut self, app: &str, busy: Busy) {
        self.notice = None;
        self.busy.insert(app.to_string(), busy);
    }

    pub fn refreshing(&mut self) {
        self.notice = None;
        if self.cards.is_empty() {
            self.load = Load::Loading;
        }
    }

    fn name_of(&self, id: &str) -> String {
        self.cards
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| id.to_string())
    }

    fn build_cards(&mut self, status: &[AppStatus]) {
        let Some((lineup, texts)) = &self.lineup else {
            return;
        };
        self.cards = lineup
            .apps
            .iter()
            .filter_map(|listed| {
                let app = listed.app;
                let s = status.iter().find(|s| s.id == app.id)?;
                let text = |k: &str| {
                    texts
                        .iter()
                        .find(|(key, _)| key == k)
                        .map(|(_, v)| v.clone())
                        .unwrap_or_default()
                };
                let state = match (&s.installed, s.update_available) {
                    (None, _) => CardState::NotInstalled,
                    (Some(_), true) => CardState::UpdateAvailable,
                    (Some(_), false) => CardState::Installed,
                };
                Some(Card {
                    id: app.id.to_string(),
                    name: app.name.to_string(),
                    description: text(&format!("apps.{}.description", app.id)),
                    installed: s.installed.clone(),
                    latest: s.latest.clone(),
                    state,
                    date: listed.latest().map(|l| l.date),
                    license: LICENSE.to_string(),
                    author: text("apps.author"),
                    running: s.running,
                })
            })
            .collect();
    }

    pub fn apply(&mut self, ev: Event, t: &Texts) {
        match ev {
            Event::Lineup(l) => {
                self.self_update = None;
                // Ein Grund für die ganze Suche (z. B. kein Netz): leiser Hinweis, die Apps bleiben sichtbar.
                self.notice = l.problem().map(|kind| {
                    let text = crate::errors::block_message(t, kind, "");
                    if kind == BlockKind::Offline || kind == BlockKind::SourceUnavailable {
                        Notice::Info(text)
                    } else {
                        Notice::Error(text)
                    }
                });
                let texts = l
                    .apps
                    .iter()
                    .map(|a| format!("apps.{}.description", a.app.id))
                    .chain(["apps.author".to_string()])
                    .map(|k| (k.clone(), t.t(&k)))
                    .collect();
                self.lineup = Some((l, texts));
                self.checked_at = Some(Instant::now());
                self.load = Load::Ready;
            }
            Event::Status(s) => self.build_cards(&s),
            Event::Running(ids) => {
                for card in &mut self.cards {
                    card.running = ids.contains(&card.id);
                }
            }
            Event::SelfUpdate(update) => self.self_update = update,
            Event::SelfUpdateStarted(Ok(Some(_))) => self.quit_requested = true,
            Event::SelfUpdateStarted(Ok(None)) => self.self_progress = None,
            Event::SelfUpdateStarted(Err(e)) => {
                self.self_progress = None;
                self.notice = Some(Notice::Error(error_message(t, &e)));
            }
            Event::Progress { app, done, total } if app == LAUNCHER_APP_ID => {
                self.self_progress = total
                    .filter(|tot| *tot > 0)
                    .map(|tot| (done as f32 / tot as f32).clamp(0.0, 1.0));
            }
            Event::Progress { app, done, total } => {
                let finished = total.is_some_and(|tot| tot > 0 && done >= tot);
                let busy = if finished {
                    Busy::Installing
                } else {
                    Busy::Downloading { done, total }
                };
                self.busy.insert(app, busy);
            }
            Event::Installed { app, result } => {
                self.busy.remove(&app);
                let name = self.name_of(&app);
                self.notice = Some(match result {
                    Ok(Outcome::Installed { version }) => Notice::Info(t.tf(
                        "notice.installed",
                        &[("name", &name), ("version", &version)],
                    )),
                    Ok(Outcome::Updated { to, .. }) => {
                        Notice::Info(t.tf("notice.updated", &[("name", &name), ("to", &to)]))
                    }
                    Ok(Outcome::UpToDate { .. }) => {
                        Notice::Info(t.tf("notice.uptodate", &[("name", &name)]))
                    }
                    Err(e) => Notice::Error(error_message(t, &e)),
                });
            }
            Event::Removed { app, result } => {
                self.busy.remove(&app);
                let name = self.name_of(&app);
                self.notice = Some(match result {
                    Ok(()) => Notice::Info(t.tf("notice.removed", &[("name", &name)])),
                    Err(e) => Notice::Error(error_message(t, &e)),
                });
            }
            Event::Opened { app, result } => {
                let name = self.name_of(&app);
                self.notice = Some(match result {
                    Ok(()) => Notice::Info(t.tf("notice.opened", &[("name", &name)])),
                    Err(e) => Notice::Error(error_message(t, &e)),
                });
            }
        }
    }
}
