//! Selbstprüfung im Release-Lauf (`release.yml`, Muster Tasks T7v M-T7v-2): prüft Setup, `entry.json` und Bündel in
//! `<Ordner>` mit dem echten GitHub-Prüfer des Launchers (eingebaute Vertrauenswurzel, Regeln des Kerns) und der festen
//! Quelle des Selbst-Updates (`apps::LAUNCHER`), bevor etwas veröffentlicht wird.
//! Aufruf: `release_selbstpruefung <Ordner> <Version> <Arbeitsordner>`. Endet mit Fehlercode, wenn der Launcher das
//! Release ablehnen würde.
use daedyx_update::attest::GithubAttestation;
use launcher_core::{apps, release_check};
use std::path::Path;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dir, version, work] = args.as_slice() else {
        return Err("Aufruf: <Ordner> <Version> <Arbeitsordner>".into());
    };
    let policy = apps::LAUNCHER
        .policy()
        .ok_or("keine Update-Regel für dieses System")?;
    let github = GithubAttestation::new(&policy).map_err(|e| format!("Prüfer: {e:?}"))?;
    release_check::verify(
        &apps::LAUNCHER,
        Path::new(dir),
        version,
        &github,
        Path::new(work),
    )
    .map(|_| ())
    .map_err(|r| format!("Der Launcher würde dieses Release ablehnen: {r:?}"))
}
