use crate::focus::focus_rect;
use egui_kittest::Harness;

/// E8: Wer mit Tab durch die Oberfläche geht, bekommt ein Rechteck für den Fokusring.
#[test]
fn tab_setzt_den_fokus_auf_einen_knopf_mit_rechteck() {
    let mut h = Harness::new_ui(|ui| {
        let _ = ui.button("Eins");
        let _ = ui.button("Zwei");
    });
    h.run();
    assert!(focus_rect(&h.ctx).is_none());
    h.key_press(eframe::egui::Key::Tab);
    h.run();
    let first = focus_rect(&h.ctx).expect("Fokus nach Tab");
    h.key_press(eframe::egui::Key::Tab);
    h.run();
    let second = focus_rect(&h.ctx).expect("Fokus nach zweitem Tab");
    assert!(second.top() >= first.bottom() - 1.0);
}
