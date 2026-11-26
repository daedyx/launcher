//! Gefälschter Server und Attrappe für den Nachweis (aus `test_support.rs` gelöst, Review U3 F-U3-9).
use daedyx_update::attest::{AttestError, AttestKind, Provenance};
use daedyx_update::fetch::{HttpsUrl, Reply, Transport, TransportError};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Gefälschter Server: Adresse zu Inhalt; jede Anfrage wird gezählt.
#[derive(Clone, Default)]
pub(crate) struct Server {
    pub(crate) routes: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    pub(crate) requests: Arc<Mutex<Vec<String>>>,
    pub(crate) offline: Arc<AtomicBool>,
}

impl Transport for Server {
    fn get(&mut self, url: &HttpsUrl) -> Result<Reply<'_>, TransportError> {
        self.requests.lock().unwrap().push(url.as_str().to_string());
        if self.offline.load(Ordering::SeqCst) {
            return Err(TransportError::Offline);
        }
        Ok(
            match self.routes.lock().unwrap().get(url.as_str()).cloned() {
                Some(b) => Reply::Body {
                    length: Some(b.len() as u64),
                    reader: Box::new(std::io::Cursor::new(b)),
                },
                None => Reply::Status(404),
            },
        )
    }
}

/// Attrappe für den Nachweis: nimmt alles an, solange `trusted` gilt (die Prüfung selbst testet der Kern).
pub(crate) struct Attrappe(pub(crate) Arc<AtomicBool>);

impl Provenance for Attrappe {
    fn verify(&self, _: &[u8; 32], _: &[u8]) -> Result<(), AttestError> {
        if self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(AttestError {
                kind: AttestKind::Invalid,
                detail: "test: not trusted".into(),
            })
        }
    }
}

pub(crate) fn sha_hex(b: &[u8]) -> String {
    Sha256::digest(b)
        .iter()
        .map(|x| format!("{x:02x}"))
        .collect()
}
