//! Rückfragen im Fenster: App entfernen, Update einer laufenden App.
use crate::app::LauncherApp;
use crate::card_view::Action;
use crate::model::Busy;
use eframe::egui::{self, RichText};
use launcher_core::jobs::Job;

impl LauncherApp {
    pub(crate) fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(id) = self.confirm_remove.clone() else {
            return;
        };
        let name = self
            .model
            .cards
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.name.clone())
            .unwrap_or(id.clone());
        let mut done = false;
        egui::Window::new(self.t.tf("confirm.remove.title", &[("name", &name)]))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(self.t.t("confirm.remove.text"));
                ui.horizontal(|ui| {
                    if ui
                        .button(RichText::new(self.t.t("confirm.remove.yes")).strong())
                        .clicked()
                    {
                        self.model.begin(&id, Busy::Removing);
                        self.send(Job::Remove(id.clone()));
                        done = true;
                    }
                    if ui.button(self.t.t("confirm.cancel")).clicked() {
                        done = true;
                    }
                });
            });
        if done {
            self.confirm_remove = None;
        }
    }

    /// Rückfrage, wenn ein Update für eine laufende App angefordert wurde. Der Knopf geht erst
    /// auf, wenn die App geschlossen ist (die Abfrage „läuft?“ aktualisiert sich von selbst).
    pub(crate) fn running_dialog(&mut self, ctx: &egui::Context) {
        let Some(id) = self.confirm_running.clone() else {
            return;
        };
        let card = self.model.cards.iter().find(|c| c.id == id);
        let name = card.map(|c| c.name.clone()).unwrap_or_else(|| id.clone());
        let running = card.is_some_and(|c| c.running);
        let (mut go, mut cancel) = (false, false);
        egui::Window::new(self.t.tf("confirm.running.title", &[("name", &name)]))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(self.t.tf("confirm.running.text", &[("name", &name)]));
                ui.horizontal(|ui| {
                    let label = RichText::new(self.t.t("confirm.running.go")).strong();
                    go = ui.add_enabled(!running, egui::Button::new(label)).clicked();
                    cancel = ui.button(self.t.t("confirm.cancel")).clicked();
                });
            });
        if go || cancel {
            self.confirm_running = None;
        }
        if go {
            self.dispatch(Action::Update(id));
        }
    }
}
