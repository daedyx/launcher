//! Rahmen des Fensters: Seitenleiste mit Marke und den drei Seiten, rechts der Inhalt.
use crate::brand;
use crate::fonts;
use crate::i18n::Texts;
use crate::palette::Palette;
use crate::theme::mix;
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, RichText, Sense, Vec2,
};

pub const SIDEBAR_WIDTH: f32 = 196.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Apps,
    Updates,
    Settings,
}

/// Große Seitenüberschrift in der Markenschrift.
pub fn page_title(ui: &mut egui::Ui, text: &str) {
    let color = Palette::current(ui).text;
    ui.label(
        RichText::new(text)
            .font(FontId::new(30.0, fonts::brand()))
            .variation(b"wght", 700.0)
            .color(color),
    );
}

/// Zeichnet Seitenleiste und Inhalt. `updates`: Zahl am Eintrag „Updates“ (0 blendet sie aus).
pub fn show(
    ui: &mut egui::Ui,
    t: &Texts,
    page: &mut Page,
    updates: usize,
    content: impl FnOnce(&mut egui::Ui),
) {
    let p = Palette::current(ui);
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(p.bg))
        .show(ui, |ui| {
            let spacing = ui.spacing().item_spacing;
            let height = ui.available_height();
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(SIDEBAR_WIDTH, height),
                    Layout::top_down(Align::Min),
                    |ui| {
                        egui::Frame::new()
                            .fill(p.sidebar)
                            .inner_margin(14)
                            .show(ui, |ui| {
                                ui.set_min_size(Vec2::new(SIDEBAR_WIDTH - 28.0, height - 28.0));
                                ui.spacing_mut().item_spacing = spacing;
                                sidebar(ui, t, &p, page, updates);
                            });
                    },
                );
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), height),
                    Layout::top_down(Align::Min),
                    |ui| {
                        egui::Frame::new().inner_margin(20).show(ui, |ui| {
                            ui.spacing_mut().item_spacing = spacing;
                            content(ui);
                        });
                    },
                );
            });
        });
}

fn sidebar(ui: &mut egui::Ui, t: &Texts, p: &Palette, page: &mut Page, updates: usize) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
        brand::paint_mark(ui.painter(), rect, false);
        ui.add_space(6.0);
        ui.label(
            RichText::new(t.t("brand.name").to_lowercase())
                .font(FontId::new(25.0, fonts::brand()))
                .variation(b"wght", 800.0)
                .color(p.sidebar_text),
        );
    });
    ui.add_space(22.0);
    for (target, key) in [
        (Page::Apps, "nav.apps"),
        (Page::Updates, "nav.updates"),
        (Page::Settings, "nav.settings"),
    ] {
        let badge = if target == Page::Updates { updates } else { 0 };
        if nav_item(ui, p, &t.t(key), *page == target, badge) {
            *page = target;
        }
        ui.add_space(4.0);
    }
    ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
        let version = t.tf("footer.launcher", &[("version", env!("CARGO_PKG_VERSION"))]);
        ui.label(RichText::new(version).monospace().size(10.5).color(mix(
            p.sidebar,
            p.sidebar_text,
            0.6,
        )));
    });
}

/// Ein Eintrag der Seitenleiste; der aktive trägt die Bernsteinleiste.
fn nav_item(ui: &mut egui::Ui, p: &Palette, label: &str, active: bool, badge: usize) -> bool {
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, label));
    let painter = ui.painter();
    if active {
        painter.rect_filled(rect, CornerRadius::same(8), p.sidebar_active);
        painter.rect_filled(
            egui::Rect::from_min_size(rect.min + Vec2::new(0.0, 8.0), Vec2::new(3.0, 20.0)),
            CornerRadius::same(2),
            p.amber,
        );
    } else if response.hovered() {
        painter.rect_filled(
            rect,
            CornerRadius::same(8),
            mix(p.sidebar, p.sidebar_active, 0.5),
        );
    }
    painter.text(
        rect.left_center() + Vec2::new(14.0, 0.0),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.5),
        p.sidebar_text,
    );
    if badge > 0 {
        let center = rect.right_center() - Vec2::new(18.0, 0.0);
        painter.circle_filled(center, 10.0, p.amber);
        painter.text(
            center,
            Align2::CENTER_CENTER,
            badge.to_string(),
            FontId::proportional(12.0),
            Color32::from_rgb(19, 19, 42),
        );
    }
    response.clicked()
}
