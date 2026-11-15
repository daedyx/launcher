use crate::error::CoreError;
use crate::selfupdate::SelfUpdate;
use crate::test_support::Env;
use daedyx_update::error::UpdateError;

fn catalog(
    env: &Env,
) -> (
    crate::service::Launcher,
    crate::test_support::FakeSystem,
    daedyx_update::catalog::Catalog,
) {
    let (l, _, system) = env.launcher_with_system();
    let c = l.load_catalog(&env.pubkey).unwrap();
    (l, system, c)
}

#[test]
fn ohne_launcher_eintrag_gibt_es_kein_update() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, c) = catalog(&env);
    assert_eq!(l.self_update_available(&c).unwrap(), None);
}

#[test]
fn neuere_version_im_katalog_wird_gemeldet_gleiche_oder_aeltere_nicht() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, _, c) = catalog(&env); // Launcher ist 0.1.0
    assert_eq!(
        l.self_update_available(&c).unwrap(),
        Some(SelfUpdate {
            version: "0.2.0".into(),
            notes: String::new()
        })
    );
    env.publish_apps(&["1.0.0"], &["0.1.0"], None);
    let (l, _, c) = catalog(&env);
    assert_eq!(l.self_update_available(&c).unwrap(), None);
}

#[test]
fn launcher_erscheint_nicht_als_app_in_der_liste() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, _, c) = catalog(&env);
    let ids: Vec<_> = l.status(&c).unwrap().into_iter().map(|s| s.id).collect();
    assert_eq!(ids, ["desk"]);
}

#[test]
fn selbst_update_laedt_geprueft_und_startet_das_setup_still() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, system, c) = catalog(&env);
    let started = l.start_self_update(&c, true, &mut |_, _| {}).unwrap();
    assert_eq!(started.map(|s| s.version), Some("0.2.0".to_string()));
    let calls = system.started.lock().unwrap();
    assert_eq!(calls.len(), 1);
    let (program, args) = &calls[0];
    assert!(program.starts_with(env.data_dir.path().join("downloads")));
    assert!(program.exists(), "das Setup muss bis zum Start da sein");
    assert_eq!(args, &["/S", "/UPDATE"]);
}

#[test]
fn ohne_neustart_wird_der_schalter_nolaunch_mitgegeben() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, system, c) = catalog(&env);
    l.start_self_update(&c, false, &mut |_, _| {}).unwrap();
    assert_eq!(
        system.started.lock().unwrap()[0].1,
        ["/S", "/UPDATE", "/NOLAUNCH"]
    );
}

#[test]
fn selbst_update_ohne_neuere_version_tut_nichts() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.1.0"], None);
    let (l, system, c) = catalog(&env);
    assert_eq!(l.start_self_update(&c, true, &mut |_, _| {}).unwrap(), None);
    assert!(system.started.lock().unwrap().is_empty());
}

#[test]
fn falsche_pruefsumme_startet_kein_setup_und_hinterlaesst_nichts() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    // Katalog gültig signiert, aber das Paket wird nach der Veröffentlichung verändert
    std::fs::write(
        env.catalog_dir.path().join("packages/launcher-0.2.0.exe"),
        b"manipuliert",
    )
    .unwrap();
    let (l, system, c) = catalog(&env);
    let err = l.start_self_update(&c, true, &mut |_, _| {}).unwrap_err();
    assert!(
        matches!(err, CoreError::Update(UpdateError::ChecksumMismatch)),
        "{err:?}"
    );
    assert!(system.started.lock().unwrap().is_empty());
    let left = std::fs::read_dir(env.data_dir.path().join("downloads"))
        .map(|d| d.count())
        .unwrap_or(0);
    assert_eq!(left, 0);
}

#[test]
fn zweites_selbst_update_gleichzeitig_wird_gesperrt() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, _, c) = catalog(&env);
    let _held = crate::lock::Lock::acquire(&env.data_dir.path().join("locks"), "launcher").unwrap();
    assert!(matches!(
        l.start_self_update(&c, true, &mut |_, _| {}),
        Err(CoreError::Busy(_))
    ));
}

#[test]
fn reste_frueherer_selbst_updates_werden_aufgeraeumt() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, _) = catalog(&env);
    let downloads = env.data_dir.path().join("downloads");
    std::fs::create_dir_all(&downloads).unwrap();
    std::fs::write(downloads.join("launcher-0.2.0.exe"), b"alt").unwrap();
    std::fs::write(downloads.join("desk-1.0.0.exe.part"), b"andere").unwrap();
    l.clean_self_update_leftovers();
    assert!(!downloads.join("launcher-0.2.0.exe").exists());
    assert!(downloads.join("desk-1.0.0.exe.part").exists());
}
