use crate::apps::{find, APPS, LAUNCHER, LAUNCHER_ID, OWNER_ID};
use daedyx_update::policy::Platform;

#[test]
fn every_app_has_a_valid_fixed_source() {
    for app in APPS.iter().chain(std::iter::once(&LAUNCHER)) {
        let p = app.policy_for(Platform::WindowsX64).unwrap();
        assert_eq!(
            (p.owner(), p.owner_id()),
            ("daedyx", OWNER_ID),
            "{}",
            app.id
        );
        assert_eq!(
            (p.repo(), p.repo_id(), p.app_id()),
            (app.repo, app.repo_id, app.update_app)
        );
        assert!(p
            .latest_asset_url("entry.json")
            .starts_with("https://github.com/daedyx/"));
    }
}

#[test]
fn the_ids_read_once_with_gh_api_stay_fixed() {
    let ids: Vec<(&str, u64)> = APPS.iter().map(|a| (a.repo, a.repo_id)).collect();
    assert_eq!(ids, [("desk", 1_475_265_095), ("tasks", 1_474_858_832)]);
    assert_eq!(
        (LAUNCHER.repo, LAUNCHER.repo_id),
        ("launcher", 1_474_858_940)
    );
    assert_eq!(OWNER_ID, 356_409_479);
}

#[test]
fn the_launcher_uses_the_product_ids_of_the_apps_never_the_preview() {
    let apps: Vec<&str> = APPS.iter().map(|a| a.update_app).collect();
    assert_eq!(apps, ["daedyx-desk", "daedyx-tasks"]);
    assert!(APPS.iter().all(|a| !a.update_app.contains("vorschau")));
}

#[test]
fn ids_are_unique_and_found() {
    assert_eq!(find("desk").map(|a| a.repo), Some("desk"));
    assert_eq!(find(LAUNCHER_ID), Some(&LAUNCHER));
    assert!(find("board").is_none() && find("").is_none());
    let mut ids: Vec<&str> = APPS.iter().map(|a| a.id).chain([LAUNCHER_ID]).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), APPS.len() + 1);
}
