use crate::i18n::Texts;
use crate::model::Notice;
use crate::palette::Palette;
use eframe::egui::{self, CornerRadius, RichText};

/// Zeichnet die Hinweisleiste (grün bei Erfolg, rot bei Fehlern); gibt `true` zurück, wenn sie
/// geschlossen wurde.
pub fn show(ui: &mut egui::Ui, t: &Texts, notice: &Notice) -> bool {
    let p = Palette::current(ui);
    let (text, bg, fg) = match notice {
        Notice::Info(x) => (x.clone(), p.green_bg, p.green_text),
        Notice::Error(x) => (x.clone(), p.danger_bg, p.danger_text),
    };
    let mut close = false;
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(text).color(fg));
                if ui.button(t.t("notice.close")).clicked() {
                    close = true;
                }
            });
        });
    close
}
