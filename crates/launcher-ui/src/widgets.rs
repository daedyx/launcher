//! Gemeinsame Bausteine von Kachel und Listenzeile.
use crate::i18n::Texts;
use crate::model::{Busy, Card, CardState};
use crate::palette::Palette;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, ProgressBar, RichText, Sense, Stroke, Vec2,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Install(String),
    Update(String),
    Open(String),
    Remove(String),
}

/// Hover der letzten Darstellung: Das Symbol ist grau und wird beim Darüberfahren farbig.
pub fn hover_state(ui: &egui::Ui, card: &Card, busy: Option<&Busy>) -> (egui::Id, bool) {
    let id = egui::Id::unique(("card-hover", &card.id));
    let hovered = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false) || busy.is_some();
    (id, hovered)
}

pub fn track_hover(ui: &egui::Ui, id: egui::Id, was: bool, rect: egui::Rect, busy: Option<&Busy>) {
    let now = ui.rect_contains_pointer(rect);
    if now != was && busy.is_none() {
        ui.data_mut(|d| d.insert_temp(id, now));
        ui.request_repaint();
    }
}

/// Installierte Apps sind immer farbig; nicht installierte grau, außer beim Darüberfahren.
pub fn badge_saturation(card: &Card, hovered: bool) -> f32 {
    if hovered || card.state != CardState::NotInstalled {
        0.55
    } else {
        0.0
    }
}

/// App-Marke: abgerundetes Quadrat mit Anfangsbuchstaben, Farbe aus der App-Kennung
/// (Symbole als Bild: spätere Version).
pub fn badge(ui: &mut egui::Ui, card: &Card, colored: bool, size: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let hue = card
        .id
        .bytes()
        .fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32))
        % 360;
    let saturation = badge_saturation(card, colored);
    let value = if ui.visuals().dark_mode { 0.68 } else { 0.55 };
    let color: Color32 = egui::ecolor::Hsva::new(hue as f32 / 360.0, saturation, value, 1.0).into();
    ui.painter()
        .rect_filled(rect, CornerRadius::same((size * 0.26) as u8), color);
    let initial = card
        .name
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string();
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        initial,
        FontId::proportional(size * 0.5),
        Color32::WHITE,
    );
}

/// Installierte Version, sonst die neueste, als „V1.2.3“.
pub fn version_text(t: &Texts, card: &Card) -> String {
    if let (CardState::UpdateAvailable, Some(from), Some(to)) =
        (card.state, card.installed.as_ref(), card.latest.as_ref())
    {
        return t.tf("version.update", &[("from", from), ("to", to)]);
    }
    match card.installed.as_ref().or(card.latest.as_ref()) {
        Some(v) => t.tf("version.short", &[("version", v)]),
        None => String::new(),
    }
}

pub fn small_mono(text: String) -> RichText {
    RichText::new(text).monospace().size(11.0)
}

fn spinner_with(ui: &mut egui::Ui, text: String) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(text);
    });
}

/// Fortschritt in fester Breite (Listenzeile): Balken oben, Text darunter und notfalls gekürzt,
/// damit die Zeile in keiner Sprache breiter wird als vorgesehen.
pub fn busy_view_in(ui: &mut egui::Ui, t: &Texts, busy: &Busy, width: f32) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, 26.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing.y = 2.0;
            let (frac, key, args) = match busy {
                Busy::Downloading { done, total } => match total.filter(|x| *x > 0) {
                    Some(total) => {
                        let f = (*done as f32 / total as f32).clamp(0.0, 1.0);
                        (
                            Some(f),
                            "busy.downloading",
                            vec![("percent", format!("{:.0}", f * 100.0))],
                        )
                    }
                    None => (
                        None,
                        "busy.downloading_mb",
                        vec![("mb", (done / (1024 * 1024)).to_string())],
                    ),
                },
                Busy::Installing => (None, "busy.installing", vec![]),
                Busy::Removing => (None, "busy.removing", vec![]),
            };
            match frac {
                Some(f) => {
                    ui.add(
                        ProgressBar::new(f)
                            .desired_width(width)
                            .desired_height(10.0)
                            .fill(Palette::current(ui).primary),
                    );
                }
                None => {
                    ui.add(
                        egui::ProgressBar::new(0.0)
                            .desired_width(width)
                            .desired_height(10.0)
                            .animate(true),
                    );
                }
            }
            let a: Vec<(&str, &str)> = args.iter().map(|(k, v)| (*k, v.as_str())).collect();
            ui.add(egui::Label::new(RichText::new(t.tf(key, &a)).size(12.0)).truncate());
        },
    );
}

pub fn busy_view(ui: &mut egui::Ui, t: &Texts, busy: &Busy) {
    match busy {
        Busy::Downloading { done, total } => match total.filter(|x| *x > 0) {
            Some(total) => {
                let frac = (*done as f32 / total as f32).clamp(0.0, 1.0);
                ui.add(
                    ProgressBar::new(frac)
                        .desired_width(64.0)
                        .fill(Palette::current(ui).primary),
                );
                ui.label(t.tf(
                    "busy.downloading",
                    &[("percent", &format!("{:.0}", frac * 100.0))],
                ));
            }
            None => spinner_with(
                ui,
                t.tf(
                    "busy.downloading_mb",
                    &[("mb", &(done / (1024 * 1024)).to_string())],
                ),
            ),
        },
        Busy::Installing => spinner_with(ui, t.t("busy.installing")),
        Busy::Removing => spinner_with(ui, t.t("busy.removing")),
    }
}

/// Alle Knöpfe einer App in einer Zeile: ein breiter Hauptknopf und, wenn es mehr gibt, ein
/// kleines „…“-Menü. `main_width`: feste Breite des Hauptknopfs, sonst füllt er die Zeile.
pub fn buttons(
    ui: &mut egui::Ui,
    t: &Texts,
    card: &Card,
    main_width: Option<f32>,
) -> Option<Action> {
    let id = card.id.clone();
    let mut action = None;
    let (key, main, more): (&str, Action, Vec<(&str, Action)>) = match card.state {
        CardState::NotInstalled => ("action.install", Action::Install(id), vec![]),
        CardState::UpdateAvailable => (
            "action.update",
            Action::Update(id.clone()),
            vec![
                ("action.open", Action::Open(id.clone())),
                ("action.remove", Action::Remove(id)),
            ],
        ),
        CardState::Installed => (
            "action.open",
            Action::Open(id.clone()),
            vec![("action.remove", Action::Remove(id))],
        ),
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().button_padding = egui::vec2(10.0, 5.0);
        ui.spacing_mut().item_spacing.x = 6.0;
        let more_width = if more.is_empty() { 0.0 } else { 40.0 };
        let width = main_width
            .unwrap_or_else(|| ui.available_width() - more_width - ui.spacing().item_spacing.x);
        let p = Palette::current(ui);
        let button = if card.state == CardState::Installed {
            // Schon installiert: heller Knopf mit Rand; der gefüllte Knopf bleibt für „Neues tun“.
            egui::Button::new(RichText::new(t.t(key)).size(13.0).color(p.primary_text))
                .fill(Color32::TRANSPARENT)
                .stroke(Stroke::new(1.0, p.primary_text))
        } else {
            egui::Button::new(
                RichText::new(t.t(key))
                    .size(13.0)
                    .strong()
                    .color(p.on_primary),
            )
            .fill(p.primary)
        };
        if ui.add_sized([width.max(60.0), 30.0], button).clicked() {
            action = Some(main);
        }
        if !more.is_empty() {
            let menu = ui.menu_button(t.t("action.more"), |ui| {
                for (key, act) in more {
                    if ui.button(t.t(key)).clicked() {
                        action = Some(act);
                        ui.close();
                    }
                }
            });
            menu.response.on_hover_text(t.t("action.more_tip"));
        }
    });
    action
}

/// „2026-11-13“ wird zu „13.11.2026“; alles andere bleibt unverändert.
pub fn format_date(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    match parts.as_slice() {
        [y, m, d] if y.len() == 4 && m.len() == 2 && d.len() == 2 => format!("{d}.{m}.{y}"),
        _ => date.to_string(),
    }
}

/// Text der Fußzeile: wie lange die Katalog-Signatur zuletzt geprüft wurde.
pub fn checked_text(t: &Texts, minutes: u64) -> String {
    if minutes == 0 {
        t.t("footer.checked_now")
    } else {
        t.tf("footer.checked_min", &[("min", &minutes.to_string())])
    }
}
