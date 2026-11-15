use crate::error::UpdateError;
use crate::version::{is_newer, parse};

#[test]
fn zehn_ist_neuer_als_neun() {
    assert_eq!(is_newer("0.10.0", "0.9.0"), Ok(true));
}

#[test]
fn gleiche_und_aeltere_version_ist_nicht_neuer() {
    assert_eq!(is_newer("1.2.3", "1.2.3"), Ok(false));
    assert_eq!(is_newer("1.2.2", "1.2.3"), Ok(false));
}

#[test]
fn ungueltige_version_wird_gemeldet() {
    assert_eq!(parse("abc"), Err(UpdateError::BadVersion("abc".into())));
}
