//! Erzeugt die Symboldatei des Launchers aus der Zeichnung der Marke:
//! `cargo run -p launcher-ui --bin make-icon -- installer/daedyx.ico`
use std::process::ExitCode;

#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "SICHERHEIT-AUSNAHMEN A-014"
)]
fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Aufruf: make-icon <Ausgabedatei.ico>");
        return ExitCode::from(2);
    };
    let bytes = launcher_ui::brand::ico_bytes(&[16, 24, 32, 48, 64, 128, 256]);
    if std::fs::write(&path, bytes).is_err() {
        eprintln!("Symboldatei konnte nicht geschrieben werden");
        return ExitCode::FAILURE;
    }
    println!("Geschrieben: {path}");
    ExitCode::SUCCESS
}
