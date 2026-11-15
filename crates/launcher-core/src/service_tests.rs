use crate::error::CoreError;
use crate::lock::Lock;
use crate::service::{Launcher, Outcome};
use crate::state::{InstalledApp, State};
use crate::system::UninstallEntry;
use crate::test_support::Env;
use daedyx_update::error::UpdateError;

fn install(l: &Launcher, env: &Env) -> Result<Outcome, CoreError> {
    let catalog = l.load_catalog(&env.pubkey)?;
    l.install(&catalog, "desk", &mut |_, _| {})
}

#[test]
fn katalog_mit_gueltiger_signatur_wird_geladen() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    assert_eq!(l.load_catalog(&env.pubkey).unwrap().apps.len(), 1);
}

#[test]
fn veraenderter_katalog_wird_abgelehnt() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let path = env.catalog_dir.path().join("catalog.json");
    let mut json = std::fs::read_to_string(&path).unwrap();
    json.push(' ');
    std::fs::write(&path, json).unwrap();
    let (l, _) = env.launcher();
    assert!(matches!(
        l.load_catalog(&env.pubkey),
        Err(CoreError::Update(UpdateError::SignatureInvalid))
    ));
}

#[test]
fn katalog_mit_fremdem_schluessel_wird_abgelehnt() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let other = Env::new();
    let (l, _) = env.launcher();
    assert!(matches!(
        l.load_catalog(&other.pubkey),
        Err(CoreError::Update(UpdateError::SignatureInvalid))
    ));
}

#[test]
fn zu_alter_launcher_wird_abgelehnt() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (mut l, _) = env.launcher();
    l.launcher_version = "0.0.1".into();
    assert!(matches!(
        l.load_catalog(&env.pubkey),
        Err(CoreError::Update(UpdateError::LauncherTooOld { .. }))
    ));
}

#[test]
fn status_vor_der_installation() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    let catalog = l.load_catalog(&env.pubkey).unwrap();
    let s = l.status(&catalog).unwrap();
    assert_eq!(s.len(), 1);
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
    let catalog = l.load_catalog(&env.pubkey).unwrap();
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
    let downloads = env.data_dir.path().join("downloads");
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
    let catalog = l.load_catalog(&env.pubkey).unwrap();
    assert!(l.status(&catalog).unwrap()[0].update_available);
    assert_eq!(
        install(&l, &env).unwrap(),
        Outcome::Updated {
            from: "1.0.0".into(),
            to: "1.1.0".into()
        }
    );
    let catalog = l.load_catalog(&env.pubkey).unwrap();
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
        Err(CoreError::Update(UpdateError::ChecksumMismatch))
    ));
    assert!(installer.calls.lock().unwrap().is_empty());
    assert!(!env.data_dir.path().join("apps/desk").exists());
    assert_eq!(
        std::fs::read_dir(env.data_dir.path().join("downloads"))
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
    for bad in ["..", "../x", "a/b", ""] {
        assert!(
            matches!(
                l.remove(bad),
                Err(CoreError::Update(UpdateError::UnknownApp(_)))
            ),
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
    let _held = Lock::acquire(&env.data_dir.path().join("locks"), "desk").unwrap();
    assert!(matches!(install(&l, &env), Err(CoreError::Busy(_))));
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
    let catalog = l.load_catalog(&env.pubkey).unwrap();
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
    let catalog = l.load_catalog(&env.pubkey).unwrap();
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
    let catalog = l.load_catalog(&env.pubkey).unwrap();
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
        std::fs::read_dir(env.data_dir.path().join("downloads"))
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
    let catalog = l.load_catalog(&env.pubkey).unwrap();
    assert!(!l.status(&catalog).unwrap()[0].running);
    assert!(l.running_apps(&catalog).unwrap().is_empty());
    system
        .running
        .lock()
        .unwrap()
        .push(env.data_dir.path().join("apps/desk"));
    assert!(l.status(&catalog).unwrap()[0].running);
    assert_eq!(l.running_apps(&catalog).unwrap(), ["desk"]);
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
    let _ = l.load_catalog(&env.pubkey).unwrap(); // merkt sich den Registry-Namen
                                                  // Das Startprogramm `desk.exe` gibt es in der Attrappe nicht: gefunden, aber nicht startbar.
    assert!(matches!(l.open("desk"), Err(CoreError::NotLaunchable(_))));
}
