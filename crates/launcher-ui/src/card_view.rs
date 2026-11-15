use crate::chips::{author_chip, chips, version_label};
use crate::i18n::Texts;
use crate::model::{Busy, Card};
use crate::palette::Palette;
use crate::widgets::{
    badge, busy_view, buttons, format_date, hover_state, small_mono, track_hover,
};
use eframe::egui::{self, RichText};

pub use crate::widgets::Action;

/// Schmalste Breite einer Kachel; das Raster verteilt die Fensterbreite gleichmäßig.
pub const MIN_CARD_WIDTH: f32 = 300.0;
const MARGIN: f32 = 14.0;
const CARD_MIN_HEIGHT: f32 = 170.0;

/// Zeichnet eine Kachel der Gesamtbreite `outer_width` (inklusive Rand). Meldet höchstens
/// eine Aktion pro Bild.
pub fn show(
    ui: &mut egui::Ui,
    t: &Texts,
    card: &Card,
    busy: Option<&Busy>,
    outer_width: f32,
) -> Option<Action> {
    let mut action = None;
    let (hover_id, hovered) = hover_state(ui, card, busy);
    let p = Palette::current(ui);
    let framed = egui::Frame::new()
        .fill(p.card)
        .stroke(egui::Stroke::new(1.0, p.border))
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(MARGIN)
        .show(ui, |ui| {
            // Der Rahmen erbt das Layout des Rasters; die Kachel selbst ist eine Spalte.
            ui.vertical(|ui| {
                ui.set_width(outer_width - 2.0 * MARGIN - 2.0);
                ui.set_min_height(CARD_MIN_HEIGHT);
                ui.horizontal(|ui| {
                    version_label(ui, t, card);
                    ui.label(small_mono(card.license.clone()).weak());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        author_chip(ui, t, card);
                    });
                });
                ui.separator();
                ui.horizontal(|ui| {
                    badge(ui, card, hovered, 52.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&card.name).strong().size(20.0));
                        ui.add(
                            egui::Label::new(RichText::new(&card.description).size(13.5)).wrap(),
                        );
                    });
                });
                ui.add_space(4.0);
                chips(ui, t, card);
                if let Some(date) = &card.date {
                    ui.label(
                        RichText::new(t.tf("card.updated", &[("date", &format_date(date))]))
                            .size(12.0)
                            .weak(),
                    );
                }
                ui.add_space(6.0);
                match busy {
                    Some(b) => busy_view(ui, t, b),
                    None => action = buttons(ui, t, card, None),
                }
            });
        });
    track_hover(ui, hover_id, hovered, framed.response.rect, busy);
    action
}
