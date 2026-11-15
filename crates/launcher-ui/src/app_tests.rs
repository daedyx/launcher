use crate::app::LauncherApp;
use crate::i18n::Texts;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

#[test]
fn ohne_katalogquelle_erklaert_das_fenster_was_fehlt() {
    let t = Texts::de();
    let msg = t.t("setup.missing");
    let mut app = LauncherApp::new(t, Err(msg));
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    assert!(h.query_by_label_contains(".pub").is_some());
}

#[test]
fn zahnrad_oeffnet_die_einstellungsseite_mit_dem_erscheinungsbild() {
    let t = Texts::de();
    let msg = t.t("setup.missing");
    let mut app = LauncherApp::new(t, Err(msg));
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    assert!(h.query_by_label("ERSCHEINUNGSBILD").is_none());
    h.get_by_label("Einstellungen").click();
    h.run();
    assert!(h.query_by_label("ERSCHEINUNGSBILD").is_some());
    assert!(h.query_by_label("DUNKEL").is_some());
}

#[test]
fn seitenleiste_wechselt_zwischen_den_seiten() {
    let t = Texts::de();
    let msg = t.t("setup.missing");
    let mut app = LauncherApp::new(t, Err(msg));
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    assert!(h.query_by_label("Deine Apps").is_some());
    h.get_by_label("Einstellungen").click();
    h.run();
    assert!(h.query_by_label("Dein Launcher").is_some());
    assert!(h.query_by_label("ERSCHEINUNGSBILD").is_some());
    h.get_by_label("Updates").click();
    h.run();
    assert!(h.query_by_label("Updates im Überblick").is_some());
    assert!(h
        .query_by_label("Alles ist auf dem neuesten Stand.")
        .is_some());
    h.get_by_label("Apps").click();
    h.run();
    assert!(h.query_by_label("Deine Apps").is_some());
}

#[test]
fn sprachwechsel_aendert_die_texte_und_wird_gespeichert() {
    use crate::settings::UiSettings;
    use crate::settings_view::AppInfo;
    let dir = tempfile::tempdir().unwrap();
    let t = Texts::de();
    let msg = t.t("setup.missing");
    let info = AppInfo {
        data_dir: dir.path().to_path_buf(),
    };
    let mut app = LauncherApp::new(t, Err(msg)).with_settings(info, UiSettings::default());
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    h.get_by_label("Einstellungen").click();
    h.run();
    assert!(h.query_by_label("SPRACHE").is_some());
    h.get_by_label("ENGLISH").click();
    h.run();
    assert!(h.query_by_label("LANGUAGE").is_some());
    assert!(h.query_by_label("Your launcher").is_some());
    drop(h);
    assert_eq!(
        UiSettings::load(dir.path()).language,
        crate::settings::LanguageChoice::En
    );
}

const TEST_PUBKEY: &str = "untrusted comment: minisign public key: EE603AFC068379EA\nRWTqeYMG/Dpg7jJhQhTLnr67E2PDv9nvxLkaLTyjaAHE72fy3n3yVdn8\n";

fn catalog_with(app_running: bool) -> (LauncherApp, crate::card_view::Action) {
    use crate::model::CardState;
    use launcher_core::jobs::Event;
    use launcher_core::service::AppStatus;
    let t = Texts::de();
    let mut app = LauncherApp::new(t, Err("x".into()));
    let t = Texts::de();
    app.model.apply(
        Event::Catalog(Ok(daedyx_update::catalog::Catalog {
            schema: 1,
            min_launcher_version: "0.1.0".into(),
            apps: vec![daedyx_update::catalog::App {
                id: "desk".into(),
                name: "Daedyx Desk".into(),
                description: String::new(),
                license: String::new(),
                author: String::new(),
                registry_key: None,
                releases: vec![],
            }],
        })),
        &t,
    );
    app.model.apply(
        Event::Status(vec![AppStatus {
            id: "desk".into(),
            name: "Daedyx Desk".into(),
            installed: Some("0.1.0".into()),
            latest: Some("0.2.0".into()),
            update_available: true,
            running: app_running,
        }]),
        &t,
    );
    assert_eq!(app.model.cards[0].state, CardState::UpdateAvailable);
    (app, crate::card_view::Action::Update("desk".into()))
}

#[test]
fn update_einer_laufenden_app_fragt_erst_nach() {
    let (mut app, update) = catalog_with(true);
    app.dispatch(update);
    assert_eq!(app.confirm_running.as_deref(), Some("desk"));
    assert!(app.model.busy.is_empty(), "es darf nichts gestartet sein");
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    assert!(h.query_by_label("Daedyx Desk läuft gerade").is_some());
    assert!(h.query_by_label("Jetzt aktualisieren").is_some());
}

#[test]
fn update_einer_nicht_laufenden_app_startet_sofort() {
    let (mut app, update) = catalog_with(false);
    app.dispatch(update);
    assert!(app.confirm_running.is_none());
    assert!(app.model.busy.contains_key("desk"));
}

#[test]
fn rueckfrage_geht_erst_nach_dem_schliessen_der_app_weiter() {
    use launcher_core::jobs::Event;
    let (mut app, update) = catalog_with(true);
    app.dispatch(update);
    // App wird geschlossen: die regelmäßige Abfrage meldet „läuft nicht mehr“
    app.model.apply(Event::Running(vec![]), &Texts::de());
    app.dispatch(crate::card_view::Action::Update("desk".into()));
    assert!(app.model.busy.contains_key("desk"));
}

fn app_with_rebuild(
    data_dir: &std::path::Path,
    rebuild: crate::app_settings::Rebuild,
) -> LauncherApp {
    use crate::settings::UiSettings;
    use crate::settings_view::AppInfo;
    let t = Texts::de();
    let msg = t.t("setup.missing");
    let info = AppInfo {
        data_dir: data_dir.to_path_buf(),
    };
    LauncherApp::new(t, Err(msg))
        .with_settings(info, UiSettings::default())
        .with_rebuild(&launcher_core::config::Config::default(), rebuild)
}

fn open_settings_and_save(app: &mut LauncherApp) {
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    h.get_by_label("Einstellungen").click();
    h.run();
    h.get_by_label("SPEICHERN UND NEU LADEN").click();
    h.run();
}

#[test]
fn ungueltige_katalogquelle_zeigt_fehler_und_speichert_nichts() {
    let dir = tempfile::tempdir().unwrap();
    let mut app = app_with_rebuild(dir.path(), Box::new(|_, t| Err(t.t("error.key_invalid"))));
    app.config_edit.catalog = "C:\\gibt-es-nicht".into();
    open_settings_and_save(&mut app);
    assert!(matches!(&app.config_edit.message, Some(Err(m)) if m.contains("Schlüssel")));
    assert!(!dir.path().join("config.json").exists());
}

#[test]
fn gueltige_katalogquelle_wird_dauerhaft_gespeichert_und_neu_geladen() {
    use launcher_core::config::{build_launcher, Config};
    use launcher_core::jobs::Worker;
    let dir = tempfile::tempdir().unwrap();
    let catalog = tempfile::tempdir().unwrap();
    let key = dir.path().join("catalog.pub");
    std::fs::write(&key, TEST_PUBKEY).unwrap();
    let data = dir.path().to_path_buf();
    let mut app = app_with_rebuild(
        dir.path(),
        Box::new(move |config, _| {
            let (launcher, pubkey) =
                build_launcher(config, data.clone(), "0.1.0").map_err(|e| e.to_string())?;
            Ok(Worker::start(launcher, pubkey, std::sync::Arc::new(|| {})))
        }),
    );
    app.config_edit.catalog = catalog.path().display().to_string();
    app.config_edit.pubkey = key.display().to_string();
    open_settings_and_save(&mut app);
    assert!(
        matches!(&app.config_edit.message, Some(Ok(_))),
        "{:?}",
        app.config_edit.message
    );
    let saved = Config::load(dir.path());
    assert_eq!(
        saved.catalog.as_deref(),
        Some(catalog.path().to_str().unwrap())
    );
    assert_eq!(saved.pubkey, Some(key));
    assert!(app.worker.is_some());
}

fn app_with_self_update(auto: bool) -> LauncherApp {
    use crate::settings::UiSettings;
    use launcher_core::jobs::Event;
    let t = Texts::de();
    let mut app = LauncherApp::new(t, Err("x".into())).with_settings(
        crate::settings_view::AppInfo::default(),
        UiSettings {
            auto_update_launcher: auto,
            ..UiSettings::default()
        },
    );
    app.model.apply(
        Event::SelfUpdate(Some(launcher_core::selfupdate::SelfUpdate {
            version: "0.2.0".into(),
            notes: String::new(),
        })),
        &Texts::de(),
    );
    app
}

/// Ein Bild zeichnen (`run` käme bei dauerhaft drehenden Spinnern nie zur Ruhe).
fn draw_once(app: &mut LauncherApp) {
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.step();
}

#[test]
fn launcher_update_wird_bei_voreinstellung_von_selbst_gestartet() {
    let mut app = app_with_self_update(true);
    draw_once(&mut app);
    assert!(app.self_update_requested);
    assert_eq!(app.model.self_progress, Some(0.0));
}

#[test]
fn ausgeschaltet_wird_nur_angezeigt_und_per_knopf_gestartet() {
    let mut app = app_with_self_update(false);
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.step();
    assert!(h.query_by_label("Launcher 0.2.0 ist verfügbar.").is_some());
    h.get_by_label("Jetzt aktualisieren").click();
    h.step();
    h.step();
    drop(h);
    assert!(app.self_update_requested);
}

#[test]
fn waehrend_einer_installation_wird_der_launcher_nicht_ersetzt() {
    let mut app = app_with_self_update(true);
    app.model.begin("desk", crate::model::Busy::Installing);
    draw_once(&mut app);
    assert!(!app.self_update_requested);
    app.model.busy.clear();
    draw_once(&mut app);
    assert!(app.self_update_requested);
}

#[test]
fn nach_fehlgeschlagenem_selbst_update_versucht_die_automatik_es_nicht_erneut() {
    use launcher_core::jobs::Event;
    let mut app = app_with_self_update(true);
    draw_once(&mut app);
    assert!(app.self_update_requested);
    app.model.apply(
        Event::SelfUpdateStarted(Err(
            daedyx_update::error::UpdateError::ChecksumMismatch.into()
        )),
        &Texts::de(),
    );
    draw_once(&mut app);
    // die Automatik hat es für diese Sitzung versucht; der Knopf bleibt für Handarbeit
    assert_eq!(app.model.self_progress, None);
    assert!(app.model.self_update.is_some());
}

#[test]
fn regelmaessige_pruefung_nur_wenn_faellig_und_nichts_laeuft() {
    use crate::app::refresh_due;
    use std::time::Duration;
    assert!(!refresh_due(Duration::from_secs(60), true));
    assert!(refresh_due(Duration::from_secs(6 * 3600 + 1), true));
    assert!(!refresh_due(Duration::from_secs(7 * 3600), false));
}

#[cfg(windows)]
#[test]
fn schalter_beim_anmelden_starten_schreibt_und_entfernt_den_eintrag() {
    use crate::autostart::Autostart;
    use crate::settings::UiSettings;
    use crate::settings_view::AppInfo;
    let dir = tempfile::tempdir().unwrap();
    let key = format!(r"Software\DaedyxLauncherTest\App{}", std::process::id());
    let auto = || Autostart {
        run_key: key.clone(),
    };
    let t = Texts::de();
    let msg = t.t("setup.missing");
    let mut app = LauncherApp::new(t, Err(msg))
        .with_settings(
            AppInfo {
                data_dir: dir.path().to_path_buf(),
            },
            UiSettings::default(),
        )
        .with_autostart(auto());
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    h.get_by_label("Einstellungen").click();
    h.run();
    h.get_by_label("Beim Anmelden starten").click();
    h.run();
    assert!(auto().is_enabled());
    h.get_by_label("Beim Anmelden starten").click();
    h.run();
    assert!(!auto().is_enabled());
    drop(h);
    let _ = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).delete_subkey_all(&key);
}
