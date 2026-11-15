use crate::error::CoreError;
use crate::jobs::{percent, Event, Job, Wake, Worker};
use crate::service::Outcome;
use crate::test_support::Env;
use daedyx_update::error::UpdateError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn start_with(env: &Env, pubkey: &str, wake: Wake) -> Worker {
    let (l, _) = env.launcher();
    Worker::start(l, pubkey.to_string(), wake)
}

fn start(env: &Env) -> Worker {
    start_with(env, &env.pubkey, Arc::new(|| {}))
}

fn next(w: &Worker) -> Event {
    w.recv_timeout(Duration::from_secs(5))
        .expect("Ereignis erwartet")
}

fn refreshed(env: &Env) -> Worker {
    let w = start(env);
    w.send(Job::Refresh);
    assert!(matches!(next(&w), Event::Catalog(Ok(_))));
    assert!(matches!(next(&w), Event::Status(_)));
    w
}

#[test]
fn prozent_wird_berechnet() {
    assert_eq!(percent(50, Some(200)), Some(25));
    assert_eq!(percent(300, Some(200)), Some(100));
    assert_eq!(percent(5, None), None);
    assert_eq!(percent(5, Some(0)), None);
}

#[test]
fn aktualisieren_meldet_katalog_und_status() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = start(&env);
    w.send(Job::Refresh);
    match next(&w) {
        Event::Catalog(Ok(c)) => assert_eq!(c.apps.len(), 1),
        other => panic!("{other:?}"),
    }
    match next(&w) {
        Event::Status(s) => assert_eq!(s[0].installed, None),
        other => panic!("{other:?}"),
    }
}

#[test]
fn installieren_meldet_fortschritt_ergebnis_und_status() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = refreshed(&env);
    w.send(Job::Install("desk".into()));
    let mut saw_full_progress = false;
    loop {
        match next(&w) {
            Event::Progress { done, total, .. } => saw_full_progress |= Some(done) == total,
            Event::Installed { app, result } => {
                assert_eq!(app, "desk");
                assert_eq!(
                    result.unwrap(),
                    Outcome::Installed {
                        version: "1.0.0".into()
                    }
                );
                break;
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(saw_full_progress);
    match next(&w) {
        Event::Status(s) => assert_eq!(s[0].installed.as_deref(), Some("1.0.0")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn fremder_schluessel_meldet_fehler_und_installieren_scheitert_sauber() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let other = Env::new();
    let w = start_with(&env, &other.pubkey, Arc::new(|| {}));
    w.send(Job::Refresh);
    assert!(matches!(
        next(&w),
        Event::Catalog(Err(CoreError::Update(UpdateError::SignatureInvalid)))
    ));
    w.send(Job::Install("desk".into()));
    assert!(matches!(next(&w), Event::Catalog(Err(_))));
    assert!(matches!(next(&w), Event::Installed { result: Err(_), .. }));
}

#[test]
fn entfernen_meldet_ergebnis_und_status() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = refreshed(&env);
    w.send(Job::Install("desk".into()));
    while !matches!(next(&w), Event::Status(_)) {}
    w.send(Job::Remove("desk".into()));
    assert!(matches!(next(&w), Event::Removed { result: Ok(()), .. }));
    match next(&w) {
        Event::Status(s) => assert_eq!(s[0].installed, None),
        other => panic!("{other:?}"),
    }
}

#[test]
fn oeffnen_einer_nicht_installierten_app_meldet_fehler() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = refreshed(&env);
    w.send(Job::Open("desk".into()));
    assert!(matches!(
        next(&w),
        Event::Opened {
            result: Err(CoreError::NotInstalled(_)),
            ..
        }
    ));
}

#[test]
fn jedes_ereignis_weckt_die_oberflaeche() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    let w = start_with(
        &env,
        &env.pubkey,
        Arc::new(move || {
            c.fetch_add(1, Ordering::SeqCst);
        }),
    );
    w.send(Job::Refresh);
    next(&w);
    next(&w);
    assert!(count.load(Ordering::SeqCst) >= 2);
}

#[test]
fn laeuft_pruefung_meldet_laufende_apps() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _, system) = env.launcher_with_system();
    let w = Worker::start(l, env.pubkey.clone(), Arc::new(|| {}));
    w.send(Job::Refresh);
    next(&w);
    next(&w);
    w.send(Job::Install("desk".into()));
    while !matches!(next(&w), Event::Status(_)) {}
    w.send(Job::CheckRunning);
    assert!(matches!(next(&w), Event::Running(ids) if ids.is_empty()));
    system
        .running
        .lock()
        .unwrap()
        .push(env.data_dir.path().join("apps/desk"));
    w.send(Job::CheckRunning);
    assert!(matches!(next(&w), Event::Running(ids) if ids == ["desk"]));
}

#[test]
fn refresh_meldet_ein_launcher_update_und_selbst_update_startet_das_setup() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.2.0"], None);
    let (l, _, system) = env.launcher_with_system();
    let w = Worker::start(l, env.pubkey.clone(), Arc::new(|| {}));
    w.send(Job::Refresh);
    let update = loop {
        if let Event::SelfUpdate(u) = next(&w) {
            break u;
        }
    };
    assert_eq!(update.map(|u| u.version), Some("0.2.0".to_string()));
    w.send(Job::UpdateSelf);
    loop {
        match next(&w) {
            Event::Progress { app, .. } => assert_eq!(app, "launcher"),
            Event::SelfUpdateStarted(result) => {
                assert_eq!(
                    result.unwrap().map(|u| u.version),
                    Some("0.2.0".to_string())
                );
                break;
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(system.started.lock().unwrap().len(), 1);
}
