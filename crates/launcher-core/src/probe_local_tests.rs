//! Installationsprobe (nur Merkmal `probe`): lokales Setup über den gesperrten Griff, echte Schalter, Zustand.
use crate::service::Outcome;
use crate::state::State;
use crate::test_support::Env;

fn setup(env: &Env, text: &str) -> std::io::Result<std::path::PathBuf> {
    let p = env.local_dir.path().join("probe-setup.exe");
    std::fs::write(&p, text)?;
    Ok(p)
}

#[test]
fn a_local_setup_is_installed_as_app_and_remembered() {
    let env = Env::new();
    let (l, installer) = env.launcher();
    let out = l
        .install_local("desk", &setup(&env, "0.0.1").unwrap())
        .unwrap();
    assert_eq!(
        out,
        Outcome::Installed {
            version: "0.0.1".into()
        }
    );
    assert_eq!(
        State::load(env.data_dir.path()).unwrap().apps["desk"].version,
        "0.0.1"
    );
    assert_eq!(
        installer.calls.lock().unwrap().as_slice(),
        ["install 0.0.1"]
    );
    let own = env.data_dir.path().join("downloads").join("daedyx-desk");
    assert_eq!(
        std::fs::read_dir(own).unwrap().count(),
        0,
        "Kopie danach gelöscht"
    );
    let out = l
        .install_local("desk", &setup(&env, "0.0.2").unwrap())
        .unwrap();
    assert_eq!(
        out,
        Outcome::Updated {
            from: "0.0.1".into(),
            to: "0.0.2".into()
        }
    );
    assert!(l
        .install_local("board", &setup(&env, "x").unwrap())
        .is_err());
}

#[test]
fn a_local_self_update_starts_silently_without_relaunch() {
    let env = Env::new();
    let (l, _, system) = env.launcher_with_system();
    l.self_update_local(&setup(&env, "neu").unwrap()).unwrap();
    let started = system.started.lock().unwrap();
    assert_eq!(started[0].1, ["/S", "/UPDATE", "/NOLAUNCH"]);
    assert!(started[0].0.starts_with(
        env.data_dir
            .path()
            .join("downloads")
            .join("daedyx-launcher")
    ));
}

#[cfg(windows)]
#[test]
fn the_probe_also_holds_the_setup_locked() {
    let env = Env::new();
    let (l, installer, system) = env.launcher_with_system();
    l.install_local("desk", &setup(&env, "0.0.1").unwrap())
        .unwrap();
    l.self_update_local(&setup(&env, "neu").unwrap()).unwrap();
    assert_eq!(installer.writable.lock().unwrap().as_slice(), [false]);
    assert_eq!(system.writable.lock().unwrap().as_slice(), [false]);
}
