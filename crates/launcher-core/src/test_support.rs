//! Gemeinsame Test-Hilfen: gefälschter Server ohne Netz, Attrappe für den Nachweis, Test-Installer (U3).
use crate::error::CoreError;
use crate::installer::Installer;
use crate::remote::Remote;
use crate::service::Launcher;
use crate::system::{SystemInfo, UninstallEntry};
use daedyx_update::attest::{AttestError, AttestKind, Provenance};
use daedyx_update::entry::PackageKind;
use daedyx_update::fetch::{HttpsUrl, Reply, Transport, TransportError};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Attrappe: Das „Paket“ ist ein Text mit der Versionsnummer; die Installation schreibt `app.json`.
#[derive(Clone, Default)]
pub(crate) struct TestInstaller {
    pub(crate) calls: Arc<Mutex<Vec<String>>>,
}

impl Installer for TestInstaller {
    fn install(&self, package: &Path, _kind: PackageKind, dir: &Path) -> Result<(), CoreError> {
        let version = std::fs::read_to_string(package)?;
        std::fs::create_dir_all(dir)?;
        std::fs::write(
            dir.join("app.json"),
            format!(r#"{{"version":"{version}"}}"#),
        )?;
        self.calls
            .lock()
            .unwrap()
            .push(format!("install {version}"));
        Ok(())
    }
    fn uninstall(&self, dir: &Path) -> Result<(), CoreError> {
        std::fs::remove_dir_all(dir)?;
        self.calls.lock().unwrap().push("uninstall".into());
        Ok(())
    }
}

/// Gestartetes Programm: Pfad und Argumente.
pub(crate) type StartedProgram = (PathBuf, Vec<String>);

/// Attrappe des Systems: Registry-Einträge und „laufende“ Ordner lassen sich vorgeben.
#[derive(Clone, Default)]
pub(crate) struct FakeSystem {
    pub(crate) entries: Arc<Mutex<BTreeMap<String, UninstallEntry>>>,
    pub(crate) running: Arc<Mutex<Vec<PathBuf>>>,
    /// Gestartete Programme (Pfad, Argumente)
    pub(crate) started: Arc<Mutex<Vec<StartedProgram>>>,
}

impl SystemInfo for FakeSystem {
    fn uninstall_entry(&self, key: &str) -> Option<UninstallEntry> {
        self.entries.lock().unwrap().get(key).cloned()
    }
    fn running(&self, dirs: &[PathBuf]) -> Vec<bool> {
        let running = self.running.lock().unwrap();
        dirs.iter().map(|d| running.contains(d)).collect()
    }
    fn start_detached(&self, program: &Path, args: &[&str]) -> Result<(), String> {
        self.started.lock().unwrap().push((
            program.to_path_buf(),
            args.iter().map(|a| a.to_string()).collect(),
        ));
        Ok(())
    }
}

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
struct Attrappe(Arc<AtomicBool>);

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

pub(crate) struct Env {
    pub(crate) server: Server,
    pub(crate) trusted: Arc<AtomicBool>,
    pub(crate) data_dir: tempfile::TempDir,
    pub(crate) local_dir: tempfile::TempDir,
}

impl Env {
    pub(crate) fn new() -> Env {
        Env {
            server: Server::default(),
            trusted: Arc::new(AtomicBool::new(true)),
            data_dir: tempfile::tempdir().unwrap(),
            local_dir: tempfile::tempdir().unwrap(),
        }
    }

    pub(crate) fn publish(&self, versions: &[&str]) {
        self.publish_with(versions, None);
    }

    /// Veröffentlicht die höchste der Versionen von `desk`. `wrong_sha`: für diese Version steht im (geprüften)
    /// Eintrag absichtlich eine falsche Prüfsumme.
    pub(crate) fn publish_with(&self, versions: &[&str], wrong_sha: Option<&str>) {
        self.publish_apps(versions, &[], wrong_sha);
    }

    /// Veröffentlicht je App die höchste Version als „latest“ (`entry.json`, Bündel, Setup). Das Setup ist ein Text
    /// mit der Versionsnummer.
    pub(crate) fn publish_apps(&self, desk: &[&str], launcher: &[&str], wrong_sha: Option<&str>) {
        for (app, versions) in [
            (crate::apps::find("desk").unwrap(), desk),
            (&crate::apps::LAUNCHER, launcher),
        ] {
            let Some(v) = versions
                .iter()
                .max_by_key(|v| daedyx_update::version::parse(v).unwrap())
            else {
                continue;
            };
            self.release(app, v, wrong_sha == Some(*v), None);
        }
    }

    /// Ein Release einer App als „latest“; `min_launcher`: Feld `minLauncherVersion`.
    pub(crate) fn release(
        &self,
        app: &crate::apps::KnownApp,
        v: &str,
        wrong_sha: bool,
        min_launcher: Option<&str>,
    ) {
        let p = app.policy().unwrap();
        let url = format!(
            "https://github.com/daedyx/{}/releases/download/v{v}/{}-{v}-setup.exe",
            app.repo, app.update_app
        );
        let sha = if wrong_sha {
            "00".repeat(32)
        } else {
            sha_hex(v.as_bytes())
        };
        let mut entry = serde_json::json!({
            "schema": 1, "app": app.update_app, "version": v, "date": "2026-11-13", "notes": format!("Neu in {v}."),
            "packages": {(p.platform().key()): {"url": url, "sha256": sha, "size": v.len(), "kind": "nsis-silent"}}
        });
        if let Some(m) = min_launcher {
            entry["minLauncherVersion"] = serde_json::json!(m);
        }
        let mut routes = self.server.routes.lock().unwrap();
        routes.insert(
            p.latest_asset_url("entry.json"),
            serde_json::to_vec(&entry).unwrap(),
        );
        routes.insert(
            p.latest_asset_url("provenance.sigstore.json"),
            b"{}".to_vec(),
        );
        routes.insert(url, v.as_bytes().to_vec());
    }

    pub(crate) fn lock_dir(&self) -> PathBuf {
        daedyx_update::lock::shared_lock_dir(self.local_dir.path())
    }

    pub(crate) fn launcher(&self) -> (Launcher, TestInstaller) {
        let (l, i, _) = self.launcher_with_system();
        (l, i)
    }

    pub(crate) fn launcher_with_system(&self) -> (Launcher, TestInstaller, FakeSystem) {
        let installer = TestInstaller::default();
        let system = FakeSystem::default();
        let trusted = Arc::clone(&self.trusted);
        let remote = Remote::new(
            Box::new(self.server.clone()),
            Box::new(move |_| Ok(Box::new(Attrappe(Arc::clone(&trusted))) as Box<dyn Provenance>)),
        );
        let launcher = Launcher::new(
            remote,
            self.data_dir.path().to_path_buf(),
            self.lock_dir(),
            "0.2.0".into(),
            Box::new(installer.clone()),
            Box::new(system.clone()),
        );
        (launcher, installer, system)
    }
}
