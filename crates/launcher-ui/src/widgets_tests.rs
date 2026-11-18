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

#[test]
fn update_zeile_zeigt_alte_und_neue_version() {
    use crate::widgets::version_text;
    let t = Texts::de();
    let mut c = card(CardState::UpdateAvailable);
    c.installed = Some("1.0.0".into());
    c.latest = Some("1.1.0".into());
    assert_eq!(version_text(&t, &c), "V1.0.0 auf V1.1.0");
    c.state = CardState::Installed;
    assert_eq!(version_text(&t, &c), "V1.0.0");
}
