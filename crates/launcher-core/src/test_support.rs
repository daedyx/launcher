//! Gemeinsame Test-Hilfen: Test-Katalog, Test-Installer.
use crate::error::CoreError;
use crate::installer::Installer;
use crate::service::Launcher;
use crate::source::Source;
use crate::system::{SystemInfo, UninstallEntry};
use daedyx_update::catalog::PackageKind;
use daedyx_update::verify::sha256_file;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub(crate) const PLATFORM: &str = "windows-x64";

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

pub(crate) struct Env {
    pub(crate) catalog_dir: tempfile::TempDir,
    pub(crate) data_dir: tempfile::TempDir,
    pub(crate) pubkey: String,
    pub(crate) keypair: minisign::KeyPair,
}

impl Env {
    pub(crate) fn new() -> Env {
        let keypair = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        Env {
            catalog_dir: tempfile::tempdir().unwrap(),
            data_dir: tempfile::tempdir().unwrap(),
            pubkey: keypair.pk.to_box().unwrap().to_string(),
            keypair,
        }
    }

    pub(crate) fn publish(&self, versions: &[&str]) {
        self.publish_with(versions, None);
    }

    /// Veröffentlicht die Versionen und signiert den Katalog. `wrong_sha`: für diese Version
    /// wird absichtlich eine falsche Prüfsumme eingetragen (der Katalog bleibt gültig signiert).
    pub(crate) fn publish_with(&self, versions: &[&str], wrong_sha: Option<&str>) {
        self.publish_apps(versions, &[], wrong_sha);
    }

    /// Veröffentlicht Versionen von `desk` und (falls angegeben) der App `launcher` und signiert den
    /// Katalog. `wrong_sha`: für diese Version von `desk` wird eine falsche Prüfsumme eingetragen.
    pub(crate) fn publish_apps(&self, desk: &[&str], launcher: &[&str], wrong_sha: Option<&str>) {
        std::fs::create_dir_all(self.catalog_dir.path().join("packages")).unwrap();
        let mut apps = Vec::new();
        for (id, name, versions, key) in [
            (
                "desk",
                "Daedyx Desk",
                desk,
                r#""registryKey":"Daedyx Desk","#,
            ),
            ("launcher", "Daedyx Launcher", launcher, ""),
        ] {
            if versions.is_empty() {
                continue;
            }
            let mut releases = Vec::new();
            for v in versions {
                let rel = format!("packages/{id}-{v}.exe");
                let path = self.catalog_dir.path().join(&rel);
                std::fs::write(&path, v.as_bytes()).unwrap();
                let sha = if id == "desk" && wrong_sha == Some(*v) {
                    "00".repeat(32)
                } else {
                    sha256_file(&path).unwrap()
                };
                releases.push(format!(
                    r#"{{"version":"{v}","date":"2026-11-13","packages":{{"{PLATFORM}":{{"url":"{rel}","sha256":"{sha}","size":{},"kind":"nsis-silent"}}}}}}"#,
                    v.len()
                ));
            }
            apps.push(format!(
                r#"{{"id":"{id}","name":"{name}",{key}"releases":[{}]}}"#,
                releases.join(",")
            ));
        }
        let json = format!(
            r#"{{"schema":1,"minLauncherVersion":"0.1.0","apps":[{}]}}"#,
            apps.join(",")
        );
        std::fs::write(self.catalog_dir.path().join("catalog.json"), &json).unwrap();
        let sig = minisign::sign(
            Some(&self.keypair.pk),
            &self.keypair.sk,
            std::io::Cursor::new(json.as_bytes()),
            Some("trusted"),
            Some("untrusted"),
        )
        .unwrap();
        std::fs::write(
            self.catalog_dir.path().join("catalog.json.minisig"),
            sig.to_string(),
        )
        .unwrap();
    }

    pub(crate) fn launcher(&self) -> (Launcher, TestInstaller) {
        let (l, i, _) = self.launcher_with_system();
        (l, i)
    }

    pub(crate) fn launcher_with_system(&self) -> (Launcher, TestInstaller, FakeSystem) {
        let installer = TestInstaller::default();
        let system = FakeSystem::default();
        let launcher = Launcher::new(
            Source::Dir(self.catalog_dir.path().to_path_buf()),
            PLATFORM.into(),
            self.data_dir.path().to_path_buf(),
            "0.1.0".into(),
            Box::new(installer.clone()),
            Box::new(system.clone()),
        );
        (launcher, installer, system)
    }
}
