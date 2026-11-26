use crate::i18n::{Language, Texts};
use crate::palette::Palette;
use crate::settings::{LanguageChoice, ThemeChoice, UiSettings};
use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke};
use std::path::PathBuf;

/// Angaben vom Start, die die Einstellungsseite nur anzeigt.
#[derive(Debug, Clone, Default)]
pub struct AppInfo {
    pub data_dir: PathBuf,
}

#[derive(Debug, Default, PartialEq)]
pub struct Outcome {
    pub theme_changed: bool,
    pub language_changed: bool,
    pub updates_changed: bool,
    pub autostart_changed: bool,
    pub quit: bool,
}

fn heading(ui: &mut egui::Ui, text: String) {
    let color = Palette::current(ui).text2;
    ui.label(
        RichText::new(text.to_uppercase())
            .monospace()
            .size(11.0)
            .color(color),
    );
}

/// Umrandete Karte mit Mono-Überschrift.
fn boxed(ui: &mut egui::Ui, title: String, content: impl FnOnce(&mut egui::Ui)) {
    let p = Palette::current(ui);
    egui::Frame::new()
        .fill(p.card)
        .stroke(Stroke::new(1.0, p.border))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(16.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            heading(ui, title);
            ui.add_space(8.0);
            content(ui);
        });
}

fn mono_button_text(text: String) -> RichText {
    RichText::new(text.to_uppercase()).monospace().size(12.0)
}

fn outlined_button(ui: &mut egui::Ui, text: String) -> bool {
    let color = Palette::current(ui).primary_text;
    ui.add(
        egui::Button::new(mono_button_text(text).color(color))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, color))
            .min_size(egui::vec2(110.0, 34.0)),
    )
    .clicked()
}

/// Auswahlleiste mit aneinanderliegenden Knöpfen; der aktive ist gefüllt.
fn segmented<T: Copy + PartialEq>(ui: &mut egui::Ui, current: &mut T, options: Vec<(T, String)>) {
    let p = Palette::current(ui);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.spacing_mut().button_padding = egui::vec2(14.0, 8.0);
        for (choice, label) in options {
            let text = mono_button_text(label);
            let button = if *current == choice {
                egui::Button::new(text.strong().color(p.on_primary)).fill(p.primary)
            } else {
                egui::Button::new(text.color(p.text))
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, p.border))
            };
            if ui.add(button).clicked() {
                *current = choice;
            }
        }
    });
}

/// Inhalt der Einstellungsseite (die Überschrift zeichnet der Rahmen). Gibt zurück, was sich
/// ergeben hat.
pub fn show(ui: &mut egui::Ui, t: &Texts, info: &AppInfo, settings: &mut UiSettings) -> Outcome {
    let mut out = Outcome::default();
    let p = Palette::current(ui);
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.columns(2, |cols| {
                let [left, right] = cols else { return };
                boxed(left, t.t("settings.theme"), |ui| {
                    let before = settings.theme;
                    segmented(
                        ui,
                        &mut settings.theme,
                        vec![
                            (ThemeChoice::System, t.t("theme.system")),
                            (ThemeChoice::Dark, t.t("theme.dark")),
                            (ThemeChoice::Light, t.t("theme.light")),
                        ],
                    );
                    out.theme_changed = before != settings.theme;
                });
                left.add_space(8.0);
                boxed(left, t.t("settings.language"), |ui| {
                    let before = settings.language;
                    let options = vec![
                        (LanguageChoice::System, t.t("language.system")),
                        (LanguageChoice::De, Language::De.native_name().to_string()),
                        (LanguageChoice::En, Language::En.native_name().to_string()),
                    ];
                    segmented(ui, &mut settings.language, options);
                    out.language_changed = before != settings.language;
                });
                left.add_space(8.0);
                boxed(left, t.t("settings.box_updates"), |ui| {
                    out.updates_changed |= ui
                        .checkbox(
                            &mut settings.auto_update_launcher,
                            t.t("settings.auto_update"),
                        )
                        .changed();
                    out.autostart_changed |= ui
                        .checkbox(&mut settings.autostart, t.t("settings.autostart"))
                        .changed();
                });
                // Update-Quelle (U3): fest im Programm, nichts einzustellen; nur der Hinweis, woher Updates kommen.
                boxed(right, t.t("settings.box_source"), |ui| {
                    ui.label(
                        RichText::new(t.t("settings.source_hint"))
                            .size(12.0)
                            .color(p.text2),
                    );
                });
            });
            ui.add_space(8.0);
            boxed(ui, t.t("settings.data"), |ui| {
                ui.add(
                    egui::Label::new(RichText::new(info.data_dir.display().to_string()).size(13.5))
                        .wrap(),
                );
            });
            ui.add_space(12.0);
            if outlined_button(ui, t.t("settings.quit")) {
                out.quit = true;
            }
        });
    out
}
