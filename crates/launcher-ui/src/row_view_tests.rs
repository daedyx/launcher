use crate::i18n::Texts;
use crate::model::{Busy, Card, CardState};
use crate::row_view;
use egui_kittest::Harness;

fn card() -> Card {
    Card {
        id: "notizbuch".into(),
        name: "Notizbuch".into(),
        description: "Kurze Notizen sammeln und schnell wiederfinden.".into(),
        installed: Some("1.0.0".into()),
        latest: Some("1.1.0".into()),
        state: CardState::UpdateAvailable,
        date: None,
        license: String::new(),
        author: "Beispiel-Studio".into(),
        running: false,
    }
}

/// Beim Download darf die Zeile nicht breiter werden als das Fenster (E1).
#[test]
fn zeile_beim_download_bleibt_im_fenster() {
    for lang in [crate::i18n::Language::De, crate::i18n::Language::En] {
        let t = Texts::for_language(lang);
        for busy in [
            Busy::Downloading {
                done: 100,
                total: Some(1000),
            },
            Busy::Downloading {
                done: 5_000_000,
                total: None,
            },
            Busy::Installing,
        ] {
            let c = card();
            let used = std::cell::Cell::new(0.0_f32);
            let avail = std::cell::Cell::new(0.0_f32);
            let mut h = Harness::builder()
                .with_size(eframe::egui::vec2(1000.0, 200.0))
                .build_ui(|ui| {
                    avail.set(ui.max_rect().right());
                    row_view::show(ui, &t, &c, Some(&busy));
                    used.set(ui.min_rect().right());
                });
            h.run_steps(3);
            let (u, a) = (used.get(), avail.get());
            assert!(u <= a + 0.5, "{lang:?} {busy:?}: Zeile ragt {u} > {a}");
        }
    }
}

/// E9: Die Zeile wird beim Download nicht höher als sonst.
#[test]
fn zeile_springt_beim_download_nicht_in_der_hoehe() {
    let t = Texts::de();
    let c = card();
    let height = |busy: Option<Busy>| {
        let h = std::cell::Cell::new(0.0_f32);
        let mut harness = Harness::builder()
            .with_size(eframe::egui::vec2(1000.0, 300.0))
            .build_ui(|ui| {
                row_view::show(ui, &t, &c, busy.as_ref());
                h.set(ui.min_rect().height());
            });
        harness.run_steps(3);
        h.get()
    };
    let idle = height(None);
    let busy = height(Some(Busy::Downloading {
        done: 10,
        total: Some(100),
    }));
    assert!(busy <= idle + 0.5, "Zeile {busy} > {idle}");
}
