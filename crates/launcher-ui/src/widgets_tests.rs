use crate::i18n::Texts;
use crate::model::{Card, CardState};
use crate::widgets::{badge_saturation, checked_text, format_date};

#[test]
fn datum_wird_deutsch_geschrieben() {
    assert_eq!(format_date("2026-11-13"), "13.11.2026");
    assert_eq!(format_date("gestern"), "gestern");
    assert_eq!(format_date("2026-1-3"), "2026-1-3");
}

#[test]
fn pruefzeit_zeigt_gerade_eben_und_minuten() {
    let t = Texts::de();
    assert_eq!(
        checked_text(&t, 0),
        "Katalog-Signatur geprüft · gerade eben"
    );
    assert_eq!(
        checked_text(&t, 11),
        "Katalog-Signatur geprüft · vor 11 Min."
    );
}

fn card(state: CardState) -> Card {
    Card {
        id: "x".into(),
        name: "X".into(),
        description: String::new(),
        installed: None,
        latest: None,
        state,
        date: None,
        license: String::new(),
        author: String::new(),
        running: false,
    }
}

#[test]
fn installierte_apps_sind_immer_farbig_andere_nur_beim_darueberfahren() {
    assert!(badge_saturation(&card(CardState::Installed), false) > 0.0);
    assert!(badge_saturation(&card(CardState::UpdateAvailable), false) > 0.0);
    assert_eq!(badge_saturation(&card(CardState::NotInstalled), false), 0.0);
    assert!(badge_saturation(&card(CardState::NotInstalled), true) > 0.0);
}
