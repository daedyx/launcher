//! Demo-Katalog für die Sichtprüfung: erfundene Apps, Testschlüssel, absichtlich langsamer Server.
//! Nur für Entwicklung; nichts davon gehört in die Veröffentlichung.
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
    reason = "SICHERHEIT-AUSNAHMEN A-007"
)]
use clap::{Parser, Subcommand};
use daedyx_update::verify::sha256_file;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::Duration;

const APPS: [(&str, &str, &str); 4] = [
    (
        "notizbuch",
        "Notizbuch",
        "Kurze Notizen sammeln und schnell wiederfinden.",
    ),
    (
        "zeichenbrett",
        "Zeichenbrett",
        "Ein großes Brett für Skizzen und Ideen.",
    ),
    (
        "stundenplan",
        "Stundenplan",
        "Der Wochenplan mit Erinnerungen, ganz ohne Konto.",
    ),
    (
        "kassenbuch",
        "Kassenbuch",
        "Einnahmen und Ausgaben für den kleinen Verein.",
    ),
];
const PADDING_BYTES: usize = 6 * 1024 * 1024;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Katalog mit Paketen und Testschlüssel erzeugen (Stufe 2 fügt Updates hinzu)
    Generate {
        dir: PathBuf,
        #[arg(long, default_value_t = 1)]
        stage: u8,
    },
    /// Ordner auf 127.0.0.1 absichtlich langsam ausliefern
    Serve {
        dir: PathBuf,
        #[arg(long, default_value_t = 8765)]
        port: u16,
        #[arg(long, default_value_t = 640)]
        kbps: u64,
    },
}

fn keys(dir: &Path) -> (minisign::PublicKey, minisign::SecretKey) {
    let secret_path = dir.join("test-secret.key");
    if let Ok(text) = std::fs::read_to_string(&secret_path) {
        let pk_text = std::fs::read_to_string(dir.join("pub.key")).expect("pub.key fehlt");
        let sk = minisign::SecretKeyBox::from_string(&text)
            .unwrap()
            .into_unencrypted_secret_key()
            .unwrap();
        let pk = minisign::PublicKeyBox::from_string(&pk_text)
            .unwrap()
            .into_public_key()
            .unwrap();
        return (pk, sk);
    }
    let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
    std::fs::write(dir.join("pub.key"), kp.pk.to_box().unwrap().to_string()).unwrap();
    std::fs::write(&secret_path, kp.sk.to_box(None).unwrap().to_string()).unwrap();
    (kp.pk, kp.sk)
}

fn generate(dir: &Path, stage: u8) {
    std::fs::create_dir_all(dir.join("packages")).unwrap();
    let fake = std::env::current_exe()
        .unwrap()
        .with_file_name("fake-installer.exe");
    let base =
        std::fs::read(&fake).expect("fake-installer.exe fehlt: mit --features test-helpers bauen");
    let mut apps = Vec::new();
    for (id, name, description) in APPS {
        let versions: &[&str] = if stage >= 2 && (id == "notizbuch" || id == "stundenplan") {
            &["1.0.0", "1.1.0"]
        } else {
            &["1.0.0"]
        };
        let mut releases = Vec::new();
        for v in versions {
            let rel = format!("packages/{id}-{v}.exe");
            let mut bytes = base.clone();
            bytes.extend(std::iter::repeat_n(0u8, PADDING_BYTES));
            bytes.extend(format!("\nDAEDYX-VERSION:{v}").bytes());
            let path = dir.join(&rel);
            std::fs::write(&path, &bytes).unwrap();
            releases.push(format!(
                r#"{{"version":"{v}","date":"2026-11-13","packages":{{"windows-x64":{{"url":"{rel}","sha256":"{}","size":{},"kind":"nsis-silent"}}}}}}"#,
                sha256_file(&path).unwrap(),
                bytes.len()
            ));
        }
        apps.push(format!(
            r#"{{"id":"{id}","name":"{name}","description":"{description}","license":"MIT","author":"Beispiel-Studio","releases":[{}]}}"#,
            releases.join(",")
        ));
    }
    let json = format!(
        r#"{{"schema":1,"minLauncherVersion":"0.1.0","apps":[{}]}}"#,
        apps.join(",")
    );
    std::fs::write(dir.join("catalog.json"), &json).unwrap();
    let (pk, sk) = keys(dir);
    let sig = minisign::sign(
        Some(&pk),
        &sk,
        std::io::Cursor::new(json.as_bytes()),
        Some("trusted"),
        Some("untrusted"),
    )
    .unwrap();
    std::fs::write(dir.join("catalog.json.minisig"), sig.to_string()).unwrap();
    println!("Demo-Katalog (Stufe {stage}) liegt in {}", dir.display());
    println!("Schlüssel: {}", dir.join("pub.key").display());
}

/// `GET /pfad HTTP/1.1` ergibt den relativen Pfad; alles, was den Ordner verlassen könnte, wird abgelehnt.
fn request_path(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let rel = parts.next()?.trim_start_matches('/').to_string();
    let bad = rel.is_empty()
        || rel.contains(':')
        || rel.contains('\\')
        || rel.split('/').any(|s| s == "..");
    (!bad).then_some(rel)
}

fn handle(mut stream: std::net::TcpStream, dir: PathBuf, chunk_pause: Duration) {
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf).unwrap_or(0);
    let head = String::from_utf8_lossy(&buf[..n]).to_string();
    let file = request_path(head.lines().next().unwrap_or(""))
        .and_then(|rel| std::fs::read(dir.join(rel)).ok());
    let Some(bytes) = file else {
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        return;
    };
    let header = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    );
    if stream.write_all(header.as_bytes()).is_err() {
        return;
    }
    for chunk in bytes.chunks(64 * 1024) {
        if stream.write_all(chunk).is_err() {
            return;
        }
        std::thread::sleep(chunk_pause);
    }
}

fn serve(dir: PathBuf, port: u16, kbps: u64) {
    let listener = TcpListener::bind(("127.0.0.1", port)).expect("Port belegt");
    let pause = Duration::from_millis(64 * 1000 / kbps.max(1));
    println!("Server: http://127.0.0.1:{port} ({kbps} KB/s). Beenden mit Strg+C.");
    for stream in listener.incoming().flatten() {
        let dir = dir.clone();
        std::thread::spawn(move || handle(stream, dir, pause));
    }
}

fn main() {
    match Cli::parse().command {
        Command::Generate { dir, stage } => generate(&dir, stage),
        Command::Serve { dir, port, kbps } => serve(dir, port, kbps),
    }
}

#[cfg(test)]
mod tests {
    use super::request_path;

    #[test]
    fn pfad_bleibt_im_ordner() {
        assert_eq!(
            request_path("GET /catalog.json HTTP/1.1"),
            Some("catalog.json".to_string())
        );
        assert_eq!(
            request_path("GET /packages/a-1.0.0.exe HTTP/1.1"),
            Some("packages/a-1.0.0.exe".to_string())
        );
        assert_eq!(request_path("GET /../geheim HTTP/1.1"), None);
        assert_eq!(request_path("GET /a/../../b HTTP/1.1"), None);
        assert_eq!(request_path("GET /c:/x HTTP/1.1"), None);
        assert_eq!(request_path("POST /x HTTP/1.1"), None);
    }
}
