use crate::card_view::{self, Action, MIN_CARD_WIDTH};
use crate::i18n::Texts;
use crate::model::{Busy, Card};
use crate::row_view;
use eframe::egui;
use std::collections::BTreeMap;

/// Abstand zwischen den Kacheln.
const TILE_GAP: f32 = 12.0;
/// Rand rechts, damit die (schwebende) Bildlaufleiste keine Kachel anschneidet.
const SCROLLBAR_GUTTER: f32 = 10.0;

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
            let available = ui.available_width() - SCROLLBAR_GUTTER;
            let columns = (((available + TILE_GAP) / (MIN_CARD_WIDTH + TILE_GAP)) as usize).max(1);
            let width = (available - TILE_GAP * (columns as f32 - 1.0)) / columns as f32;
            // Höhen der Reihen aus dem letzten Bild: so sind alle Kacheln einer Reihe gleich hoch.
            let mem = ui.make_persistent_id(("tile_row_heights", id));
            let last: Vec<f32> = ui.data(|d| d.get_temp(mem)).unwrap_or_default();
            let mut heights = vec![0.0_f32; cards.len().div_ceil(columns)];
            egui::Grid::new(id)
                .spacing([TILE_GAP, TILE_GAP])
                .show(ui, |ui| {
                    for (i, card) in cards.iter().enumerate() {
                        let row = i / columns;
                        let min_h = last.get(row).copied().unwrap_or(0.0);
                        let (a, h) =
                            card_view::show_sized(ui, t, card, busy.get(&card.id), width, min_h);
                        heights[row] = heights[row].max(h);
                        if a.is_some() {
                            chosen = a;
                        }
                        if (i + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
            if heights != last {
                ui.data_mut(|d| d.insert_temp(mem, heights));
                ui.ctx().request_repaint();
            }
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
