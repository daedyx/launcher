use crate::card_view::Action;
use crate::filter::Sort;
use crate::i18n::Texts;
use crate::model::{Busy, Load, Model};
use crate::section_view::View;
use crate::settings::UiSettings;
use crate::settings_view::{AppInfo, ConfigEdit};
use crate::shell::{self, Page};
use eframe::egui;
use launcher_core::jobs::{Job, Worker};
use std::time::{Duration, Instant};

/// So oft wird nachgesehen, ob eine App läuft.
const RUNNING_POLL: Duration = Duration::from_secs(3);
/// So oft wird der Katalog neu geprüft, solange der Launcher offen bleibt.
const REFRESH_EVERY: Duration = Duration::from_secs(6 * 3600);

/// Ist die regelmäßige Katalogprüfung fällig? Nur wenn gerade nichts installiert wird.
pub fn refresh_due(since_last: Duration, idle: bool) -> bool {
    idle && since_last >= REFRESH_EVERY
}

pub struct LauncherApp {
    pub(crate) t: Texts,
    pub(crate) model: Model,
    pub(crate) worker: Option<Worker>,
    pub(crate) confirm_remove: Option<String>,
    /// Update einer laufenden App: Rückfrage offen für diese Kennung
    pub(crate) confirm_running: Option<String>,
    last_poll: Instant,
    last_refresh: Instant,
    /// Die Automatik hat das Selbst-Update in dieser Sitzung schon versucht
    auto_tried: bool,
    pub(crate) self_update_requested: bool,
    pub(crate) autostart: crate::autostart::Autostart,
    pub(crate) config_edit: ConfigEdit,
    pub(crate) rebuild: Option<crate::app_settings::Rebuild>,
    pub(crate) query: String,
    pub(crate) sort: Sort,
    pub(crate) view: View,
    pub(crate) info: AppInfo,
    pub(crate) settings: UiSettings,
    pub(crate) page: Page,
}

impl LauncherApp {
    /// `worker` ist `Err(Text)`, wenn der Start nicht möglich war (zum Beispiel fehlende Katalogquelle).
    pub fn new(t: Texts, worker: Result<Worker, String>) -> LauncherApp {
        let mut model = Model::new();
        let worker = match worker {
            Ok(w) => {
                w.send(Job::Refresh);
                Some(w)
            }
            Err(text) => {
                model.load = Load::Failed(text);
                None
            }
        };
        LauncherApp {
            t,
            model,
            worker,
            confirm_remove: None,
            confirm_running: None,
            last_poll: Instant::now(),
            last_refresh: Instant::now(),
            auto_tried: false,
            self_update_requested: false,
            autostart: crate::autostart::Autostart::default(),
            config_edit: ConfigEdit::default(),
            rebuild: None,
            query: String::new(),
            sort: Sort::default(),
            view: View::default(),
            info: AppInfo::default(),
            settings: UiSettings::default(),
            page: Page::default(),
        }
    }

    fn poll(&mut self) {
        if let Some(w) = &self.worker {
            while let Some(ev) = w.try_recv() {
                self.model.apply(ev, &self.t);
            }
        }
    }

    pub(crate) fn send(&self, job: Job) {
        if let Some(w) = &self.worker {
            w.send(job);
        }
    }

    pub(crate) fn dispatch(&mut self, action: Action) {
        match action {
            // Eine laufende App wird nicht aktualisiert: erst nachfragen.
            Action::Update(id) if self.model.cards.iter().any(|c| c.id == id && c.running) => {
                self.confirm_running = Some(id);
            }
            Action::Install(id) | Action::Update(id) => {
                self.model.begin(
                    &id,
                    Busy::Downloading {
                        done: 0,
                        total: None,
                    },
                );
                self.send(Job::Install(id));
            }
            Action::Open(id) => self.send(Job::Open(id)),
            Action::Remove(id) => self.confirm_remove = Some(id),
        }
    }

    pub(crate) fn refresh(&mut self) {
        self.last_refresh = Instant::now();
        self.model.refreshing();
        self.send(Job::Refresh);
    }

    /// Zeichnet das ganze Fenster in das gegebene `Ui`.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.poll();
        self.poll_running();
        self.auto_update_launcher();
        if self.model.quit_requested {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ui.ctx().request_repaint_after(RUNNING_POLL);
        let t = self.t.clone();
        let current = self.page;
        let mut page = current;
        let updates = self.model.app_updates() + usize::from(self.model.self_update.is_some());
        shell::show(ui, &t, &mut page, updates, |ui| match current {
            Page::Apps => self.apps_page(ui),
            Page::Updates => self.updates_page(ui),
            Page::Settings => self.settings_page(ui),
        });
        self.page = page;
        self.confirm_dialog(ui.ctx());
        self.running_dialog(ui.ctx());
        crate::focus::draw_ring(ui.ctx());
    }

    /// Startet das Selbst-Update, wenn nicht schon eines läuft.
    pub(crate) fn request_self_update(&mut self) {
        if self.model.self_progress.is_some() || self.model.quit_requested {
            return;
        }
        self.self_update_requested = true;
        self.model.self_progress = Some(0.0);
        self.send(Job::UpdateSelf);
    }

    /// Voreinstellung: eine neue Launcher-Version von selbst installieren, aber nie mitten in
    /// einer Installation und nur einmal je Sitzung (bei einem Fehler bleibt der Knopf).
    fn auto_update_launcher(&mut self) {
        let wanted = self.settings.auto_update_launcher
            && !self.auto_tried
            && self.model.self_update.is_some()
            && self.model.busy.is_empty();
        if wanted {
            self.auto_tried = true;
            self.request_self_update();
        }
    }

    /// Fragt regelmäßig, welche Apps laufen (nur wenn gerade nichts installiert wird).
    fn poll_running(&mut self) {
        let idle = self.model.busy.is_empty() && matches!(self.model.load, Load::Ready);
        if refresh_due(self.last_refresh.elapsed(), idle) && self.worker.is_some() {
            self.refresh();
        }
        if idle && self.worker.is_some() && self.last_poll.elapsed() >= RUNNING_POLL {
            self.last_poll = Instant::now();
            self.send(Job::CheckRunning);
        }
    }
}

impl eframe::App for LauncherApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}
