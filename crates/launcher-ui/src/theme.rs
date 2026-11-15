use crate::palette::Palette;
use crate::settings::ThemeChoice;
use eframe::egui::{self, Color32, CornerRadius, Stroke};

fn preference(choice: ThemeChoice) -> egui::ThemePreference {
    match choice {
        ThemeChoice::System => egui::ThemePreference::System,
        ThemeChoice::Light => egui::ThemePreference::Light,
        ThemeChoice::Dark => egui::ThemePreference::Dark,
    }
}

/// Stellt das Thema um (Einstellungsseite).
pub fn set(ctx: &egui::Context, choice: ThemeChoice) {
    ctx.set_theme(preference(choice));
}

/// Mischt zwei Farben (`t` = Anteil von `b`).
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// Hell oder dunkel folgt Windows, solange nichts anderes gewählt ist. Zum Prüfen überstimmt
/// `DAEDYX_THEME=dark|light` die Einstellung.
pub fn apply(ctx: &egui::Context, choice: ThemeChoice) {
    match std::env::var("DAEDYX_THEME").as_deref() {
        Ok("dark") => set(ctx, ThemeChoice::Dark),
        Ok("light") => set(ctx, ThemeChoice::Light),
        _ => set(ctx, choice),
    }
    crate::fonts::install(ctx);
    ctx.all_styles_mut(|style| {
        let p = Palette::of(style.visuals.dark_mode);
        let round = CornerRadius::same(8);
        let v = &mut style.visuals;
        v.panel_fill = p.bg;
        v.window_fill = p.card;
        v.extreme_bg_color = p.card;
        v.faint_bg_color = p.bg;
        v.hyperlink_color = p.primary_text;
        v.weak_text_color = Some(p.text2);
        v.warn_fg_color = p.amber_text;
        v.error_fg_color = p.danger_text;
        v.selection.bg_fill = p.primary;
        v.selection.stroke = Stroke::new(1.0, p.on_primary);
        v.window_corner_radius = CornerRadius::same(10);
        v.window_stroke = Stroke::new(1.0, p.border);
        let hover = mix(p.card, p.primary, 0.10);
        let press = mix(p.card, p.primary, 0.20);
        let w = &mut v.widgets;
        for (state, fill) in [
            (&mut w.noninteractive, p.bg),
            (&mut w.inactive, p.card),
            (&mut w.hovered, hover),
            (&mut w.active, press),
            (&mut w.open, hover),
        ] {
            state.corner_radius = round;
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.bg_stroke = Stroke::new(1.0, p.border);
            state.fg_stroke = Stroke::new(1.0, p.text);
        }
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        for (text_style, font) in style.text_styles.iter_mut() {
            if matches!(text_style, egui::TextStyle::Body | egui::TextStyle::Button) {
                font.size = 15.0;
            }
        }
    });
}
