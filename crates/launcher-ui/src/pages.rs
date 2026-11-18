//! Die Seiten im Fenster: Apps und Updates (die Einstellungsseite steht in `app_settings.rs`).
use crate::app::LauncherApp;
use crate::chrome;
use crate::filter::filter_sort;
use crate::model::{Card, CardState, Load, Notice};
use crate::notice_view;
use crate::row_view;
use crate::section_view::section;
use crate::shell::page_title;
use eframe::egui::{self, RichText};

impl LauncherApp {
    /// Seite „Apps“: Werkzeugleiste, Hinweise und die Liste oder Kacheln der Apps.
    pub(crate) fn apps_page(&mut self, ui: &mut egui::Ui) {
        page_title(ui, &self.t.t("page.apps"));
        if chrome::toolbar(ui, &self.t, &mut self.query, &mut self.sort, &mut self.view) {
            self.refresh();
        }
        ui.add_space(4.0);
        self.update_banner(ui);
        // Fußzeile zuerst (unten), der Rest füllt den übrigen Platz.
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            chrome::footer(
                ui,
                &self.t,
                self.model.minutes_since_check(),
                self.model.app_updates(),
            );
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                self.notice(ui);
                self.body(ui);
            });
        });
    }

    /// Seite „Updates“: das Launcher-Update (falls da) und die Apps mit neuer Version.
    pub(crate) fn updates_page(&mut self, ui: &mut egui::Ui) {
        page_title(ui, &self.t.t("page.updates"));
        self.notice(ui);
        self.update_banner(ui);
        let mut chosen = None;
        let pending: Vec<&Card> = self
            .model
            .cards
            .iter()
            .filter(|c| c.state == CardState::UpdateAvailable)
            .collect();
        let failed = matches!(self.model.notice, Some(Notice::Error(_)));
        if pending.is_empty() && self.model.self_update.is_none() && !failed {
            ui.label(RichText::new(self.t.t("updates.none")).size(15.0));
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for card in pending {
                    if let Some(a) =
                        row_view::show(ui, &self.t, card, self.model.busy.get(&card.id))
                    {
                        chosen = Some(a);
                    }
                }
            });
        if let Some(a) = chosen {
            self.dispatch(a);
        }
    }

    pub(crate) fn update_banner(&mut self, ui: &mut egui::Ui) {
        let Some(update) = self.model.self_update.clone() else {
            return;
        };
        let restarting = self.model.quit_requested;
        if crate::update_banner::show(ui, &self.t, &update, self.model.self_progress, restarting) {
            self.request_self_update();
        }
        ui.add_space(4.0);
    }

    pub(crate) fn notice(&mut self, ui: &mut egui::Ui) {
        if let Some(notice) = &self.model.notice {
            if notice_view::show(ui, &self.t, notice) {
                self.model.notice = None;
            }
        }
    }

    pub(crate) fn body(&mut self, ui: &mut egui::Ui) {
        match &self.model.load {
            Load::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(self.t.t("header.loading"));
                });
            }
            Load::Failed(text) => {
                ui.label(RichText::new(text.clone()));
                if self.worker.is_some() && ui.button(self.t.t("header.retry")).clicked() {
                    self.refresh();
                }
            }
            Load::Ready if self.model.cards.is_empty() => {
                ui.label(self.t.t("list.empty"));
            }
            Load::Ready => self.grid(ui),
        }
    }

    pub(crate) fn grid(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;
        let shown = filter_sort(&self.model.cards, &self.query, self.sort);
        if shown.is_empty() {
            ui.label(self.t.t("list.no_match"));
            return;
        }
        let (installed, available): (Vec<&Card>, Vec<&Card>) = shown
            .into_iter()
            .partition(|c| c.state != CardState::NotInstalled);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                let sections = [
                    ("installed", self.t.t("section.installed"), installed),
                    ("available", self.t.t("section.available"), available),
                ];
                for (id, title, cards) in sections {
                    if cards.is_empty() {
                        continue;
                    }
                    ui.add_space(4.0);
                    ui.label(RichText::new(title.to_uppercase()).monospace().size(11.5));
                    let a = section(ui, id, self.view, &self.t, &self.model.busy, &cards);
                    if a.is_some() {
                        chosen = a;
                    }
                }
            });
        if let Some(a) = chosen {
            self.dispatch(a);
        }
    }
}
