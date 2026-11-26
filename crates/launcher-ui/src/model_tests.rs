use crate::i18n::Texts;
use crate::model::{Busy, CardState, Load, Model, Notice};
use launcher_core::apps;
use launcher_core::daedyx_update::reason::{BlockKind, Blocked};
use launcher_core::daedyx_update::version::Version;
use launcher_core::error::CoreError;
use launcher_core::jobs::Event;
use launcher_core::lineup::{Found, Lineup, Listed};
use launcher_core::service::{AppStatus, Outcome};

/// Suchergebnis wie ohne Veröffentlichung (Desk „aktuell“, Tasks ohne Quelle); Angebote prüft der Kern.
fn lineup() -> Lineup {
    lineup_with(Found::UpToDate(Version::new(1, 1, 0)))
}

fn lineup_with(desk: Found) -> Lineup {
    let blocked = |kind| {
        Found::Blocked(Blocked {
            kind,
            detail: "test".into(),
        })
    };
    Lineup {
        apps: vec![
            Listed {
                app: apps::find("desk").unwrap(),
                found: desk,
            },
            Listed {
                app: apps::find("tasks").unwrap(),
                found: blocked(BlockKind::SourceUnavailable),
            },
        ],
        launcher: Listed {
            app: &apps::LAUNCHER,
            found: blocked(BlockKind::SourceUnavailable),
        },
    }
}

fn not_trusted() -> CoreError {
    CoreError::Blocked {
        app: "desk".into(),
        blocked: Blocked {
            kind: BlockKind::NotTrusted,
            detail: "test".into(),
        },
    }
}

fn status(installed: Option<&str>, latest: &str, update: bool) -> Event {
    Event::Status(vec![AppStatus {
        id: "desk".into(),
        name: "Daedyx Desk".into(),
        installed: installed.map(String::from),
        latest: Some(latest.into()),
        update_available: update,
        running: false,
    }])
}

fn ready(installed: Option<&str>, update: bool) -> (Model, Texts) {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(Event::Lineup(lineup()), &t);
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
    assert_eq!(m.cards[0].name, "Daedyx Desk");
    assert_eq!(
        m.cards[0].description,
        "Mail, Kalender und Aufgaben in einer App."
    );
    assert_eq!(m.cards[0].author, "Daedyx");
    assert_eq!(m.cards[0].license, "MIT");
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
fn ohne_netz_bleiben_die_karten_und_ein_leiser_hinweis_erscheint() {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(
        Event::Lineup(lineup_with(Found::Blocked(Blocked {
            kind: BlockKind::Offline,
            detail: "x".into(),
        }))),
        &t,
    );
    m.apply(status(Some("1.0.0"), "1.0.0", false), &t);
    assert_eq!(m.load, Load::Ready);
    assert_eq!(m.cards.len(), 1);
    match &m.notice {
        Some(Notice::Info(text)) => assert!(text.contains("Keine Verbindung"), "{text}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn ein_abgelehnter_nachweis_ist_ein_fehlerhinweis() {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(
        Event::Lineup(lineup_with(Found::Blocked(Blocked {
            kind: BlockKind::NotTrusted,
            detail: "x".into(),
        }))),
        &t,
    );
    match &m.notice {
        Some(Notice::Error(text)) => assert!(text.contains("nicht sicher von Daedyx"), "{text}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn fortschritt_setzt_busy_und_wechselt_bei_100_prozent_zum_installieren() {
    let (mut m, t) = ready(None, false);
    m.begin(
        "desk",
        Busy::Downloading {
            done: 0,
            total: None,
        },
    );
    m.apply(
        Event::Progress {
            app: "desk".into(),
            done: 50,
            total: Some(100),
        },
        &t,
    );
    assert_eq!(
        m.busy["desk"],
        Busy::Downloading {
            done: 50,
            total: Some(100)
        }
    );
    m.apply(
        Event::Progress {
            app: "desk".into(),
            done: 100,
            total: Some(100),
        },
        &t,
    );
    assert_eq!(m.busy["desk"], Busy::Installing);
}

#[test]
fn erfolgreiche_installation_raeumt_auf_und_meldet() {
    let (mut m, t) = ready(None, false);
    m.begin("desk", Busy::Installing);
    m.apply(
        Event::Installed {
            app: "desk".into(),
            result: Ok(Outcome::Installed {
                version: "1.1.0".into(),
            }),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    assert_eq!(
        m.notice,
        Some(Notice::Info("Daedyx Desk 1.1.0 wurde installiert.".into()))
    );
}

#[test]
fn update_und_schon_aktuell_haben_eigene_hinweise() {
    let (mut m, t) = ready(Some("1.0.0"), true);
    m.apply(
        Event::Installed {
            app: "desk".into(),
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
            "Daedyx Desk wurde auf Version 1.1.0 aktualisiert.".into()
        ))
    );
    m.apply(
        Event::Installed {
            app: "desk".into(),
            result: Ok(Outcome::UpToDate {
                version: "1.1.0".into(),
            }),
        },
        &t,
    );
    assert_eq!(
        m.notice,
        Some(Notice::Info("Daedyx Desk ist schon aktuell.".into()))
    );
}

#[test]
fn fehlgeschlagene_installation_zeigt_fehlertext_und_gibt_die_karte_frei() {
    let (mut m, t) = ready(None, false);
    m.begin("desk", Busy::Installing);
    m.apply(
        Event::Installed {
            app: "desk".into(),
            result: Err(not_trusted()),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    match m.notice {
        Some(Notice::Error(text)) => assert!(text.contains("nicht installiert")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn entfernen_und_oeffnen_melden_ergebnis() {
    let (mut m, t) = ready(Some("1.0.0"), false);
    m.begin("desk", Busy::Removing);
    m.apply(
        Event::Removed {
            app: "desk".into(),
            result: Ok(()),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    assert_eq!(
        m.notice,
        Some(Notice::Info("Daedyx Desk wurde entfernt.".into()))
    );
    m.apply(
        Event::Opened {
            app: "desk".into(),
            result: Ok(()),
        },
        &t,
    );
    assert_eq!(
        m.notice,
        Some(Notice::Info("Daedyx Desk wird gestartet.".into()))
    );
    m.apply(
        Event::Opened {
            app: "desk".into(),
            result: Err(CoreError::NotInstalled("desk".into())),
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
        "desk",
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
    m.apply(Event::Running(vec!["desk".into()]), &t);
    assert!(m.cards[0].running);
    m.apply(Event::Running(vec![]), &t);
    assert!(!m.cards[0].running);
}

#[test]
fn status_mit_laeuft_wird_uebernommen() {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(Event::Lineup(lineup()), &t);
    m.apply(
        Event::Status(vec![AppStatus {
            id: "desk".into(),
            name: "Daedyx Desk".into(),
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
    m.apply(Event::Lineup(lineup()), &t);
    assert!(m.self_update.is_none(), "die neue Suche entscheidet neu");
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
    m.apply(Event::SelfUpdateStarted(Err(not_trusted())), &t);
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
