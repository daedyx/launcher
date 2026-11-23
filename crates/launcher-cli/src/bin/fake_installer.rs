//! Test-Installer: verhält sich wie ein NSIS-Setup (`/S /D=<Ordner>`) und legt `app.json`
//! sowie `uninstall.exe` an. Die Version steht am Dateiende hinter `\nDAEDYX-VERSION:`.
//! Heißt die Datei `uninstall.exe`, entfernt sie `app.json` (wie ein Deinstaller).
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
    reason = "SICHERHEIT-AUSNAHMEN A-006"
)]

use std::path::{Path, PathBuf};
use std::{env, fs, process::exit};

const MARKER: &[u8] = b"\nDAEDYX-VERSION:";

fn own_version(exe: &Path) -> String {
    let bytes = fs::read(exe).unwrap_or_default();
    match bytes.windows(MARKER.len()).rposition(|w| w == MARKER) {
        Some(pos) => String::from_utf8_lossy(&bytes[pos + MARKER.len()..])
            .trim()
            .to_string(),
        None => "0.0.0".to_string(),
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if !args.iter().any(|a| a == "/S") {
        eprintln!("fake-installer: nur /S wird unterstützt");
        exit(2);
    }
    let exe = env::current_exe().unwrap();
    let stem = exe.file_stem().unwrap().to_string_lossy().to_lowercase();

    if stem == "app" {
        exit(0); // Test-App: startet und beendet sich sofort
    }

    if stem == "uninstall" {
        let _ = fs::remove_file(exe.parent().unwrap().join("app.json"));
        exit(0);
    }

    let Some(dir) = args
        .iter()
        .find_map(|a| a.strip_prefix("/D="))
        .map(PathBuf::from)
    else {
        eprintln!("fake-installer: /D= fehlt");
        exit(3);
    };
    fs::create_dir_all(&dir).unwrap();
    let version = own_version(&exe);
    fs::write(
        dir.join("app.json"),
        format!(r#"{{"version":"{version}","exe":"app.exe"}}"#),
    )
    .unwrap();
    fs::copy(&exe, dir.join("app.exe")).unwrap();
    fs::copy(&exe, dir.join("uninstall.exe")).unwrap();
}
