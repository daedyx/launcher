//! Erzeugt die Symboldatei des Launchers aus der Zeichnung der Marke:
//! `cargo run -p launcher-ui --bin make-icon -- installer/daedyx.ico`
fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("Aufruf: make-icon <Ausgabedatei.ico>");
        std::process::exit(2);
    };
    let bytes = launcher_ui::brand::ico_bytes(&[16, 24, 32, 48, 64, 128, 256]);
    std::fs::write(&path, bytes).expect("Symboldatei schreiben");
    println!("Geschrieben: {path}");
}
