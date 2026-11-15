use crate::i18n::Texts;
use crate::model::{Busy, CardState, Load, Model, Notice};
use daedyx_update::catalog::{App, Catalog};
use daedyx_update::error::UpdateError;
use launcher_core::error::CoreError;
use launcher_core::jobs::Event;
use launcher_core::service::{AppStatus, Outcome};

fn catalog() -> Catalog {
    Catalog {
        schema: 1,
        min_launcher_version: "0.1.0".into(),
        apps: vec![App {
            id: "notizbuch".into(),
            name: "Notizbuch".into(),
            description: "Kurze Notizen.".into(),
            license: String::new(),
            author: "Beispiel-Studio".into(),
            registry_key: None,
            releases: vec![],
        }],
    }
}

fn status(installed: Option<&str>, latest: &str, update: bool) -> Event {
    Event::Status(vec![AppStatus {
        id: "notizbuch".into(),
        name: "Notizbuch".into(),
        installed: installed.map(String::from),
        latest: Some(latest.into()),
        update_available: update,
        running: false,
    }])
}

fn ready(installed: Option<&str>, update: bool) -> (Model, Texts) {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(Event::Catalog(Ok(catalog())), &t);
    m.apply(status(installed, "1.1.0", update), &t);
    (m, t)
}

#[test]
fn am_anfang_wird_geladen() {
    assert_eq!(Model::new().load, Load::Loading);
}

#[test]
fn katalog_und_status_ergeben_karten() {
    let (m, _) = ready(None, false);
    assert_eq!(m.load, Load::Ready);
    assert_eq!(m.cards.len(), 1);
    assert_eq!(m.cards[0].name, "Notizbuch");
    assert_eq!(m.cards[0].description, "Kurze Notizen.");
    assert_eq!(m.cards[0].author, "Beispiel-Studio");
    assert_eq!(m.cards[0].state, CardState::NotInstalled);
}

#[test]
fn installierte_und_veraltete_apps_haben_ihren_zustand() {
    let (m, _) = ready(Some("1.1.0"), false);
    assert_eq!(m.cards[0].state, CardState::Installed);
    let (m, _) = ready(Some("1.0.0"), true);
    assert_eq!(m.cards[0].state, CardState::UpdateAvailable);
}

#[test]
fn fehler_beim_ersten_laden_ersetzt_die_ansicht() {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(
        Event::Catalog(Err(UpdateError::SignatureInvalid.into())),
        &t,
    );
    match &m.load {
        Load::Failed(text) => assert!(text.contains("Echtheit")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn fehler_beim_neuladen_behaelt_die_karten() {
    let (mut m, t) = ready(None, false);
    m.apply(Event::Catalog(Err(CoreError::Download("weg".into()))), &t);
    assert_eq!(m.load, Load::Ready);
    assert_eq!(m.cards.len(), 1);
    assert!(matches!(m.notice, Some(Notice::Error(_))));
}

#[test]
fn fortschritt_setzt_busy_und_wechselt_bei_100_prozent_zum_installieren() {
    let (mut m, t) = ready(None, false);
    m.begin(
        "notizbuch",
        Busy::Downloading {
            done: 0,
            total: None,
        },
    );
    m.apply(
        Event::Progress {
            app: "notizbuch".into(),
            done: 50,
            total: Some(100),
        },
        &t,
    );
    assert_eq!(
        m.busy["notizbuch"],
        Busy::Downloading {
            done: 50,
            total: Some(100)
        }
    );
    m.apply(
        Event::Progress {
            app: "notizbuch".into(),
            done: 100,
            total: Some(100),
        },
        &t,
    );
    assert_eq!(m.busy["notizbuch"], Busy::Installing);
}

#[test]
fn erfolgreiche_installation_raeumt_auf_und_meldet() {
    let (mut m, t) = ready(None, false);
    m.begin("notizbuch", Busy::Installing);
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Ok(Outcome::Installed {
                version: "1.1.0".into(),
            }),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    assert_eq!(
        m.notice,
        Some(Notice::Info("Notizbuch 1.1.0 wurde installiert.".into()))
    );
}

#[test]
fn update_und_schon_aktuell_haben_eigene_hinweise() {
    let (mut m, t) = ready(Some("1.0.0"), true);
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Ok(Outcome::Updated {
                from: "1.0.0".into(),
                to: "1.1.0".into(),
            }),
        },
        &t,
    );
    assert_eq!(
        m.notice,
        Some(Notice::Info(
            "Notizbuch wurde auf Version 1.1.0 aktualisiert.".into()
        ))
    );
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Ok(Outcome::UpToDate {
                version: "1.1.0".into(),
            }),
        },
        &t,
    );
    assert_eq!(
        m.notice,
        Some(Notice::Info("Notizbuch ist schon aktuell.".into()))
    );
}

#[test]
fn fehlgeschlagene_installation_zeigt_fehlertext_und_gibt_die_karte_frei() {
    let (mut m, t) = ready(None, false);
    m.begin("notizbuch", Busy::Installing);
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Err(UpdateError::ChecksumMismatch.into()),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    match m.notice {
        Some(Notice::Error(text)) => assert!(text.contains("nichts installiert")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn entfernen_und_oeffnen_melden_ergebnis() {
    let (mut m, t) = ready(Some("1.0.0"), false);
    m.begin("notizbuch", Busy::Removing);
    m.apply(
        Event::Removed {
            app: "notizbuch".into(),
            result: Ok(()),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    assert_eq!(
        m.notice,
        Some(Notice::Info("Notizbuch wurde entfernt.".into()))
    );
    m.apply(
        Event::Opened {
            app: "notizbuch".into(),
            result: Ok(()),
        },
        &t,
    );
    assert_eq!(
        m.notice,
        Some(Notice::Info("Notizbuch wird gestartet.".into()))
    );
    m.apply(
        Event::Opened {
            app: "notizbuch".into(),
            result: Err(CoreError::NotInstalled("notizbuch".into())),
        },
        &t,
    );
    assert!(matches!(m.notice, Some(Notice::Error(_))));
}

#[test]
fn neue_aktion_loescht_den_alten_hinweis() {
    let (mut m, _) = ready(None, false);
    m.notice = Some(Notice::Info("alt".into()));
    m.begin(
        "notizbuch",
        Busy::Downloading {
            done: 0,
            total: None,
        },
    );
    assert_eq!(m.notice, None);
}

#[test]
fn laeuft_meldung_setzt_und_loescht_die_markierung() {
    let (mut m, t) = ready(Some("1.0.0"), false);
    assert!(!m.cards[0].running);
    m.apply(Event::Running(vec!["notizbuch".into()]), &t);
    assert!(m.cards[0].running);
    m.apply(Event::Running(vec![]), &t);
    assert!(!m.cards[0].running);
}

#[test]
fn status_mit_laeuft_wird_uebernommen() {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(Event::Catalog(Ok(catalog())), &t);
    m.apply(
        Event::Status(vec![AppStatus {
            id: "notizbuch".into(),
            name: "Notizbuch".into(),
            installed: Some("1.0.0".into()),
            latest: Some("1.0.0".into()),
            update_available: false,
            running: true,
        }]),
        &t,
    );
    assert!(m.cards[0].running);
}

fn launcher_update() -> launcher_core::selfupdate::SelfUpdate {
    launcher_core::selfupdate::SelfUpdate {
        version: "0.2.0".into(),
        notes: String::new(),
    }
}

#[test]
fn launcher_update_wird_gemerkt_und_beim_neuen_katalog_geloescht() {
    let (mut m, t) = ready(None, false);
    m.apply(Event::SelfUpdate(Some(launcher_update())), &t);
    assert_eq!(
        m.self_update.as_ref().map(|u| u.version.as_str()),
        Some("0.2.0")
    );
    m.apply(Event::Catalog(Ok(catalog())), &t);
    assert!(m.self_update.is_none(), "der neue Katalog entscheidet neu");
}

#[test]
fn fortschritt_des_launcher_updates_steht_getrennt_von_den_apps() {
    let (mut m, t) = ready(None, false);
    m.apply(Event::SelfUpdate(Some(launcher_update())), &t);
    m.apply(
        Event::Progress {
            app: "launcher".into(),
            done: 50,
            total: Some(200),
        },
        &t,
    );
    assert_eq!(m.self_progress, Some(0.25));
    assert!(m.busy.is_empty());
}

#[test]
fn gestartetes_setup_loest_das_beenden_aus_ein_fehler_nicht() {
    let (mut m, t) = ready(None, false);
    m.apply(Event::SelfUpdateStarted(Ok(Some(launcher_update()))), &t);
    assert!(m.quit_requested);
    let (mut m, t) = ready(None, false);
    m.self_progress = Some(0.5);
    m.apply(
        Event::SelfUpdateStarted(Err(UpdateError::ChecksumMismatch.into())),
        &t,
    );
    assert!(!m.quit_requested);
    assert_eq!(m.self_progress, None);
    assert!(matches!(m.notice, Some(Notice::Error(_))));
}

#[test]
fn anzahl_der_app_updates_wird_gezaehlt() {
    let (m, _) = ready(Some("1.0.0"), true);
    assert_eq!(m.app_updates(), 1);
    let (m, _) = ready(Some("1.1.0"), false);
    assert_eq!(m.app_updates(), 0);
}
