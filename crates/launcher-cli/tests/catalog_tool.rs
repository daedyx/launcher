//! Ablauf mit dem Katalog-Werkzeug: Schlüssel erzeugen, Release einfügen, signieren, prüfen,
//! und der Launcher liest den Katalog. Mit einem Testschlüssel ohne Passwort.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::arithmetic_side_effects,
    clippy::exit,
    clippy::disallowed_methods,
    reason = "SICHERHEIT-AUSNAHMEN A-009"
)]
use std::path::Path;
use std::process::Command;

fn tool(args: &[&str]) -> (bool, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_daedyx-catalog"))
        .args(args)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn entry(dir: &Path, version: &str) -> String {
    let path = dir.join(format!("entry-{version}.json"));
    std::fs::write(
        &path,
        format!(
            r#"{{"version":"{version}","date":"2026-11-14","notes":"neu","packages":{{"windows-x64":{{"url":"https://example.org/desk-{version}.exe","sha256":"{}","size":123,"kind":"nsis-silent"}}}}}}"#,
            "ab".repeat(32)
        ),
    )
    .unwrap();
    path.display().to_string()
}

#[test]
fn ablauf_schluessel_release_signieren_pruefen_und_launcher_liest_ihn() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let keys = d.join("keys");
    let (ok, _, err) = tool(&["keygen", "--out", keys.to_str().unwrap(), "--no-password"]);
    assert!(ok, "{err}");
    assert!(keys.join("catalog.key").exists() && keys.join("catalog.pub").exists());
    // Ein vorhandener Schlüssel wird nie überschrieben
    let (ok, _, err) = tool(&["keygen", "--out", keys.to_str().unwrap(), "--no-password"]);
    assert!(!ok && err.contains("nichts überschrieben"), "{err}");

    let catalog = d.join("catalog").join("catalog.json");
    std::fs::create_dir_all(catalog.parent().unwrap()).unwrap();
    let c = catalog.to_str().unwrap();
    let (ok, out, err) = tool(&[
        "add-release",
        "--catalog",
        c,
        "--app",
        "desk",
        "--entry",
        &entry(d, "0.1.0"),
        "--name",
        "Daedyx Desk",
        "--description",
        "Mail, Kalender, Aufgaben",
        "--registry-key",
        "Daedyx Desk",
    ]);
    assert!(ok, "{err}");
    assert!(out.contains("daedyx-catalog sign"), "{out}");

    let key = keys.join("catalog.key");
    let public = keys.join("catalog.pub");
    let (ok, out, err) = tool(&["sign", "--catalog", c, "--key", key.to_str().unwrap()]);
    assert!(ok, "{err}");
    assert!(out.contains("Gegenprobe bestanden"), "{out}");
    let (ok, out, _) = tool(&[
        "verify",
        "--catalog",
        c,
        "--public",
        public.to_str().unwrap(),
    ]);
    assert!(ok && out.contains("1 App(s), 1 Release(s)"), "{out}");

    // Zweite Version: doppelte wird abgelehnt, die alte Signatur ist danach weg
    let (ok, _, err) = tool(&[
        "add-release",
        "--catalog",
        c,
        "--app",
        "desk",
        "--entry",
        &entry(d, "0.1.0"),
    ]);
    assert!(!ok && err.contains("gibt es schon"), "{err}");
    let (ok, _, err) = tool(&[
        "add-release",
        "--catalog",
        c,
        "--app",
        "desk",
        "--entry",
        &entry(d, "0.2.0"),
    ]);
    assert!(ok, "{err}");
    assert!(!d.join("catalog").join("catalog.json.minisig").exists());
    // Ohne neue Signatur ist der Katalog nicht prüfbar
    let (ok, _, _) = tool(&[
        "verify",
        "--catalog",
        c,
        "--public",
        public.to_str().unwrap(),
    ]);
    assert!(!ok);
    let (ok, _, err) = tool(&["sign", "--catalog", c, "--key", key.to_str().unwrap()]);
    assert!(ok, "{err}");

    // Veränderung nach dem Signieren wird erkannt
    let text = std::fs::read_to_string(&catalog).unwrap();
    std::fs::write(&catalog, text.replace("Mail, Kalender", "Mail, Spam")).unwrap();
    let (ok, _, _) = tool(&[
        "verify",
        "--catalog",
        c,
        "--public",
        public.to_str().unwrap(),
    ]);
    assert!(!ok);
    std::fs::write(&catalog, text).unwrap();

    // Der Launcher liest den Katalog mit dem öffentlichen Schlüssel
    let data = d.join("data");
    let out = Command::new(env!("CARGO_BIN_EXE_daedyx"))
        .args(["--catalog", d.join("catalog").to_str().unwrap()])
        .args(["--pubkey", public.to_str().unwrap()])
        .args(["--data-dir", data.to_str().unwrap(), "--json", "list"])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v[0]["id"], "desk", "{v}");
    assert_eq!(v[0]["latest"], "0.2.0");
}

#[test]
fn package_url_ersetzt_die_adresse_fuer_einen_lokalen_katalogordner() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let catalog = d.join("catalog.json");
    let (ok, _, err) = tool(&[
        "add-release",
        "--catalog",
        catalog.to_str().unwrap(),
        "--app",
        "desk",
        "--entry",
        &entry(d, "0.3.0"),
        "--name",
        "Daedyx Desk",
        "--package-url",
        "packages/Daedyx.Desk_0.3.0_x64-setup.exe",
    ]);
    assert!(ok, "{err}");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&catalog).unwrap()).unwrap();
    let pkg = &json["apps"][0]["releases"][0]["packages"]["windows-x64"];
    assert_eq!(pkg["url"], "packages/Daedyx.Desk_0.3.0_x64-setup.exe");
    // Prüfsumme und Größe aus dem Eintrag bleiben unverändert
    assert_eq!(pkg["size"], 123);
    assert_eq!(pkg["sha256"], "ab".repeat(32));
}

/// Wie die GitHub-Pipeline: geheimer Schlüssel und Passwort kommen aus Umgebungsvariablen.
#[test]
fn signieren_ohne_rueckfrage_mit_schluessel_und_passwort_aus_der_umgebung() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    // Verschlüsseltes Testschlüsselpaar (nur für diesen Test erzeugt)
    let pair = minisign::KeyPair::generate_encrypted_keypair(Some("pw-test".into())).unwrap();
    let secret = pair.sk.to_box(None).unwrap().to_string();
    let public = pair.pk.to_box().unwrap().to_string();
    let public_file = d.join("catalog.pub");
    std::fs::write(&public_file, &public).unwrap();

    let catalog = d.join("catalog.json");
    let c = catalog.to_str().unwrap();
    let (ok, _, err) = tool(&[
        "add-release",
        "--catalog",
        c,
        "--app",
        "desk",
        "--entry",
        &entry(d, "0.1.0"),
        "--name",
        "Daedyx Desk",
    ]);
    assert!(ok, "{err}");

    let run = |password: &str| {
        Command::new(env!("CARGO_BIN_EXE_daedyx-catalog"))
            .args(["sign", "--catalog", c, "--key-env", "DAEDYX_CATALOG_KEY"])
            .args(["--public", public_file.to_str().unwrap()])
            .env("DAEDYX_CATALOG_KEY", &secret)
            .env("DAEDYX_KEY_PASSWORD", password)
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap()
    };
    // Falsches Passwort: Fehler, keine Signatur, nichts Geheimes in der Ausgabe
    let bad = run("falsch");
    assert!(!bad.status.success());
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&bad.stdout),
        String::from_utf8_lossy(&bad.stderr)
    );
    assert!(
        !text.contains("falsch") && !text.contains(&secret),
        "{text}"
    );
    assert!(!d.join("catalog.json.minisig").exists());
    // Richtiges Passwort: signiert, Gegenprobe bestanden, prüfbar
    let good = run("pw-test");
    assert!(
        good.status.success(),
        "{}",
        String::from_utf8_lossy(&good.stderr)
    );
    let (ok, out, _) = tool(&[
        "verify",
        "--catalog",
        c,
        "--public",
        public_file.to_str().unwrap(),
    ]);
    assert!(ok && out.contains("Signatur gültig"), "{out}");
}

#[test]
fn schluessel_datei_und_schluessel_aus_umgebung_zugleich_oder_keiner_wird_abgelehnt() {
    let (ok, _, _) = tool(&["sign", "--catalog", "x.json"]);
    assert!(!ok);
    let (ok, _, _) = tool(&[
        "sign",
        "--catalog",
        "x.json",
        "--key",
        "a",
        "--key-env",
        "B",
    ]);
    assert!(!ok);
}
