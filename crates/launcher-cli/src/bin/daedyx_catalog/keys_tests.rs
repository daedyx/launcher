use crate::keys::{generate, load_public, load_secret, sign};
use daedyx_update::verify::verify_signature;

#[test]
fn schluessel_ohne_passwort_signiert_und_die_pruefung_klappt() {
    let (public, secret) = generate(None).unwrap();
    let sk = load_secret(&secret, || panic!("kein Passwort erwartet")).unwrap();
    let pk = load_public(&public).unwrap();
    let sig = sign(&sk, Some(&pk), b"daten").unwrap();
    assert!(verify_signature(&public, b"daten", &sig).is_ok());
    assert!(verify_signature(&public, b"daten!", &sig).is_err());
}

#[test]
fn verschluesselter_schluessel_fragt_einmal_nach_dem_passwort() {
    let (public, secret) = generate(Some("geheim".into())).unwrap();
    let mut asked = 0;
    let sk = load_secret(&secret, || {
        asked += 1;
        Ok("geheim".to_string())
    })
    .unwrap();
    assert_eq!(asked, 1);
    let sig = sign(&sk, None, b"daten").unwrap();
    assert!(verify_signature(&public, b"daten", &sig).is_ok());
}

#[test]
fn falsches_passwort_und_fremde_dateien_werden_abgelehnt() {
    let (_, secret) = generate(Some("geheim".into())).unwrap();
    assert!(load_secret(&secret, || Ok("falsch".to_string())).is_err());
    assert!(load_secret("kein schlüssel", || Ok(String::new())).is_err());
    assert!(load_public("kein schlüssel").is_err());
}
