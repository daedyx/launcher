use crate::chips::{author_chip, chips, version_label};
use crate::i18n::Texts;
use crate::model::{Busy, Card};
use crate::palette::Palette;
use crate::widgets::{badge, busy_view_in, buttons, hover_state, track_hover, Action};
use eframe::egui::{self, RichText};

const MAIN_BUTTON_WIDTH: f32 = 120.0;
/// Hauptknopf, Abstand und „…“-Knopf.
const ACTIONS_WIDTH: f32 = MAIN_BUTTON_WIDTH + 6.0 + 40.0;

/// Zeichnet eine Zeile der Listenansicht. Meldet höchstens eine Aktion pro Bild.
pub fn show(ui: &mut egui::Ui, t: &Texts, card: &Card, busy: Option<&Busy>) -> Option<Action> {
    let mut action = None;
    let (hover_id, hovered) = hover_state(ui, card, busy);
    let width = ui.available_width();
    let p = Palette::current(ui);
    let framed = egui::Frame::new()
        .fill(p.card)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.set_width(width - 22.0);
            ui.horizontal(|ui| {
                badge(ui, card, hovered, 40.0);
                ui.vertical(|ui| {
                    ui.set_width((width * 0.38).max(200.0));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&card.name).strong().size(18.0));
                        author_chip(ui, t, card);
                    });
                    ui.add(
                        egui::Label::new(RichText::new(&card.description).size(13.0)).truncate(),
                    );
                });
                version_label(ui, t, card);
                ui.vertical(|ui| {
                    ui.add_space(14.0);
                    chips(ui, t, card);
                });
                // Knöpfe rechts: Platz davor auffüllen (so bleibt die Reihenfolge Hauptknopf, „…“).
                ui.add_space((ui.available_width() - ACTIONS_WIDTH).max(8.0));
                match busy {
                    Some(b) => {
                        busy_view_in(ui, t, b, ACTIONS_WIDTH);
                    }
                    None => action = buttons(ui, t, card, Some(MAIN_BUTTON_WIDTH)),
                }
            });
        });
    track_hover(ui, hover_id, hovered, framed.response.rect, busy);
    action
}
