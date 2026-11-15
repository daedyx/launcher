//! Ruhige Anzeige eines Launcher-Updates: eine schmale Zeile, kein Dialog.
use crate::i18n::Texts;
use crate::palette::Palette;
use eframe::egui::{self, RichText};
use launcher_core::selfupdate::SelfUpdate;

/// Zeichnet die Zeile; gibt `true` zurück, wenn „Jetzt aktualisieren“ gedrückt wurde.
/// `progress`: Fortschritt des Downloads, sobald er läuft.
pub fn show(
    ui: &mut egui::Ui,
    t: &Texts,
    update: &SelfUpdate,
    progress: Option<f32>,
    restarting: bool,
) -> bool {
    let mut clicked = false;
    let p = Palette::current(ui);
    egui::Frame::new()
        .fill(p.amber_bg)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let say = |text: String| RichText::new(text).size(13.5).color(p.amber_text);
                if restarting {
                    ui.label(say(t.t("banner.restart")));
                } else if let Some(fraction) = progress {
                    ui.spinner();
                    let percent = format!("{:.0}", fraction * 100.0);
                    ui.label(say(t.tf("banner.updating", &[("percent", &percent)])));
                } else {
                    ui.label(say(t.tf("banner.launcher", &[("version", &update.version)])));
                    clicked = ui.button(t.t("banner.update")).clicked();
                }
            });
        });
    clicked
}
