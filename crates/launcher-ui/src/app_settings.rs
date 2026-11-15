//! Einstellungsseite im Fenster: Anzeige, Speichern, Sprach- und Themenwechsel, Katalogquelle.
use crate::app::LauncherApp;
use crate::i18n::{system_locale, Texts};
use crate::model::{Model, Notice};
use crate::settings::UiSettings;
use crate::settings_view::{self, AppInfo, ConfigEdit};
use crate::theme;
use eframe::egui;
use launcher_core::config::Config;
use launcher_core::jobs::{Job, Worker};
use std::path::PathBuf;

/// Baut aus den Einstellungen einen neuen Arbeiter (prüft sie dabei) oder meldet den Fehlertext.
pub type Rebuild = Box<dyn Fn(&Config, &Texts) -> Result<Worker, String>>;

fn non_empty(text: &str) -> Option<String> {
    let t = text.trim();
    (!t.is_empty()).then(|| t.to_string())
}

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

    /// Macht Katalogquelle und Schlüsseldatei in der Oberfläche änderbar. `config` sind die beim
    /// Start geltenden Werte (Startparameter vor gespeicherter Einstellung).
    pub fn with_rebuild(mut self, config: &Config, rebuild: Rebuild) -> LauncherApp {
        self.config_edit = ConfigEdit {
            catalog: config.catalog.clone().unwrap_or_default(),
            pubkey: config
                .pubkey
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            message: None,
        };
        self.rebuild = Some(rebuild);
        self
    }

    /// Prüft die eingegebene Quelle (über den Arbeiter-Bau), speichert sie dauerhaft und lädt neu.
    fn save_config(&mut self) {
        let config = Config {
            catalog: non_empty(&self.config_edit.catalog),
            pubkey: non_empty(&self.config_edit.pubkey).map(PathBuf::from),
        };
        let Some(rebuild) = &self.rebuild else {
            return;
        };
        let result = rebuild(&config, &self.t).and_then(|worker| {
            config
                .save(&self.info.data_dir)
                .map(|_| worker)
                .map_err(|e| crate::errors::error_message(&self.t, &e))
        });
        self.config_edit.message = Some(match result {
            Ok(worker) => {
                worker.send(Job::Refresh);
                self.worker = Some(worker);
                self.model = Model::new();
                Ok(self.t.t("settings.saved"))
            }
            Err(text) => Err(text),
        });
    }

    /// Einstellungsseite im Rahmen: Überschrift, Karten, Auswertung der Eingaben.
    pub(crate) fn settings_page(&mut self, ui: &mut egui::Ui) {
        crate::shell::page_title(ui, &self.t.t("page.settings"));
        ui.add_space(6.0);
        let out = settings_view::show(
            ui,
            &self.t,
            &self.info,
            &mut self.settings,
            &mut self.config_edit,
        );
        let ctx = ui.ctx().clone();
        if out.config_save {
            self.save_config();
        }
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
