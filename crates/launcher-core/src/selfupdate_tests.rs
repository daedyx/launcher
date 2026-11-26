//! Selbst-Update mit gefälschtem Server (U3): Quelle ist das eigene Repo `daedyx/launcher`, nur eine echt höhere
//! geprüfte Version, Setup still gestartet, bei jedem Fehler bleibt nichts liegen. Der Test-Launcher ist 0.2.0.
use crate::error::CoreError;
use crate::lineup::Lineup;
use crate::selfupdate::SelfUpdate;
use crate::test_support::Env;
use daedyx_update::lock::Lock;
use daedyx_update::reason::BlockKind;

fn lineup(
    env: &Env,
) -> (
    crate::service::Launcher,
    crate::test_support::FakeSystem,
    Lineup,
) {
    let (l, _, system) = env.launcher_with_system();
    let c = l.load_lineup();
    (l, system, c)
}

fn left_in_downloads(env: &Env) -> usize {
    std::fs::read_dir(
        env.data_dir
            .path()
            .join("downloads")
            .join("daedyx-launcher"),
    )
    .map(|d| d.count())
    .unwrap_or(0)
}

#[test]
fn without_a_launcher_release_there_is_no_update() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, c) = lineup(&env);
    assert_eq!(l.self_update_available(&c), None);
    assert_eq!(
        c.launcher.blocked().map(|b| b.kind),
        Some(BlockKind::SourceUnavailable)
    );
}

#[test]
fn only_a_higher_verified_version_is_offered() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, _, c) = lineup(&env);
    let want = SelfUpdate {
        version: "0.3.0".into(),
        notes: "Neu in 0.3.0.".into(),
    };
    assert_eq!(l.self_update_available(&c), Some(want));
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, _, c) = lineup(&env);
    assert_eq!(
        l.self_update_available(&c),
        None,
        "gleiche Version: keine Schleife"
    );
}

#[test]
fn the_launcher_is_not_listed_as_an_app() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, _, c) = lineup(&env);
    let ids: Vec<_> = l.status(&c).unwrap().into_iter().map(|s| s.id).collect();
    assert_eq!(ids, ["desk", "tasks"]);
}

#[test]
fn the_self_update_loads_verified_and_starts_the_setup_silently() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, system, c) = lineup(&env);
    let started = l.start_self_update(&c, true, &mut |_, _| {}).unwrap();
    assert_eq!(started.map(|s| s.version), Some("0.3.0".to_string()));
    let calls = system.started.lock().unwrap();
    assert_eq!(calls.len(), 1);
    let (program, args) = &calls[0];
    assert!(program.starts_with(
        env.data_dir
            .path()
            .join("downloads")
            .join("daedyx-launcher")
    ));
    assert!(program.ends_with("daedyx-launcher-0.3.0-setup.exe"));
    assert!(program.exists(), "das Setup muss bis zum Start da sein");
    assert_eq!(args, &["/S", "/UPDATE"]);
}

#[test]
fn without_relaunch_the_nolaunch_switch_is_passed() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, system, c) = lineup(&env);
    l.start_self_update(&c, false, &mut |_, _| {}).unwrap();
    assert_eq!(
        system.started.lock().unwrap()[0].1,
        ["/S", "/UPDATE", "/NOLAUNCH"]
    );
}

#[test]
fn without_a_newer_version_nothing_happens() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, system, c) = lineup(&env);
    assert_eq!(l.start_self_update(&c, true, &mut |_, _| {}).unwrap(), None);
    assert!(system.started.lock().unwrap().is_empty());
}

#[test]
fn a_changed_setup_starts_nothing_and_leaves_nothing() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, system, c) = lineup(&env);
    // Eintrag geprüft, aber das Setup wird danach auf dem Server verändert.
    let url = "https://github.com/daedyx/launcher/releases/download/v0.3.0/daedyx-launcher-0.3.0-setup.exe";
    env.server
        .routes
        .lock()
        .unwrap()
        .insert(url.to_string(), b"0.3.1".to_vec());
    let err = l.start_self_update(&c, true, &mut |_, _| {}).unwrap_err();
    assert!(
        matches!(&err, CoreError::Blocked { blocked, .. } if blocked.kind == BlockKind::NotTrusted),
        "{err:?}"
    );
    assert!(system.started.lock().unwrap().is_empty());
    assert_eq!(left_in_downloads(&env), 0);
}

#[test]
fn a_second_self_update_at_the_same_time_is_locked() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, system, c) = lineup(&env);
    let _held = Lock::acquire(&env.lock_dir(), "daedyx-launcher").unwrap();
    assert!(matches!(
        l.start_self_update(&c, true, &mut |_, _| {}),
        Err(CoreError::Busy(_))
    ));
    assert!(system.started.lock().unwrap().is_empty());
}

#[test]
fn a_launcher_release_for_a_newer_launcher_is_not_loaded() {
    let env = Env::new();
    env.release(&crate::apps::LAUNCHER, "0.3.0", false, Some("0.2.5"));
    let (l, system, c) = lineup(&env);
    assert!(matches!(
        l.start_self_update(&c, true, &mut |_, _| {}),
        Err(CoreError::LauncherTooOld { .. })
    ));
    assert!(system.started.lock().unwrap().is_empty());
}

#[test]
fn leftovers_are_cleaned_per_app_but_not_foreign_files() {
    let env = Env::new();
    let (l, _, _) = lineup(&env);
    let downloads = env.data_dir.path().join("downloads");
    let (me, desk) = (
        downloads.join("daedyx-launcher"),
        downloads.join("daedyx-desk"),
    );
    std::fs::create_dir_all(&me).unwrap();
    std::fs::create_dir_all(&desk).unwrap();
    std::fs::write(me.join("daedyx-launcher-0.3.0-setup.exe"), b"alt").unwrap();
    std::fs::write(
        desk.join("daedyx-desk-1.0.0-setup.exe.part"),
        b"abgebrochen",
    )
    .unwrap();
    std::fs::write(desk.join("notiz.txt"), b"fremd").unwrap();
    l.clean_self_update_leftovers();
    assert!(!me.join("daedyx-launcher-0.3.0-setup.exe").exists());
    assert!(!desk.join("daedyx-desk-1.0.0-setup.exe.part").exists());
    assert!(desk.join("notiz.txt").exists());
}

#[test]
fn a_running_download_of_a_second_launcher_is_never_cleaned() {
    let env = Env::new();
    let (l, _, _) = lineup(&env);
    let desk = env.data_dir.path().join("downloads").join("daedyx-desk");
    std::fs::create_dir_all(&desk).unwrap();
    let part = desk.join("daedyx-desk-1.0.0-setup.exe.part");
    std::fs::write(&part, b"laedt gerade").unwrap();
    // Ein zweiter Launcher (oder Desk selbst) lädt gerade unter der Sperre der App (Review-Probe N-U3-1).
    let held = Lock::acquire(&env.lock_dir(), "daedyx-desk").unwrap();
    l.clean_self_update_leftovers();
    assert!(part.exists(), "fremder laufender Download bleibt");
    drop(held);
    l.clean_self_update_leftovers();
    assert!(!part.exists());
}

#[cfg(windows)]
#[test]
fn the_setup_is_locked_against_writing_while_it_starts() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, system, c) = lineup(&env);
    l.start_self_update(&c, true, &mut |_, _| {}).unwrap();
    assert_eq!(
        system.writable.lock().unwrap().as_slice(),
        [false],
        "M-U3-1"
    );
}
