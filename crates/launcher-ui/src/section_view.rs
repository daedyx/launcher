use crate::card_view::{self, Action, MIN_CARD_WIDTH};
use crate::i18n::Texts;
use crate::model::{Busy, Card};
use crate::row_view;
use eframe::egui;
use std::collections::BTreeMap;

/// Abstand zwischen den Kacheln.
const TILE_GAP: f32 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    Tiles,
    #[default]
    List,
}

/// Ein Abschnitt als Kachelraster (Spaltenzahl aus der Fensterbreite) oder als Liste.
pub fn section(
    ui: &mut egui::Ui,
    id: &str,
    view: View,
    t: &Texts,
    busy: &BTreeMap<String, Busy>,
    cards: &[&Card],
) -> Option<Action> {
    let mut chosen = None;
    match view {
        View::Tiles => {
            // Kacheln füllen die Breite gleichmäßig, mit einem kleinen Abstand dazwischen.
            let available = ui.available_width();
            let columns = (((available + TILE_GAP) / (MIN_CARD_WIDTH + TILE_GAP)) as usize).max(1);
            let width = (available - TILE_GAP * (columns as f32 - 1.0)) / columns as f32;
            egui::Grid::new(id)
                .spacing([TILE_GAP, TILE_GAP])
                .show(ui, |ui| {
                    for (i, card) in cards.iter().enumerate() {
                        if let Some(a) = card_view::show(ui, t, card, busy.get(&card.id), width) {
                            chosen = Some(a);
                        }
                        if (i + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        }
        View::List => {
            for card in cards {
                if let Some(a) = row_view::show(ui, t, card, busy.get(&card.id)) {
                    chosen = Some(a);
                }
            }
        }
    }
    chosen
}
