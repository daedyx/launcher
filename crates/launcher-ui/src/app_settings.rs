//! Einstellungsseite im Fenster: Anzeige, Speichern, Sprach- und Themenwechsel. Die Update-Quelle steht fest im
//! Programm (U3) und ist nicht einstellbar.
use crate::app::LauncherApp;
use crate::i18n::{system_locale, Texts};
use crate::model::Notice;
use crate::settings::UiSettings;
use crate::settings_view::{self, AppInfo};
use crate::theme;
use eframe::egui;

impl LauncherApp {
    /// Angaben für die Einstellungsseite und die geladenen Einstellungen.
    pub fn with_settings(mut self, info: AppInfo, settings: UiSettings) -> LauncherApp {
        self.info = info;
        self.settings = settings;
        self
    }

    /// Ort des Anmelde-Eintrags (nur für Tests anders als der echte).
    pub fn with_autostart(mut self, autostart: crate::autostart::Autostart) -> LauncherApp {
        self.autostart = autostart;
        self
    }

    /// Einstellungsseite im Rahmen: Überschrift, Karten, Auswertung der Eingaben.
    pub(crate) fn settings_page(&mut self, ui: &mut egui::Ui) {
        crate::shell::page_title(ui, &self.t.t("page.settings"));
        ui.add_space(6.0);
        let out = settings_view::show(ui, &self.t, &self.info, &mut self.settings);
        let ctx = ui.ctx().clone();
        if out.theme_changed {
            theme::set(&ctx, self.settings.theme);
        }
        if out.language_changed {
            self.t =
                Texts::for_language(self.settings.language.resolve(system_locale().as_deref()));
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(self.t.t("app.title")));
        }
        if out.autostart_changed {
            let exe = std::env::current_exe().unwrap_or_default();
            if let Err(detail) = self.autostart.set(self.settings.autostart, &exe) {
                self.settings.autostart = !self.settings.autostart;
                self.model.notice = Some(Notice::Error(
                    self.t.tf("settings.save_failed", &[("detail", &detail)]),
                ));
            }
        }
        if out.theme_changed || out.language_changed || out.updates_changed || out.autostart_changed
        {
            if let Err(detail) = self.settings.save(&self.info.data_dir) {
                self.model.notice = Some(Notice::Error(
                    self.t.tf("settings.save_failed", &[("detail", &detail)]),
                ));
            }
        }
        if out.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
