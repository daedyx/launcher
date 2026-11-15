//! Ende-zu-Ende: echte Kommandozeile, echter Download aus einem Katalogordner, echter
//! Prozessstart eines (Test-)Installers. Läuft nur unter Windows und mit `--features test-helpers`.
#![cfg(all(windows, feature = "test-helpers"))]

use daedyx_update::verify::sha256_file;
use std::process::Command;

struct Env {
    catalog: tempfile::TempDir,
    data: tempfile::TempDir,
    keys: tempfile::TempDir,
    keypair: minisign::KeyPair,
}

impl Env {
    fn new() -> Env {
        let keypair = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let env = Env {
            catalog: tempfile::tempdir().unwrap(),
            data: tempfile::tempdir().unwrap(),
            keys: tempfile::tempdir().unwrap(),
            keypair,
        };
        std::fs::write(
            env.keys.path().join("pub.key"),
            env.keypair.pk.to_box().unwrap().to_string(),
        )
        .unwrap();
        env
    }

    /// Veröffentlicht Versionen: Jedes Paket ist der Test-Installer mit angehängter Versionsnummer.
    fn publish(&self, versions: &[&str], wrong_sha: Option<&str>) {
        std::fs::create_dir_all(self.catalog.path().join("packages")).unwrap();
        let mut releases = Vec::new();
        for v in versions {
            let rel = format!("packages/desk-{v}.exe");
            let path = self.catalog.path().join(&rel);
            let mut bytes = std::fs::read(env!("CARGO_BIN_EXE_fake-installer")).unwrap();
            bytes.extend(format!("\nDAEDYX-VERSION:{v}").bytes());
            std::fs::write(&path, &bytes).unwrap();
            let sha = if wrong_sha == Some(*v) {
                "00".repeat(32)
            } else {
                sha256_file(&path).unwrap()
            };
            releases.push(format!(
                r#"{{"version":"{v}","date":"2026-11-13","packages":{{"windows-x64":{{"url":"{rel}","sha256":"{sha}","size":{},"kind":"nsis-silent"}}}}}}"#,
                bytes.len()
            ));
        }
        let json = format!(
            r#"{{"schema":1,"minLauncherVersion":"0.1.0","apps":[{{"id":"desk","name":"Daedyx Desk","releases":[{}]}}]}}"#,
            releases.join(",")
        );
        std::fs::write(self.catalog.path().join("catalog.json"), &json).unwrap();
        let sig = minisign::sign(
            Some(&self.keypair.pk),
            &self.keypair.sk,
            std::io::Cursor::new(json.as_bytes()),
            Some("trusted"),
            Some("untrusted"),
        )
        .unwrap();
        std::fs::write(
            self.catalog.path().join("catalog.json.minisig"),
            sig.to_string(),
        )
        .unwrap();
    }

    fn run(&self, args: &[&str]) -> (bool, serde_json::Value) {
        let out = Command::new(env!("CARGO_BIN_EXE_daedyx"))
            .arg("--catalog")
            .arg(self.catalog.path())
            .arg("--pubkey")
            .arg(self.keys.path().join("pub.key"))
            .arg("--data-dir")
            .arg(self.data.path())
            .arg("--json")
            .args(args)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        let value = serde_json::from_str(&text).unwrap_or_else(|e| {
            panic!(
                "keine JSON-Ausgabe: {e}\nstdout: {text}\nstderr: {}",
                String::from_utf8_lossy(&out.stderr)
            )
        });
        (out.status.success(), value)
    }
}

#[test]
fn installieren_aktualisieren_entfernen() {
    let env = Env::new();
    env.publish(&["1.0.0"], None);

    let (ok, v) = env.run(&["list"]);
    assert!(ok, "{v}");
    assert_eq!(v[0]["id"], "desk");
    assert!(v[0]["installed"].is_null());

    let (ok, v) = env.run(&["install", "desk"]);
    assert!(ok, "{v}");
    assert_eq!(v[0]["result"], "installed");
    assert_eq!(v[0]["version"], "1.0.0");
    assert!(env.data.path().join("apps/desk/app.json").exists());
    assert!(env.data.path().join("apps/desk/uninstall.exe").exists());

    let (_, v) = env.run(&["list"]);
    assert_eq!(v[0]["installed"], "1.0.0");
    assert_eq!(v[0]["update_available"], false);

    env.publish(&["1.0.0", "1.1.0"], None);
    let (_, v) = env.run(&["list"]);
    assert_eq!(v[0]["update_available"], true);

    let (ok, v) = env.run(&["update"]);
    assert!(ok, "{v}");
    assert_eq!(v[0]["result"], "updated");
    assert_eq!(v[0]["from"], "1.0.0");
    assert_eq!(v[0]["to"], "1.1.0");

    let (ok, v) = env.run(&["remove", "desk"]);
    assert!(ok, "{v}");
    assert!(!env.data.path().join("apps/desk").exists());
}

#[test]
fn veraenderter_katalog_wird_abgelehnt() {
    let env = Env::new();
    env.publish(&["1.0.0"], None);
    let path = env.catalog.path().join("catalog.json");
    let mut json = std::fs::read_to_string(&path).unwrap();
    json.push(' ');
    std::fs::write(&path, json).unwrap();

    let (ok, v) = env.run(&["install", "desk"]);
    assert!(!ok);
    assert!(v["error"].as_str().unwrap().contains("Signatur"), "{v}");
    assert!(!env.data.path().join("apps/desk").exists());
}

#[test]
fn falsche_pruefsumme_wird_abgelehnt_und_nichts_installiert() {
    let env = Env::new();
    env.publish(&["1.0.0"], Some("1.0.0"));

    let (ok, v) = env.run(&["install", "desk"]);
    assert!(!ok);
    assert!(v["error"].as_str().unwrap().contains("Prüfsumme"), "{v}");
    assert!(!env.data.path().join("apps/desk").exists());
}

#[test]
fn update_einer_nicht_installierten_app_wird_abgelehnt() {
    let env = Env::new();
    env.publish(&["1.0.0"], None);
    let (ok, v) = env.run(&["update", "desk"]);
    assert!(!ok);
    assert!(
        v["error"].as_str().unwrap().contains("nicht installiert"),
        "{v}"
    );
}

#[test]
fn launch_startet_die_installierte_app() {
    let env = Env::new();
    env.publish(&["1.0.0"], None);
    let (ok, _) = env.run(&["install", "desk"]);
    assert!(ok);
    let (ok, v) = env.run(&["launch", "desk"]);
    assert!(ok, "{v}");
    assert_eq!(v["result"], "launched");
    let (ok, v) = env.run(&["launch", "gibtsnicht"]);
    assert!(!ok);
    assert!(v["error"].is_string());
}

#[test]
fn einstellung_wird_geprueft_gespeichert_und_ersetzt_die_startparameter() {
    let env = Env::new();
    env.publish(&["1.0.0"], None);
    let run_plain = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_daedyx"))
            .arg("--data-dir")
            .arg(env.data.path())
            .arg("--json")
            .args(args)
            .output()
            .unwrap();
        (
            out.status.success(),
            serde_json::from_slice::<serde_json::Value>(&out.stdout).unwrap_or_default(),
        )
    };
    // Ohne Einstellung und ohne Parameter: verständlicher Fehler
    let (ok, v) = run_plain(&["list"]);
    assert!(!ok);
    assert!(v["error"].as_str().unwrap().contains("config set"), "{v}");
    // Ungültiger Schlüssel wird nicht gespeichert
    let junk = env.keys.path().join("junk.pub");
    std::fs::write(&junk, "nichts").unwrap();
    let (ok, _) = run_plain(&[
        "config",
        "set",
        "--catalog",
        env.catalog.path().to_str().unwrap(),
        "--pubkey",
        junk.to_str().unwrap(),
    ]);
    assert!(!ok);
    assert!(!env.data.path().join("config.json").exists());
    // Gültig: gespeichert, danach geht `list` ohne Parameter
    let (ok, _) = run_plain(&[
        "config",
        "set",
        "--catalog",
        env.catalog.path().to_str().unwrap(),
        "--pubkey",
        env.keys.path().join("pub.key").to_str().unwrap(),
    ]);
    assert!(ok);
    let (ok, v) = run_plain(&["config", "show"]);
    assert!(ok);
    assert_eq!(v["catalog"], env.catalog.path().to_str().unwrap());
    let (ok, v) = run_plain(&["list"]);
    assert!(ok, "{v}");
    assert_eq!(v[0]["id"], "desk");
    assert_eq!(v[0]["running"], false);
}
