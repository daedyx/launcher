//! Suche über alle Apps (U3): nur die festen Quellen, kein Setup ohne Auftrag, höchste gesehene Version im eigenen
//! Datenordner, ohne Netz bleiben die Apps sichtbar.
use crate::lineup::Found;
use crate::test_support::Env;
use daedyx_update::reason::BlockKind;
use daedyx_update::state::UpdateState;

#[test]
fn the_search_asks_only_the_fixed_sources_and_loads_no_setup() {
    let env = Env::new();
    env.publish_apps(&["1.0.0"], &["0.3.0"], None);
    let (l, installer) = env.launcher();
    let lineup = l.load_lineup();
    assert!(matches!(lineup.get("desk").unwrap().found, Found::Offer(_)));
    assert_eq!(
        lineup.get("tasks").unwrap().blocked().map(|b| b.kind),
        Some(BlockKind::SourceUnavailable)
    );
    let urls = env.server.requests.lock().unwrap().clone();
    let allowed = [
        "https://github.com/daedyx/desk/releases/latest/download/",
        "https://github.com/daedyx/tasks/releases/latest/download/",
        "https://github.com/daedyx/launcher/releases/latest/download/",
    ];
    assert!(
        urls.iter()
            .all(|u| allowed.iter().any(|a| u.starts_with(a))),
        "{urls:?}"
    );
    assert!(
        !urls.iter().any(|u| u.ends_with("-setup.exe")),
        "Setup erst nach Auftrag"
    );
    assert!(installer.calls.lock().unwrap().is_empty());
}

#[test]
fn the_offer_carries_date_and_plain_notes() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let (l, _) = env.launcher();
    let latest = l.load_lineup().get("desk").unwrap().latest().unwrap();
    assert_eq!(
        (latest.version.as_str(), latest.date.as_str()),
        ("1.0.0", "2026-11-13")
    );
    assert_eq!(latest.notes, "Neu in 1.0.0.");
}

#[test]
fn the_highest_seen_version_stays_in_the_launchers_own_folder() {
    let env = Env::new();
    env.publish(&["1.2.0"]);
    let (l, _) = env.launcher();
    let _ = l.load_lineup();
    let seen = UpdateState::load(&env.data_dir.path().join("seen").join("desk")).unwrap();
    assert_eq!(
        seen.highest_seen().map(|v| v.to_string()).as_deref(),
        Some("1.2.0")
    );
    // Nie im lokalen Ordner der Apps oder neben der Sperre.
    assert!(!env.local_dir.path().join("de.daedyx.desk").exists());
    // Springt die Quelle danach zurück, gibt es kein Angebot, auch für eine nicht installierte App.
    env.publish(&["1.1.0"]);
    assert_eq!(
        l.load_lineup()
            .get("desk")
            .unwrap()
            .blocked()
            .map(|b| b.kind),
        Some(BlockKind::SourceWentBack)
    );
}

#[test]
fn without_network_the_whole_search_reports_offline() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    env.server
        .offline
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let (l, _) = env.launcher();
    let lineup = l.load_lineup();
    assert_eq!(lineup.problem(), Some(BlockKind::Offline));
    assert_eq!(lineup.apps.len(), 2, "die Apps bleiben sichtbar");
    assert!(lineup.apps.iter().all(|a| a.latest_version().is_none()));
}
