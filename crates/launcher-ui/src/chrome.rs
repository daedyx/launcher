//! Rahmen des Fensters: Kopfleiste mit Marke, Werkzeugleiste, Fußzeile.
use crate::filter::Sort;
use crate::i18n::Texts;
use crate::section_view::View;
use crate::widgets::checked_text;
use eframe::egui::{self, RichText};

/// Werkzeugleiste: Neu laden links; Sortierung, Suche und Ansicht rechts. Gibt `true` zurück,
/// wenn „Aktualisieren“ gedrückt wurde.
pub fn toolbar(
    ui: &mut egui::Ui,
    t: &Texts,
    query: &mut String,
    sort: &mut Sort,
    view: &mut View,
) -> bool {
    let mut refresh = false;
    ui.horizontal(|ui| {
        refresh = ui.button(t.t("header.refresh")).clicked();
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.selectable_value(view, View::List, t.t("view.list"));
            ui.selectable_value(view, View::Tiles, t.t("view.tiles"));
            ui.add(
                egui::TextEdit::singleline(query)
                    .hint_text(t.t("header.search"))
                    .desired_width(180.0),
            );
            let current = match sort {
                Sort::NameAsc => t.t("sort.name_asc"),
                Sort::NameDesc => t.t("sort.name_desc"),
            };
            egui::ComboBox::from_label(t.t("header.sort"))
                .selected_text(current)
                .show_ui(ui, |ui| {
                    ui.selectable_value(sort, Sort::NameAsc, t.t("sort.name_asc"));
                    ui.selectable_value(sort, Sort::NameDesc, t.t("sort.name_desc"));
                });
        });
    });
    refresh
}

/// Fußzeile: links die Signaturprüfung des Katalogs, rechts die Launcher-Version.
pub fn footer(ui: &mut egui::Ui, t: &Texts, checked_minutes: Option<u64>, updates: usize) {
    ui.horizontal(|ui| {
        if let Some(min) = checked_minutes {
            ui.label(
                RichText::new(checked_text(t, min).to_uppercase())
                    .monospace()
                    .size(10.5)
                    .weak(),
            );
        }
        if updates > 0 {
            let text = if updates == 1 {
                t.t("footer.update_one")
            } else {
                t.tf("footer.updates", &[("count", &updates.to_string())])
            };
            ui.label(
                RichText::new(format!("· {}", text.to_uppercase()))
                    .monospace()
                    .size(10.5),
            );
        }
    });
    ui.separator();
}
