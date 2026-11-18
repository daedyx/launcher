//! Sichtbarer Tastaturfokus: ein Ring um das Element, das gerade die Tastatur hat.
//! (Die Knöpfe setzen Füllung und Rand selbst, daher reicht das Theme allein nicht.)
use crate::palette::Palette;
use eframe::egui::{self, Rect, Stroke, StrokeKind};

/// Rechteck des Elements mit Tastaturfokus, falls es eines gibt.
pub fn focus_rect(ctx: &egui::Context) -> Option<Rect> {
    let id = ctx.memory(|m| m.focused())?;
    ctx.read_response(id).map(|r| r.rect)
}

/// Zeichnet den Fokusring über alles andere. Am Ende jedes Bildes aufrufen.
pub fn draw_ring(ctx: &egui::Context) {
    let Some(rect) = focus_rect(ctx) else { return };
    let dark = ctx.global_style().visuals.dark_mode;
    let p = Palette::of(dark);
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        #[allow(deprecated)]
        egui::Id::new("focus-ring"),
    ));
    painter.rect_stroke(
        rect.expand(2.0),
        egui::CornerRadius::same(10),
        Stroke::new(2.0, p.amber),
        StrokeKind::Outside,
    );
}
