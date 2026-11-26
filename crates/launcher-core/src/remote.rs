//! Update-Quelle des Launchers (U3): dieselbe Prüfung wie in den Apps (`daedyx-update`, Kern). Je App wird
//! `entry.json` mit GitHub-Nachweis geholt und geprüft; das Setup lädt erst ein Auftrag (Klick in der Oberfläche oder
//! Befehl auf der Kommandozeile). Netz und Nachweis-Prüfer sind austauschbar, damit die Tests ohne Netz laufen.
use crate::sync::lock_or_recover;
use daedyx_update::attest::{AttestError, GithubAttestation, Provenance};
use daedyx_update::check::{self, Offer, Outcome};
use daedyx_update::fetch::Transport;
use daedyx_update::net::UreqTransport;
use daedyx_update::policy::UpdatePolicy;
use daedyx_update::reason::Blocked;
use daedyx_update::version::Version;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

/// Prüfer für den Nachweis einer Quelle.
pub type MakeProvenance =
    Box<dyn Fn(&UpdatePolicy) -> Result<Box<dyn Provenance>, AttestError> + Send + Sync>;

/// Netz und Nachweis.
pub struct Remote {
    transport: Mutex<Box<dyn Transport + Send>>,
    provenance: MakeProvenance,
}

impl Remote {
    /// Echtes Netz, echter GitHub-Nachweis mit der eingebauten Wurzel.
    pub fn real() -> Remote {
        Remote::new(
            Box::new(UreqTransport::default()),
            Box::new(|p| Ok(Box::new(GithubAttestation::new(p)?) as Box<dyn Provenance>)),
        )
    }

    pub fn new(transport: Box<dyn Transport + Send>, provenance: MakeProvenance) -> Remote {
        Remote {
            transport: Mutex::new(transport),
            provenance,
        }
    }

    /// Sucht für eine Quelle: geprüfter Eintrag, neuer als `installed` und nicht älter als `seen`.
    pub fn check(
        &self,
        policy: &UpdatePolicy,
        installed: &Version,
        seen: Option<&Version>,
    ) -> Outcome {
        let provenance = match (self.provenance)(policy) {
            Ok(p) => p,
            Err(e) => return Outcome::Blocked(e.into()),
        };
        let mut transport = lock_or_recover(&self.transport);
        check::check(
            transport.as_mut(),
            provenance.as_ref(),
            policy,
            installed,
            seen,
        )
    }

    /// Lädt das Setup eines geprüften Angebots nach `dir`: Größe und SHA-256 aus dem Eintrag, Subjekt im Bündel.
    pub fn fetch(
        &self,
        policy: &UpdatePolicy,
        offer: &Offer,
        dir: &Path,
        progress: &mut dyn FnMut(u64, u64),
    ) -> Result<PathBuf, Blocked> {
        let provenance = (self.provenance)(policy)?;
        let mut transport = lock_or_recover(&self.transport);
        check::fetch_setup(
            transport.as_mut(),
            provenance.as_ref(),
            policy,
            offer,
            dir,
            None,
            progress,
        )
    }
}
