//! Netz ohne Netz geprüft: Einstellungen des Agenten, Fehlerzuordnung und Antwortarten (wie `daedyx-updater`).
use crate::net::{config, kind, map_error, Kind, CONNECT_TIMEOUT, IDLE_TIMEOUT, USER_AGENT};
use daedyx_update::fetch::TransportError;
use std::io;

#[test]
fn the_transport_never_follows_a_redirect_itself() {
    let c = config();
    assert_eq!(
        c.max_redirects(),
        0,
        "Weiterleitungen prüft der Kern je Schritt"
    );
    assert!(c.https_only());
    assert!(
        !c.http_status_as_error(),
        "404/500 kommen als Status zurück"
    );
    assert!(!c.save_redirect_history());
}

#[test]
fn time_limits_and_user_agent_are_fixed() {
    let c = config();
    let t = c.timeouts();
    assert_eq!(c.timeout_per_read(), Some(IDLE_TIMEOUT));
    assert_eq!(
        (IDLE_TIMEOUT.as_secs(), CONNECT_TIMEOUT.as_secs()),
        (60, 15)
    );
    assert_eq!(
        (t.connect, t.resolve, t.recv_response),
        (
            Some(CONNECT_TIMEOUT),
            Some(CONNECT_TIMEOUT),
            Some(CONNECT_TIMEOUT)
        )
    );
    assert_eq!(
        (t.global, t.recv_body),
        (None, None),
        "keine Gesamtgrenze, damit langsame Leitungen klappen"
    );
    assert_eq!(USER_AGENT, "daedyx-update");
}

#[test]
fn a_redirect_needs_a_location() {
    let to = "https://objects.githubusercontent.com/x";
    assert_eq!(kind(302, Some(to)), Kind::Redirect(to.to_owned()));
    assert_eq!(kind(301, None), Kind::Status(301));
    assert_eq!(kind(200, Some(to)), Kind::Body);
    assert_eq!(kind(404, None), Kind::Status(404));
    assert_eq!(kind(500, Some(to)), Kind::Status(500));
    assert_eq!(kind(101, None), Kind::Status(101));
}

#[test]
fn errors_map_to_offline_timeout_or_failed() {
    use ureq::Error as E;
    assert_eq!(
        map_error(&E::Timeout(ureq::Timeout::Connect)),
        TransportError::Timeout
    );
    assert_eq!(map_error(&E::HostNotFound), TransportError::Offline);
    assert_eq!(map_error(&E::ConnectionFailed), TransportError::Offline);
    let io = |k| E::Io(io::Error::from(k));
    assert_eq!(
        map_error(&io(io::ErrorKind::ConnectionRefused)),
        TransportError::Offline
    );
    assert_eq!(
        map_error(&io(io::ErrorKind::TimedOut)),
        TransportError::Timeout
    );
    assert_eq!(
        map_error(&io(io::ErrorKind::InvalidData)),
        TransportError::Failed
    );
    assert_eq!(map_error(&E::BadUri("x".into())), TransportError::Failed);
}
