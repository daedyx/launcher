use crate::card_view::{show, Action};
use crate::i18n::Texts;
use crate::model::{Busy, Card, CardState};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

fn card(state: CardState) -> Card {
    Card {
        id: "notizbuch".into(),
        name: "Notizbuch".into(),
        description: "Kurze Notizen sammeln.".into(),
        installed: (state != CardState::NotInstalled).then(|| "1.0.0".to_string()),
        latest: Some("1.1.0".into()),
        state,
        date: Some("2026-11-13".into()),
        license: "MIT".into(),
        author: String::new(),
        running: false,
    }
}

/// Zeichnet die Karte, klickt optional einen Knopf und gibt die gemeldete Aktion zurück.
fn click(state: CardState, busy: Option<Busy>, label: &str) -> Option<Action> {
    let t = Texts::de();
    let c = card(state);
    let mut action = None;
    {
        let mut h = Harness::new_ui(|ui| {
            if let Some(a) = show(ui, &t, &c, busy.as_ref(), 320.0) {
                action = Some(a);
            }
        });
        h.get_by_label(label).click();
        h.run();
    }
    action
}

/// Öffnet das „…“-Menü und klickt dort einen Eintrag.
fn click_in_menu(state: CardState, label: &str) -> Option<Action> {
    let t = Texts::de();
    let c = card(state);
    let mut action = None;
    {
        let mut h = Harness::new_ui(|ui| {
            if let Some(a) = show(ui, &t, &c, None, 320.0) {
                action = Some(a);
            }
        });
        h.get_by_label("…").click();
        h.run();
        h.get_by_label(label).click();
        h.run();
    }
    action
}

fn has_label(state: CardState, busy: Option<Busy>, label: &str) -> bool {
    let t = Texts::de();
    let c = card(state);
    let h = Harness::new_ui(|ui| {
        show(ui, &t, &c, busy.as_ref(), 320.0);
    });
    h.query_by_label(label).is_some()
}

#[test]
fn nicht_installiert_bietet_installieren() {
    assert!(has_label(CardState::NotInstalled, None, "Installieren"));
    assert!(!has_label(CardState::NotInstalled, None, "Öffnen"));
    assert_eq!(
        click(CardState::NotInstalled, None, "Installieren"),
        Some(Action::Install("notizbuch".into()))
    );
}

#[test]
fn installiert_bietet_oeffnen_und_entfernen() {
    assert_eq!(
        click(CardState::Installed, None, "Öffnen"),
        Some(Action::Open("notizbuch".into()))
    );
    assert_eq!(
        click_in_menu(CardState::Installed, "Entfernen"),
        Some(Action::Remove("notizbuch".into()))
    );
    assert!(!has_label(CardState::Installed, None, "Installieren"));
}

#[test]
fn update_bietet_aktualisieren() {
    assert_eq!(
        click(CardState::UpdateAvailable, None, "Aktualisieren"),
        Some(Action::Update("notizbuch".into()))
    );
    assert_eq!(
        click_in_menu(CardState::UpdateAvailable, "Öffnen"),
        Some(Action::Open("notizbuch".into()))
    );
}

#[test]
fn etikett_version_lizenz_und_datum_stehen_auf_der_kachel() {
    assert!(has_label(CardState::Installed, None, "Installiert"));
    assert!(has_label(
        CardState::UpdateAvailable,
        None,
        "Update verfügbar"
    ));
    assert!(has_label(CardState::NotInstalled, None, "Installierbar"));
    assert!(has_label(CardState::Installed, None, "V1.0.0"));
    assert!(has_label(CardState::NotInstalled, None, "V1.1.0"));
    assert!(has_label(CardState::Installed, None, "MIT"));
    assert!(has_label(
        CardState::Installed,
        None,
        "Zuletzt aktualisiert: 13.11.2026"
    ));
}

#[test]
fn nicht_installiert_hat_kein_menue() {
    assert!(!has_label(CardState::NotInstalled, None, "…"));
    assert!(has_label(CardState::Installed, None, "…"));
}

#[test]
fn name_und_kurztext_stehen_auf_der_karte() {
    assert!(has_label(CardState::NotInstalled, None, "Notizbuch"));
    assert!(has_label(
        CardState::NotInstalled,
        None,
        "Kurze Notizen sammeln."
    ));
}

#[test]
fn waehrend_des_downloads_gibt_es_keine_knoepfe_aber_fortschritt() {
    let busy = Some(Busy::Downloading {
        done: 50,
        total: Some(100),
    });
    assert!(!has_label(
        CardState::NotInstalled,
        busy.clone(),
        "Installieren"
    ));
    assert!(has_label(
        CardState::NotInstalled,
        busy,
        "Lädt herunter … 50 %"
    ));
}

#[test]
fn waehrend_installation_und_entfernen_steht_der_zustand_da() {
    assert!(has_label(
        CardState::NotInstalled,
        Some(Busy::Installing),
        "Wird installiert …"
    ));
    assert!(has_label(
        CardState::Installed,
        Some(Busy::Removing),
        "Wird entfernt …"
    ));
}

#[test]
fn karte_merkt_sich_das_darueberfahren_fuer_die_farbe() {
    let t = Texts::de();
    let c = card(CardState::Installed);
    let id = eframe::egui::Id::unique(("card-hover", &c.id));
    let mut h = Harness::new_ui(|ui| {
        show(ui, &t, &c, None, 320.0);
    });
    h.run();
    assert_eq!(h.ctx.data(|d| d.get_temp::<bool>(id)), None);
    h.get_by_label("Notizbuch").hover();
    h.run();
    assert_eq!(h.ctx.data(|d| d.get_temp::<bool>(id)), Some(true));
}

#[test]
fn laufende_app_zeigt_das_etikett_laeuft() {
    let t = Texts::de();
    let mut c = card(CardState::Installed);
    c.running = true;
    let h = Harness::new_ui(|ui| {
        show(ui, &t, &c, None, 320.0);
    });
    assert!(h.query_by_label("Läuft").is_some());
    assert!(!has_label(CardState::Installed, None, "Läuft"));
}
