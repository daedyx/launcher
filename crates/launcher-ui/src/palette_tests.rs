use crate::palette::{contrast, Palette};

/// Lesbarkeit nach WCAG: normaler Text braucht 4,5 zu 1, große Flächenfarben mindestens 3 zu 1.
fn check(name: &str, p: &Palette) {
    let text_pairs = [
        ("Text auf Fläche", p.text, p.bg),
        ("Text auf Karte", p.text, p.card),
        ("Zweittext auf Fläche", p.text2, p.bg),
        ("Zweittext auf Karte", p.text2, p.card),
        ("Schrift auf Hauptknopf", p.on_primary, p.primary),
        ("Seitenleiste", p.sidebar_text, p.sidebar),
        ("Seitenleiste aktiv", p.sidebar_text, p.sidebar_active),
        ("Hinweis Bernstein", p.amber_text, p.amber_bg),
        ("Hinweis Grün", p.green_text, p.green_bg),
        ("Hinweis Rot", p.danger_text, p.danger_bg),
        ("Hauptfarbe als Text auf Karte", p.primary_text, p.card),
    ];
    for (what, fg, bg) in text_pairs {
        let c = contrast(fg, bg);
        assert!(c >= 4.5, "{name}: {what} hat nur Kontrast {c:.2}");
    }
    let c = contrast(p.border, p.bg);
    assert!(c >= 1.1, "{name}: Linie ist unsichtbar ({c:.2})");
}

#[test]
fn helle_palette_ist_gut_lesbar() {
    check("hell", &Palette::light());
}

#[test]
fn dunkle_palette_ist_gut_lesbar() {
    check("dunkel", &Palette::dark());
}

#[test]
fn kontrast_rechnet_wie_wcag() {
    use eframe::egui::Color32;
    let black = Color32::BLACK;
    let white = Color32::WHITE;
    assert!((contrast(black, white) - 21.0).abs() < 0.01);
    assert!((contrast(white, white) - 1.0).abs() < 0.01);
}
