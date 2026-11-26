//! Dienste des Launchers mit gefälschtem Server und Attrappe für den Nachweis (U3): installieren, aktualisieren,
//! entfernen, Registry übernehmen, laufende Apps, gemeinsame Sperre mit den Apps.
use crate::error::CoreError;
use crate::service::{Launcher, Outcome};
use crate::state::{InstalledApp, State};
use crate::system::UninstallEntry;
use crate::test_support::Env;
use daedyx_update::lock::Lock;
use daedyx_update::reason::BlockKind;

fn install(l: &Launcher, _env: &Env) -> Result<Outcome, CoreError> {
    let lineup = l.load_lineup();
    l.install(&lineup, "desk", &mut |_, _| {})
}

#[test]
fn status_vor_der_installation() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    let catalog = l.load_lineup();
    let s = l.status(&catalog).unwrap();
    assert_eq!(
        s.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
        ["desk", "tasks"]
    );
    assert_eq!(s[1].latest, None, "Tasks ist nicht veröffentlicht (404)");
    assert_eq!(s[0].installed, None);
    assert_eq!(s[0].latest.as_deref(), Some("1.0.0"));
    assert!(!s[0].update_available);
}

#[test]
fn installieren_schreibt_app_und_zustand() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, installer) = env.launcher();
    assert_eq!(
        install(&l, &env).unwrap(),
        Outcome::Installed {
            version: "1.0.0".into()
        }
    );
    assert!(env.data_dir.path().join("apps/desk/app.json").exists());
    assert_eq!(
        State::load(env.data_dir.path()).unwrap().apps["desk"].version,
        "1.0.0"
    );
    assert_eq!(
        installer.calls.lock().unwrap().as_slice(),
        ["install 1.0.0"]
    );
    let catalog = l.load_lineup();
    assert_eq!(
        l.status(&catalog).unwrap()[0].installed.as_deref(),
        Some("1.0.0")
    );
}

#[test]
fn download_ordner_ist_nach_der_installation_leer() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    install(&l, &env).unwrap();
    let downloads = env.data_dir.path().join("downloads").join("daedyx-desk");
    assert_eq!(std::fs::read_dir(downloads).unwrap().count(), 0);
}

#[test]
fn zweite_installation_ist_schon_aktuell() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, installer) = env.launcher();
    install(&l, &env).unwrap();
    assert_eq!(
        install(&l, &env).unwrap(),
        Outcome::UpToDate {
            version: "1.0.0".into()
        }
    );
    assert_eq!(installer.calls.lock().unwrap().len(), 1);
}

#[test]
fn update_auf_neuere_version() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    install(&l, &env).unwrap();

    env.publish(&["1.0.0", "1.1.0"]);
    let catalog = l.load_lineup();
    assert!(l.status(&catalog).unwrap()[0].update_available);
    assert_eq!(
        install(&l, &env).unwrap(),
        Outcome::Updated {
            from: "1.0.0".into(),
            to: "1.1.0".into()
        }
    );
    let catalog = l.load_lineup();
    let s = &l.status(&catalog).unwrap()[0];
    assert_eq!(s.installed.as_deref(), Some("1.1.0"));
    assert!(!s.update_available);
}

#[test]
fn falsche_pruefsumme_startet_den_installer_nicht() {
    let env = Env::new();
    env.publish_with(&["1.0.0"], Some("1.0.0"));
    let (l, installer) = env.launcher();
    assert!(matches!(
        install(&l, &env),
        Err(CoreError::Blocked { blocked, .. }) if blocked.kind == BlockKind::NotTrusted
    ));
    assert!(installer.calls.lock().unwrap().is_empty());
    assert!(!env.data_dir.path().join("apps/desk").exists());
    assert_eq!(
        std::fs::read_dir(env.data_dir.path().join("downloads").join("daedyx-desk"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn entfernen_loescht_app_und_zustand() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    install(&l, &env).unwrap();
    l.remove("desk").unwrap();
    assert!(!env.data_dir.path().join("apps/desk").exists());
    assert!(State::load(env.data_dir.path()).unwrap().apps.is_empty());
    assert!(matches!(l.remove("desk"), Err(CoreError::NotInstalled(_))));
}

#[test]
fn entfernen_lehnt_gefaehrliche_kennungen_ab() {
    let env = Env::new();
    let (l, _) = env.launcher();
    for bad in ["..", "../x", "a/b", "", "board"] {
        assert!(
            matches!(l.remove(bad), Err(CoreError::UnknownApp(_))),
            "{bad:?} hätte abgelehnt werden müssen"
        );
    }
}

#[test]
fn ordner_ausserhalb_des_programmordners_wird_nie_entfernt() {
    let env = Env::new();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("wichtig.txt"), b"x").unwrap();
    let mut state = State::default();
    state.apps.insert(
        "desk".into(),
        InstalledApp {
            version: "1.0.0".into(),
            install_dir: outside.path().to_path_buf(),
        },
    );
    state.save(env.data_dir.path()).unwrap();
    let (l, installer) = env.launcher();
    assert!(matches!(l.remove("desk"), Err(CoreError::Io(_))));
    assert!(outside.path().join("wichtig.txt").exists());
    assert!(installer.calls.lock().unwrap().is_empty());
}

#[test]
fn laufende_installation_sperrt_eine_zweite() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    // Dieselbe Sperre, die Desk bei seinem eigenen Update holt (`<lokal>\daedyx\locks\daedyx-desk.lock`).
    let held = Lock::acquire(&env.lock_dir(), "daedyx-desk").unwrap();
    assert!(matches!(install(&l, &env), Err(CoreError::Busy(a)) if a == "desk"));
    assert!(!env.data_dir.path().join("apps/desk").exists());
    drop(held);
    assert!(matches!(install(&l, &env), Ok(Outcome::Installed { .. })));
}

#[test]
fn launcher_kann_zwischen_threads_wandern() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Launcher>();
}

/// Legt eine „von Hand installierte“ App an: Ordner mit `app.json` und Registry-Eintrag.
fn manual_install(
    env: &Env,
    system: &crate::test_support::FakeSystem,
    version: &str,
) -> std::path::PathBuf {
    let dir = env.data_dir.path().join("fremd").join("Daedyx Desk");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("app.json"),
        format!(r#"{{"version":"{version}","exe":"desk.exe"}}"#),
    )
    .unwrap();
    system.entries.lock().unwrap().insert(
        "Daedyx Desk".into(),
        UninstallEntry {
            display_version: Some(version.into()),
            install_location: Some(dir.clone()),
        },
    );
    dir
}

#[test]
fn von_hand_installierte_app_wird_aus_der_registry_uebernommen() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, system) = env.launcher_with_system();
    let dir = manual_install(&env, &system, "0.5.0");
    let catalog = l.load_lineup();
    let s = l.status(&catalog).unwrap();
    assert_eq!(s[0].installed.as_deref(), Some("0.5.0"));
    assert!(s[0].update_available);
    assert_eq!(
        State::load(env.data_dir.path()).unwrap().apps["desk"].install_dir,
        dir
    );
}

#[test]
fn ohne_app_json_gilt_die_version_aus_der_registry() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, system) = env.launcher_with_system();
    let dir = manual_install(&env, &system, "0.4.0");
    std::fs::remove_file(dir.join("app.json")).unwrap();
    let catalog = l.load_lineup();
    assert_eq!(
        l.status(&catalog).unwrap()[0].installed.as_deref(),
        Some("0.4.0")
    );
}

#[test]
fn registry_eintrag_ohne_ordner_gilt_nicht_als_installiert() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, system) = env.launcher_with_system();
    let dir = manual_install(&env, &system, "0.4.0");
    std::fs::remove_dir_all(&dir).unwrap();
    let catalog = l.load_lineup();
    assert_eq!(l.status(&catalog).unwrap()[0].installed, None);
}

#[test]
fn update_einer_uebernommenen_app_geschieht_an_ort_und_stelle() {
    let env = Env::new();
    env.publish(&["1.1.0"]);
    let (l, installer, system) = env.launcher_with_system();
    let dir = manual_install(&env, &system, "1.0.0");
    assert_eq!(
        install(&l, &env).unwrap(),
        Outcome::Updated {
            from: "1.0.0".into(),
            to: "1.1.0".into()
        }
    );
    assert_eq!(
        installer.calls.lock().unwrap().as_slice(),
        ["install 1.1.0"]
    );
    let json = std::fs::read_to_string(dir.join("app.json")).unwrap();
    assert!(json.contains("1.1.0"), "{json}");
    assert!(!env.data_dir.path().join("apps/desk").exists());
}

#[test]
fn laufende_app_wird_nicht_aktualisiert_und_nichts_geladen() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, installer, system) = env.launcher_with_system();
    install(&l, &env).unwrap();
    env.publish(&["1.0.0", "1.1.0"]);
    system
        .running
        .lock()
        .unwrap()
        .push(env.data_dir.path().join("apps/desk"));
    assert!(matches!(install(&l, &env), Err(CoreError::AppRunning(a)) if a == "desk"));
    assert_eq!(installer.calls.lock().unwrap().len(), 1);
    assert_eq!(
        std::fs::read_dir(env.data_dir.path().join("downloads").join("daedyx-desk"))
            .unwrap()
            .count(),
        0
    );
    // Wieder geschlossen: das Update geht durch.
    system.running.lock().unwrap().clear();
    assert!(matches!(install(&l, &env), Ok(Outcome::Updated { .. })));
}

#[test]
fn status_und_running_apps_melden_laeuft() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, system) = env.launcher_with_system();
    install(&l, &env).unwrap();
    let catalog = l.load_lineup();
    assert!(!l.status(&catalog).unwrap()[0].running);
    assert!(l.running_apps().unwrap().is_empty());
    system
        .running
        .lock()
        .unwrap()
        .push(env.data_dir.path().join("apps/desk"));
    assert!(l.status(&catalog).unwrap()[0].running);
    assert_eq!(l.running_apps().unwrap(), ["desk"]);
}

#[test]
fn laufende_app_wird_nicht_entfernt() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, system) = env.launcher_with_system();
    install(&l, &env).unwrap();
    system
        .running
        .lock()
        .unwrap()
        .push(env.data_dir.path().join("apps/desk"));
    assert!(matches!(l.remove("desk"), Err(CoreError::AppRunning(_))));
    assert!(env.data_dir.path().join("apps/desk/app.json").exists());
}

#[test]
fn oeffnen_findet_auch_eine_uebernommene_app() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, system) = env.launcher_with_system();
    manual_install(&env, &system, "0.5.0");
    // Das Startprogramm `desk.exe` gibt es in der Attrappe nicht: gefunden, aber nicht startbar.
    assert!(matches!(l.open("desk"), Err(CoreError::NotLaunchable(_))));
}

#[test]
fn ein_abgelehnter_nachweis_installiert_nichts() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    env.trusted
        .store(false, std::sync::atomic::Ordering::SeqCst);
    let (l, installer) = env.launcher();
    let lineup = l.load_lineup();
    assert_eq!(
        lineup.get("desk").unwrap().blocked().unwrap().kind,
        BlockKind::NotTrusted
    );
    assert!(matches!(
        l.install(&lineup, "desk", &mut |_, _| {}),
        Err(CoreError::Blocked { blocked, .. }) if blocked.kind == BlockKind::NotTrusted
    ));
    assert!(installer.calls.lock().unwrap().is_empty());
}

#[test]
fn ohne_netz_bleiben_installierte_apps_sichtbar_und_nichts_wird_installiert() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, installer) = env.launcher();
    install(&l, &env).unwrap();
    env.server
        .offline
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let lineup = l.load_lineup();
    assert_eq!(lineup.problem(), Some(BlockKind::Offline));
    let s = l.status(&lineup).unwrap();
    assert_eq!(s[0].installed.as_deref(), Some("1.0.0"));
    assert_eq!(s[0].latest, None);
    assert!(matches!(
        l.install(&lineup, "desk", &mut |_, _| {}),
        Err(CoreError::Blocked { blocked, .. }) if blocked.kind == BlockKind::Offline
    ));
    assert_eq!(installer.calls.lock().unwrap().len(), 1);
}

#[test]
fn ein_eintrag_fuer_einen_neueren_launcher_wird_nicht_geladen() {
    let env = Env::new();
    env.release(
        crate::apps::find("desk").unwrap(),
        "1.0.0",
        false,
        Some("9.0.0"),
    );
    let (l, installer) = env.launcher();
    assert!(
        matches!(install(&l, &env), Err(CoreError::LauncherTooOld { need, .. }) if need == "9.0.0")
    );
    assert!(installer.calls.lock().unwrap().is_empty());
    let urls = env.server.requests.lock().unwrap().clone();
    assert!(
        !urls.iter().any(|u| u.ends_with("-setup.exe")),
        "kein Setup angefragt: {urls:?}"
    );
}

#[test]
fn nur_eine_echt_hoehere_version_wird_angeboten() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    install(&l, &env).unwrap();
    // Die Quelle springt zurück: kein Angebot, kein Update.
    env.publish(&["0.9.0"]);
    let lineup = l.load_lineup();
    assert_eq!(
        lineup.get("desk").unwrap().blocked().unwrap().kind,
        BlockKind::SourceWentBack
    );
    assert!(!l.status(&lineup).unwrap()[0].update_available);
}

#[cfg(windows)]
#[test]
fn das_setup_ist_waehrend_der_installation_gegen_schreiben_gesperrt() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, installer) = env.launcher();
    install(&l, &env).unwrap();
    assert_eq!(
        installer.writable.lock().unwrap().as_slice(),
        [false],
        "M-U3-1: gesperrter Griff bis zum Ende"
    );
}

#[test]
fn die_app_laedt_in_ihren_eigenen_ordner_und_hinterlaesst_nichts() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    install(&l, &env).unwrap();
    let own = env.data_dir.path().join("downloads").join("daedyx-desk");
    assert!(own.is_dir(), "eigener Ordner je App (N-U3-1)");
    assert_eq!(std::fs::read_dir(own).unwrap().count(), 0);
}
