//! Kleine Etiketten (Pillen) und Versionsangabe.
use crate::i18n::Texts;
use crate::model::{Card, CardState};
use crate::palette::Palette;
use crate::theme::mix;
use crate::widgets::small_mono;
use eframe::egui::{self, Color32, CornerRadius, Margin, RichText, Stroke};

/// Gefüllte Pille mit kurzem Text.
pub fn pill(ui: &mut egui::Ui, text: &str, fg: Color32, bg: Color32) {
    egui::Frame::new()
        .fill(bg)
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::symmetric(8, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.5).color(fg));
        });
}

/// Status-Etikett einer App.
pub fn chip(ui: &mut egui::Ui, t: &Texts, card: &Card) {
    let p = Palette::current(ui);
    let (key, fg, bg) = match card.state {
        CardState::NotInstalled => (
            "tag.available",
            p.primary_text,
            mix(p.card, p.primary, 0.10),
        ),
        CardState::Installed => ("tag.installed", p.text2, p.bg),
        CardState::UpdateAvailable => ("tag.update", p.amber_text, p.amber_bg),
    };
    pill(ui, &t.t(key), fg, bg);
}

/// Grünes Etikett „Läuft“, solange die App gerade läuft.
pub fn running_chip(ui: &mut egui::Ui, t: &Texts, card: &Card) {
    if card.running {
        let p = Palette::current(ui);
        pill(ui, &t.t("tag.running"), p.green_text, p.green_bg);
    }
}

/// Status-Etikett, daneben (falls zutreffend) „Läuft“.
pub fn chips(ui: &mut egui::Ui, t: &Texts, card: &Card) {
    ui.horizontal(|ui| {
        chip(ui, t, card);
        running_chip(ui, t, card);
    });
}

/// „von Autor“, schlicht umrandet; ohne Autor im Katalog entfällt es.
pub fn author_chip(ui: &mut egui::Ui, t: &Texts, card: &Card) {
    if card.author.is_empty() {
        return;
    }
    let p = Palette::current(ui);
    egui::Frame::new()
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::symmetric(8, 1))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    RichText::new(t.tf("tag.author", &[("author", &card.author)]))
                        .size(11.0)
                        .color(p.text2),
                )
                .extend(),
            );
        });
}

/// Installierte Version, sonst die neueste, als „V1.2.3“ in der Hauptfarbe.
pub fn version_label(ui: &mut egui::Ui, t: &Texts, card: &Card) {
    let color = Palette::current(ui).primary_text;
    ui.label(small_mono(crate::widgets::version_text(t, card)).color(color));
}
