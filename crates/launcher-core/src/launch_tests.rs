use crate::error::CoreError;
use crate::launch::launch_target;
use crate::test_support::Env;

fn app_dir_with(json: &str, file: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("app.json"), json).unwrap();
    if let Some(f) = file {
        let p = dir.path().join(f);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"x").unwrap();
    }
    dir
}

#[test]
fn startprogramm_im_installationsordner_wird_gefunden() {
    let dir = app_dir_with(
        r#"{"version":"1.0.0","exe":"bin/app.exe"}"#,
        Some("bin/app.exe"),
    );
    assert_eq!(
        launch_target(dir.path()).unwrap(),
        dir.path().join("bin/app.exe")
    );
}

#[test]
fn gefaehrliche_oder_fehlende_startprogramme_werden_abgelehnt() {
    for exe in [
        "../x.exe",
        "..\\x.exe",
        "C:\\Windows\\notepad.exe",
        "\\\\srv\\s\\x.exe",
        "/etc/x",
        "",
        "a:b.exe",
    ] {
        let json = format!(r#"{{"version":"1.0.0","exe":{}}}"#, serde_json::json!(exe));
        let dir = app_dir_with(&json, None);
        assert!(
            matches!(launch_target(dir.path()), Err(CoreError::NotLaunchable(_))),
            "{exe:?} hätte abgelehnt werden müssen"
        );
    }
    let ohne_feld = app_dir_with(r#"{"version":"1.0.0"}"#, None);
    assert!(matches!(
        launch_target(ohne_feld.path()),
        Err(CoreError::NotLaunchable(_))
    ));
    let datei_fehlt = app_dir_with(r#"{"version":"1.0.0","exe":"app.exe"}"#, None);
    assert!(matches!(
        launch_target(datei_fehlt.path()),
        Err(CoreError::NotLaunchable(_))
    ));
}

#[test]
fn nicht_installierte_app_laesst_sich_nicht_oeffnen() {
    let env = Env::new();
    let (l, _) = env.launcher();
    assert!(matches!(l.open("desk"), Err(CoreError::NotInstalled(_))));
    assert!(matches!(l.open("../x"), Err(CoreError::Update(_))));
}

#[cfg(windows)]
#[test]
fn installierte_app_wird_gestartet() {
    let env = Env::new();
    let dir = env.data_dir.path().join("apps/desk");
    std::fs::create_dir_all(&dir).unwrap();
    let system =
        std::path::PathBuf::from(std::env::var("SystemRoot").unwrap()).join("System32\\whoami.exe");
    std::fs::copy(system, dir.join("app.exe")).unwrap();
    std::fs::write(
        dir.join("app.json"),
        r#"{"version":"1.0.0","exe":"app.exe"}"#,
    )
    .unwrap();
    let (l, _) = env.launcher();
    l.open("desk").unwrap();
}
