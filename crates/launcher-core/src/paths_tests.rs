use crate::paths::default_data_dir;

#[test]
fn datenordner_endet_auf_daedyx_launcher() {
    let p = default_data_dir();
    assert!(
        p.ends_with("Daedyx/Launcher") || p.ends_with(r"Daedyx\Launcher"),
        "{p:?}"
    );
}
