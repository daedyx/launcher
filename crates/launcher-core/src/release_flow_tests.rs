//! Ein erfundenes Release, gebaut wie im Workflow (`scripts/ci/make-entry.ps1`, Kennung, Dateinamen und Subjekte aus
//! `release.yml`), besteht die Selbstprüfung `release_check::verify`, also Suche und Download des Kerns mit der festen
//! Quelle dieser App. Nur Windows: Das Skript läuft mit Windows PowerShell 5.1, wie im Workflow-Schritt.
use super::{repo_root, subjects, workflow, workflow_app, VERSION};
use crate::apps::{LAUNCHER, OWNER};
use crate::release_check;
use daedyx_update::attest::{AttestError, AttestKind, Provenance};
use daedyx_update::check::{BUNDLE_FILE, ENTRY_FILE};
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

const SETUP: &[u8] = b"erfundenes Launcher-Setup 9.9.9";
const BUNDLE: &[u8] = b"{\"erfunden\": true}";
const NOTES: &str = "Erfundene Neuerung mit \u{201e}Anf\u{fc}hrungszeichen\u{201c} und \"Zitat\".";

/// Nimmt nur die Subjekte an, die der Workflow attestiert, und nur im veröffentlichten Bündel.
struct Subjects(Vec<[u8; 32]>);

impl Provenance for Subjects {
    fn verify(&self, sha256: &[u8; 32], bundle: &[u8]) -> Result<(), AttestError> {
        let kind = if bundle != BUNDLE {
            AttestKind::Malformed
        } else if self.0.contains(sha256) {
            return Ok(());
        } else {
            AttestKind::NotInBundle
        };
        Err(AttestError {
            kind,
            detail: "Attrappe".into(),
        })
    }
}

fn sha(path: &Path) -> [u8; 32] {
    use sha2::Digest;
    sha2::Sha256::digest(std::fs::read(path).unwrap()).into()
}

/// Windows PowerShell 5.1 mit einem Skript und Argumenten; liefert den Fehlercode.
#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-031")]
pub(super) fn powershell(script: &Path, args: &[&str]) -> Option<i32> {
    let root = std::env::var_os("SystemRoot").unwrap();
    let ps = Path::new(&root).join(r"System32\WindowsPowerShell\v1.0\powershell.exe");
    let head = [
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ];
    Command::new(ps)
        .args(head)
        .arg(script)
        .args(args)
        .output()
        .unwrap()
        .status
        .code()
}

/// `entry.json` mit dem Skript des Workflows; liefert das Ende des Skripts.
pub(super) fn make_entry(dir: &Path, setup: &Path, app: &str, notes: &str) -> Option<i32> {
    let notes_file = dir.join("notiz.txt");
    std::fs::write(&notes_file, notes).unwrap();
    let s = |p: &Path| p.to_str().unwrap().to_owned();
    let repo = format!("{OWNER}/{}", LAUNCHER.repo);
    let (notes_file, setup, out) = (s(&notes_file), s(setup), s(&dir.join(ENTRY_FILE)));
    let args = [
        "-App",
        app,
        "-Repo",
        &repo,
        "-Version",
        VERSION,
        "-Date",
        "2026-12-24",
        "-NotesFile",
        &notes_file,
        "-Setup",
        &setup,
        "-Out",
        &out,
    ];
    powershell(&repo_root().join(r"scripts\ci\make-entry.ps1"), &args)
}

/// Ordner mit Setup, `entry.json` und Bündel, wie ihn der Workflow vor der Selbstprüfung hat.
fn release_dir(app: &str) -> TempDir {
    let setup_name = format!("daedyx-launcher-{VERSION}-setup.exe");
    assert_eq!(
        subjects(&workflow()),
        [ENTRY_FILE.to_owned(), setup_name.clone()]
    );
    let dir = TempDir::new().unwrap();
    let setup = dir.path().join(&setup_name);
    std::fs::write(&setup, SETUP).unwrap();
    assert_eq!(make_entry(dir.path(), &setup, app, NOTES), Some(0));
    std::fs::write(dir.path().join(BUNDLE_FILE), BUNDLE).unwrap();
    std::fs::remove_file(dir.path().join("notiz.txt")).unwrap();
    dir
}

#[test]
fn erfundenes_release_wie_der_workflow_es_baut_besteht_die_selbstpruefung() {
    let tmp = release_dir(&workflow_app(&workflow()));
    let dir = tmp.path();
    let setup = dir.join(format!("daedyx-launcher-{VERSION}-setup.exe"));
    let all = Subjects(vec![sha(&dir.join(ENTRY_FILE)), sha(&setup)]);
    let check = |dir: &Path, v: &str, p: &Subjects, w: &str| {
        release_check::verify(&LAUNCHER, dir, v, p, &dir.join(w))
    };
    let got = check(dir, VERSION, &all, "probe").unwrap();
    assert_eq!(std::fs::read(got).unwrap(), SETUP);
    // Gegenproben: ohne den Eintrag unter den Subjekten, mit falscher Version, mit falscher Kennung.
    let only_setup = Subjects(vec![sha(&setup)]);
    assert!(check(dir, VERSION, &only_setup, "p2").is_err());
    assert!(check(dir, "9.9.8", &all, "p3").is_err());
    let other = release_dir("daedyx-launcher-falsch");
    let all = Subjects(vec![sha(&other.path().join(ENTRY_FILE)), sha(&setup)]);
    assert!(check(other.path(), VERSION, &all, "p4").is_err());
}
