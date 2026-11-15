# L2 Oberfläche (egui) — Detailplan

> **Für ausführende Agenten:** Pflicht-Skill: `superpowers:subagent-driven-development` (empfohlen) oder `superpowers:executing-plans`. Schritte sind mit Checkboxen (`- [ ]`) markiert. Test zuerst, kleine Commits.

**Ziel:** Ein Fenster `daedyx-launcher`, das die Apps des signierten Katalogs als Karten zeigt und installieren, aktualisieren, entfernen und öffnen lässt, mit Fortschritt, verständlichen Fehlern, deutschen Texten aus einer Sprachdatei und Hell/Dunkel nach Windows.

**Architektur:** Der Kern `launcher-core` bekommt zwei Dinge, die der Oberfläche fehlen: „Öffnen“ (App starten) und einen Hintergrund-Arbeiter (`jobs.rs`), der Aufträge in einem eigenen Thread abarbeitet und Ereignisse (Katalog, Fortschritt, Ergebnis, Status) über einen Kanal meldet; ein Wecker-Rückruf (`request_repaint`) hält den Kern frei von egui. Das neue Crate `launcher-ui` trennt Zustand (`model.rs`, ohne egui, voll getestet), Darstellung (`card_view.rs`, mit `egui_kittest` getestet) und Fenster (`app.rs`).

**Tech Stack:** Rust 1.99, eframe/egui **0.37.0** (Renderer wgpu, AccessKit eingeschaltet), `egui_kittest 0.37.0` für Oberflächentests, bestehende Crates `daedyx-update` und `launcher-core`.

## Global Constraints
- Sicherheit zuerst: nichts wird installiert oder gestartet, bevor Signatur des Katalogs und SHA-256 samt Größe geprüft sind; die Oberfläche ruft nur `Launcher::install/remove/open` auf und umgeht nichts. `http://` nur für `localhost` und `127.0.0.1`.
- Keine Geheimnisse und keine echten Daten im Repository. Demo-Apps sind erfunden, Schlüssel sind Testschlüssel und liegen nur im Temp-Ordner.
- Texte für Nutzer der Oberfläche stehen in `crates/launcher-ui/lang/de.json`, keine festen Texte im UI-Code. Code-Bezeichner Englisch, Kommentare nur für das Warum.
- Dateien klein (Richtwert unter 200 Zeilen), Tests in `*_tests.rs` neben dem Code.
- Keine Lizenzdatei anlegen. Passwörter und Schlüssel tippt Björn selbst.
- Windows zuerst; macOS ist L5. Kein Code, der macOS-Verhalten festlegt, außer einem einfachen Platzhalter, der klar scheitert.
- Befehle (PowerShell): `$env:Path += ";$env:USERPROFILE\.cargo\bin"`, dann `cargo test --workspace --features launcher-cli/test-helpers`, `cargo clippy --workspace --all-targets --features launcher-cli/test-helpers`, `cargo fmt --all`.
- Commit-Nachrichten enden mit `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.

## Vorab-Prüfung der Bibliothek (Wegwerf-Projekt, erledigt am 2026-11-13)
Ein Prototyp (außerhalb des Repositorys) hat bestätigt:
- `eframe 0.37.0` baut unter Windows in rund 40 s (erster Lauf), Fenster öffnet sich, Titel und Mindestgröße über `egui::ViewportBuilder` (`with_inner_size`, `with_min_inner_size`, `with_title`) funktionieren.
- **API ist neu gegenüber älteren Anleitungen:** `App` verlangt `fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame)` (kein `update`); `egui::CentralPanel::default().show(ui, |ui| …)` nimmt ein `Ui`; der Eigentumsstil ist `Frame::group(ui.style())`, `ui.ctx().global_style().visuals.dark_mode`. `run_native(name, NativeOptions, Box::new(|cc| Ok(Box::new(app))))`.
- Hell/Dunkel: `ctx.set_theme(egui::ThemePreference::System)` (Standard); das Fenster folgte der Windows-Einstellung (hell, `dark_mode == false`).
- Schrift: eingebaute Schrift stellt Umlaute und ß korrekt dar; es ist keine eigene Schriftdatei nötig.
- Hintergrund-Thread: `cc.egui_ctx.clone()` plus `ctx.request_repaint()` aus dem Thread aktualisiert die Fortschrittsanzeige sofort.
- Barrierefreiheit: `accesskit` ist in den Standard-Features von eframe; `egui_kittest::Harness` findet Knöpfe über ihr Label (`get_by_label("Installieren").click()`), das nutzt dieselbe AccessKit-Schicht, die Windows-UI-Automation speist.
- `ViewportCommand::Screenshot` hat in 0.37 eine andere Form (Callback); wir fotografieren das Fenster stattdessen über PowerShell (`CopyFromScreen`) und prüfen Bedienung über UI-Automation.
- Beim Bauen weiter gegen den Compiler prüfen: `Window::show`, `Frame::inner_margin`, `ProgressBar`.

## Nicht in L2 (bewusst)
- **Symbole als Bild:** Der Katalog (Schema 1) hat kein Symbolfeld. L2 zeigt eine Buchstabenmarke in der Farbe der App-Kennung. Echte Symbole (signiert, mit Prüfsumme) gehören in eine spätere Schema-Erweiterung (L4).
- **Status „läuft“ und Laufprüfung vor einem Update** (Update-Regel 4 im Konzept): kommt mit der ersten echten App (L3), wo es zählt. Schlägt ein Setup wegen einer laufenden App fehl, erklärt der Fehlertext das.
- **Einstellungsdialog** (Katalogquelle, Schlüssel): L2 liest `--catalog`, `--pubkey`, `--data-dir` bzw. `DAEDYX_CATALOG`, `DAEDYX_PUBKEY`, `DAEDYX_DATA_DIR` wie die Kommandozeile; fehlen sie, erklärt das Fenster es. Eingebauter Schlüssel und Einstellungen: L4.
- Wiederaufnahme von Downloads, Abbrechen eines Downloads, mehrere Quellen, macOS, Fenstersymbol (Marken-Grafik).
- Die Texte der Kommandozeile (`launcher-cli`) und die deutschen Fehlertexte im Kern bleiben, wie sie sind; die Oberfläche übersetzt Fehler-Varianten in ihre Sprachdatei.

## Dateistruktur
**Kern (`crates/launcher-core/src`)**
- Ändern: `installer.rs` (Trait `Send + Sync`), `service.rs` (zwei Methoden `pub(crate)`), `error.rs` (neue Variante `NotLaunchable`), `lib.rs`, `service_tests.rs` (Hilfen ausgelagert)
- Neu: `test_support.rs` (gemeinsame Test-Hilfen `Env`, `TestInstaller`), `launch.rs` + `launch_tests.rs` (Öffnen), `jobs.rs` + `jobs_tests.rs` (Hintergrund-Arbeiter), `paths.rs` + `paths_tests.rs` (Datenordner)

**Kommandozeile (`crates/launcher-cli`)**
- Ändern: `src/main.rs` (Befehl `launch`, Datenordner aus dem Kern), `src/bin/fake_installer.rs` (schreibt `exe`, legt `app.exe` an), `tests/e2e.rs`, `Cargo.toml`
- Neu: `src/bin/demo_catalog.rs` (Demo-Katalog und langsamer Testserver, nur Feature `test-helpers`)

**Oberfläche (`crates/launcher-ui`, neu)**
- `Cargo.toml`, `lang/de.json`
- `src/lib.rs`, `src/main.rs`
- `src/i18n.rs` (+ `i18n_tests.rs`): Sprachdatei laden, `t`, `tf`
- `src/errors.rs` (+ `errors_tests.rs`): `CoreError` → deutscher Text
- `src/model.rs` (+ `model_tests.rs`): Karten, Fortschritt, Hinweise aus Ereignissen
- `src/card_view.rs` (+ `card_view_tests.rs`): eine Karte zeichnen, Aktion melden
- `src/app.rs`: Fenster, Kopfzeile, Raster, Entfernen-Rückfrage
- `src/theme.rs`: Thema und Schriftgrößen

**Sonstiges:** `Cargo.toml` (Arbeitsbereich), `scripts/window-shot.ps1`, `scripts/ui-invoke.ps1`, `docs/STAND.md`

---

### Task 1: Kern wird thread-tauglich, Test-Hilfen werden gemeinsam

**Files:**
- Modify: `crates/launcher-core/src/installer.rs:6`
- Modify: `crates/launcher-core/src/service_tests.rs` (Zeilen 1–114 auslagern)
- Create: `crates/launcher-core/src/test_support.rs`
- Modify: `crates/launcher-core/src/lib.rs`

**Interfaces:**
- Produces: `trait Installer: Send + Sync`; `crate::test_support::{PLATFORM, TestInstaller, Env}` (alle `pub(crate)`) mit `Env::new() -> Env`, `Env::publish(&self, &[&str])`, `Env::publish_with(&self, &[&str], Option<&str>)`, `Env::launcher(&self) -> (Launcher, TestInstaller)`, Felder `catalog_dir`, `data_dir`, `pubkey`, `keypair` (alle `pub(crate)`).

- [ ] **Step 1: Failing test schreiben** — in `service_tests.rs` am Ende anfügen:

```rust
#[test]
fn launcher_kann_zwischen_threads_wandern() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Launcher>();
}
```

- [ ] **Step 2: Test laufen lassen, muss scheitern**

Run: `cargo test -p launcher-core launcher_kann_zwischen_threads_wandern`
Expected: Übersetzungsfehler „`dyn Installer` cannot be sent/shared between threads safely“.

- [ ] **Step 3: Minimal umsetzen** — `installer.rs` Zeile 6:

```rust
pub trait Installer: Send + Sync {
```

- [ ] **Step 4: Test-Hilfen auslagern** — Neue Datei `crates/launcher-core/src/test_support.rs` mit dem Inhalt der Zeilen 1–114 von `service_tests.rs` (die `use`-Zeilen, `PLATFORM`, `TestInstaller`, `Env`, `impl Env`), mit diesen mechanischen Änderungen: `const PLATFORM` → `pub(crate) const PLATFORM`; `struct TestInstaller` → `pub(crate) struct TestInstaller` mit Feld `pub(crate) calls`; `struct Env` → `pub(crate) struct Env` mit allen Feldern `pub(crate)`; in `impl Env` jedes `fn` → `pub(crate) fn`. Unbenutzte `use`-Zeilen (`State`, `InstalledApp`, `Lock`, `Outcome`, `CoreError`, `UpdateError`) bleiben dort weg. Oben in der Datei `//! Gemeinsame Test-Hilfen: Test-Katalog, Test-Installer.`

  In `service_tests.rs` die Zeilen 1–114 durch diese Kopfzeilen ersetzen (der Rest der Datei bleibt):

```rust
use crate::error::CoreError;
use crate::lock::Lock;
use crate::service::{Launcher, Outcome};
use crate::state::{InstalledApp, State};
use crate::test_support::Env;
use daedyx_update::error::UpdateError;
```

  In `lib.rs` ergänzen:

```rust
#[cfg(test)]
mod test_support;
```

- [ ] **Step 5: Alle Kerntests laufen lassen**

Run: `cargo test -p launcher-core`
Expected: alle bisherigen 32 Tests plus der neue grün (33 passed).

- [ ] **Step 6: Commit**

```bash
git add crates/launcher-core
git commit -m "refactor: Kern thread-tauglich, Test-Hilfen gemeinsam nutzbar"
```

---

### Task 2: „Öffnen“ im Kern und in der Kommandozeile

**Files:**
- Create: `crates/launcher-core/src/launch.rs`, `crates/launcher-core/src/launch_tests.rs`
- Modify: `crates/launcher-core/src/error.rs`, `crates/launcher-core/src/service.rs` (`apps_dir`, `install_dir_for` → `pub(crate)`), `crates/launcher-core/src/lib.rs`
- Modify: `crates/launcher-cli/src/bin/fake_installer.rs`, `crates/launcher-cli/src/main.rs`, `crates/launcher-cli/tests/e2e.rs`

**Interfaces:**
- Produces: `launch::launch_target(&Path) -> Result<PathBuf, CoreError>`; `Launcher::open(&self, app_id: &str) -> Result<(), CoreError>`; `CoreError::NotLaunchable(String)`. Vertrag: `app.json` einer App enthält `"exe": "<relativer Pfad im Installationsordner>"`.

- [ ] **Step 1: Failing tests schreiben** — `launch_tests.rs`:

```rust
use crate::error::CoreError;
use crate::launch::launch_target;
use crate::test_support::Env;

fn app_dir_with(json: &str, file: Option<&str>) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("app.json"), json).unwrap();
    if let Some(f) = file {
        let p = dir.path().join(f);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, b"x").unwrap();
    }
    dir
}

#[test]
fn startprogramm_im_installationsordner_wird_gefunden() {
    let dir = app_dir_with(r#"{"version":"1.0.0","exe":"bin/app.exe"}"#, Some("bin/app.exe"));
    assert_eq!(launch_target(dir.path()).unwrap(), dir.path().join("bin/app.exe"));
}

#[test]
fn gefaehrliche_oder_fehlende_startprogramme_werden_abgelehnt() {
    for exe in ["../x.exe", "..\\x.exe", "C:\\Windows\\notepad.exe", "\\\\srv\\s\\x.exe", "/etc/x", "", "a:b.exe"] {
        let json = format!(r#"{{"version":"1.0.0","exe":{}}}"#, serde_json::json!(exe));
        let dir = app_dir_with(&json, None);
        assert!(
            matches!(launch_target(dir.path()), Err(CoreError::NotLaunchable(_))),
            "{exe:?} hätte abgelehnt werden müssen"
        );
    }
    let ohne_feld = app_dir_with(r#"{"version":"1.0.0"}"#, None);
    assert!(matches!(launch_target(ohne_feld.path()), Err(CoreError::NotLaunchable(_))));
    let datei_fehlt = app_dir_with(r#"{"version":"1.0.0","exe":"app.exe"}"#, None);
    assert!(matches!(launch_target(datei_fehlt.path()), Err(CoreError::NotLaunchable(_))));
}

#[test]
fn nicht_installierte_app_laesst_sich_nicht_oeffnen() {
    let env = Env::new();
    let (l, _) = env.launcher();
    assert!(matches!(l.open("desk"), Err(CoreError::NotInstalled(_))));
    assert!(matches!(l.open("../x"), Err(CoreError::Update(_))));
}

#[cfg(windows)]
#[test]
fn installierte_app_wird_gestartet() {
    let env = Env::new();
    let dir = env.data_dir.path().join("apps/desk");
    std::fs::create_dir_all(&dir).unwrap();
    let system = std::path::PathBuf::from(std::env::var("SystemRoot").unwrap()).join("System32\\whoami.exe");
    std::fs::copy(system, dir.join("app.exe")).unwrap();
    std::fs::write(dir.join("app.json"), r#"{"version":"1.0.0","exe":"app.exe"}"#).unwrap();
    let (l, _) = env.launcher();
    l.open("desk").unwrap();
}
```

- [ ] **Step 2: Test laufen lassen, muss scheitern**

Run: `cargo test -p launcher-core launch`
Expected: Übersetzungsfehler (Modul `launch` fehlt).

- [ ] **Step 3: Implementieren** — `error.rs` in `CoreError` ergänzen:

```rust
    #[error("Die App lässt sich nicht starten: {0}")]
    NotLaunchable(String),
```

  `service.rs`: `fn apps_dir` und `fn install_dir_for` je zu `pub(crate) fn` machen. `lib.rs`: `pub mod launch;` und `#[cfg(test)] mod launch_tests;`. Neue Datei `launch.rs`:

```rust
use crate::error::CoreError;
use crate::probe::read_installed_version;
use crate::service::Launcher;
use crate::state::State;
use daedyx_update::catalog::is_valid_app_id;
use daedyx_update::error::UpdateError;
use std::path::{Component, Path, PathBuf};

fn not_launchable(why: &str) -> CoreError {
    CoreError::NotLaunchable(why.to_string())
}

/// Das Startprogramm steht in `app.json` (`exe`) und muss ein relativer Pfad im Installationsordner sein.
pub fn launch_target(install_dir: &Path) -> Result<PathBuf, CoreError> {
    let bytes = std::fs::read(install_dir.join("app.json")).map_err(|_| not_launchable("app.json fehlt"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| not_launchable("app.json ist beschädigt"))?;
    let rel = value
        .get("exe")
        .and_then(|v| v.as_str())
        .ok_or_else(|| not_launchable("app.json nennt kein Startprogramm"))?;
    let path = Path::new(rel);
    let safe = !rel.is_empty()
        && !rel.contains(':')
        && path.components().all(|c| matches!(c, Component::Normal(_)));
    if !safe {
        return Err(not_launchable("Das Startprogramm liegt nicht im Installationsordner"));
    }
    let target = install_dir.join(path);
    if !target.is_file() {
        return Err(not_launchable("Das Startprogramm wurde nicht gefunden"));
    }
    Ok(target)
}

impl Launcher {
    /// Startet die installierte App und lässt sie unabhängig vom Launcher weiterlaufen.
    pub fn open(&self, app_id: &str) -> Result<(), CoreError> {
        if !is_valid_app_id(app_id) {
            return Err(UpdateError::UnknownApp(app_id.to_string()).into());
        }
        let state = State::load(&self.data_dir)?;
        let dir = self.install_dir_for(&state, app_id);
        if read_installed_version(&dir).is_none() {
            return Err(CoreError::NotInstalled(app_id.to_string()));
        }
        if !dir.starts_with(self.apps_dir()) {
            return Err(not_launchable("Der Ordner liegt außerhalb des Programmordners"));
        }
        spawn_detached(&launch_target(&dir)?, &dir)
    }
}

#[cfg(windows)]
fn spawn_detached(target: &Path, dir: &Path) -> Result<(), CoreError> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    std::process::Command::new(target)
        .current_dir(dir)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map(|_| ())
        .map_err(|e| CoreError::Io(e.to_string()))
}

#[cfg(not(windows))]
fn spawn_detached(_target: &Path, _dir: &Path) -> Result<(), CoreError> {
    Err(CoreError::PlatformUnsupported("Öffnen (kommt mit L5)".into()))
}
```

- [ ] **Step 3b: Test-Installer anpassen** — `fake_installer.rs`: in `main()` direkt nach `let stem = …` einfügen:

```rust
    if stem == "app" {
        exit(0); // Test-„App“: startet und beendet sich sofort
    }
```

  und das Schreiben der `app.json` ersetzen durch:

```rust
    fs::write(
        dir.join("app.json"),
        format!(r#"{{"version":"{version}","exe":"app.exe"}}"#),
    )
    .unwrap();
    fs::copy(&exe, dir.join("app.exe")).unwrap();
    fs::copy(&exe, dir.join("uninstall.exe")).unwrap();
```
  (die bisherige `fs::copy(… "uninstall.exe")`-Zeile entfällt, sie steckt nun im Block darüber).

- [ ] **Step 4: Kerntests laufen lassen**

Run: `cargo test -p launcher-core`
Expected: alle grün, inklusive der 4 neuen (`installierte_app_wird_gestartet` unter Windows).

- [ ] **Step 5: Befehl `launch` der Kommandozeile — erst Test** — in `e2e.rs` einen Test anfügen (Aufbau wie die vorhandenen Tests dort: `Env::new()`, `publish`, `run`):

```rust
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
```

  Run: `cargo test -p launcher-cli --features test-helpers launch` — Expected: FAIL (Befehl unbekannt).

- [ ] **Step 6: Umsetzen** — in `main.rs` im `enum Command` ergänzen `/// App starten\n    Launch { app: String },` und im `match` von `run`:

```rust
        Command::Launch { app } => {
            launcher.open(app)?;
            Ok(if cli.json {
                serde_json::json!({ "app": app, "result": "launched" }).to_string()
            } else {
                format!("{app} wird gestartet.")
            })
        }
```

- [ ] **Step 7: Alles laufen lassen und committen**

Run: `cargo test --workspace --features launcher-cli/test-helpers` — Expected: alle grün (die bisherigen 4 Ende-zu-Ende-Tests weiterhin).

```bash
git add -A
git commit -m "feat: App öffnen im Kern und als Befehl launch"
```

---

### Task 3: Hintergrund-Arbeiter im Kern

**Files:**
- Create: `crates/launcher-core/src/jobs.rs`, `crates/launcher-core/src/jobs_tests.rs`
- Modify: `crates/launcher-core/src/lib.rs`

**Interfaces:**
- Consumes: `Launcher::{load_catalog, status, install, remove, open}`, `test_support::Env`.
- Produces:
  - `enum Job { Refresh, Install(String), Remove(String), Open(String) }`
  - `enum Event { Catalog(Result<Catalog, CoreError>), Status(Vec<AppStatus>), Progress { app: String, done: u64, total: Option<u64> }, Installed { app: String, result: Result<Outcome, CoreError> }, Removed { app: String, result: Result<(), CoreError> }, Opened { app: String, result: Result<(), CoreError> } }` (`#[derive(Debug)]`)
  - `type Wake = Arc<dyn Fn() + Send + Sync>`
  - `Worker::start(launcher: Launcher, pubkey: String, wake: Wake) -> Worker`, `Worker::send(&self, Job)`, `Worker::try_recv(&self) -> Option<Event>`, `Worker::recv_timeout(&self, Duration) -> Option<Event>`
  - `fn percent(done: u64, total: Option<u64>) -> Option<u8>`
- Vertrag: Auf `Refresh` folgt `Catalog(Ok)` und `Status`, oder `Catalog(Err)`. Auf `Install`/`Remove` folgt `Progress*`, dann `Installed`/`Removed`, dann `Status`. Auf `Open` folgt `Opened`. Fortschritt wird nur bei geändertem Prozentwert (bzw. je 256 KiB ohne Gesamtgröße) gemeldet.

- [ ] **Step 1: Failing tests schreiben** — `jobs_tests.rs`:

```rust
use crate::error::CoreError;
use crate::jobs::{percent, Event, Job, Wake, Worker};
use crate::service::Outcome;
use crate::test_support::Env;
use daedyx_update::error::UpdateError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn start_with(env: &Env, pubkey: &str, wake: Wake) -> Worker {
    let (l, _) = env.launcher();
    Worker::start(l, pubkey.to_string(), wake)
}

fn start(env: &Env) -> Worker {
    start_with(env, &env.pubkey, Arc::new(|| {}))
}

fn next(w: &Worker) -> Event {
    w.recv_timeout(Duration::from_secs(5)).expect("Ereignis erwartet")
}

fn refreshed(env: &Env) -> Worker {
    let w = start(env);
    w.send(Job::Refresh);
    assert!(matches!(next(&w), Event::Catalog(Ok(_))));
    assert!(matches!(next(&w), Event::Status(_)));
    w
}

#[test]
fn prozent_wird_berechnet() {
    assert_eq!(percent(50, Some(200)), Some(25));
    assert_eq!(percent(300, Some(200)), Some(100));
    assert_eq!(percent(5, None), None);
    assert_eq!(percent(5, Some(0)), None);
}

#[test]
fn aktualisieren_meldet_katalog_und_status() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = start(&env);
    w.send(Job::Refresh);
    match next(&w) {
        Event::Catalog(Ok(c)) => assert_eq!(c.apps.len(), 1),
        other => panic!("{other:?}"),
    }
    match next(&w) {
        Event::Status(s) => assert_eq!(s[0].installed, None),
        other => panic!("{other:?}"),
    }
}

#[test]
fn installieren_meldet_fortschritt_ergebnis_und_status() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = refreshed(&env);
    w.send(Job::Install("desk".into()));
    let mut saw_full_progress = false;
    loop {
        match next(&w) {
            Event::Progress { done, total, .. } => saw_full_progress |= Some(done) == total,
            Event::Installed { app, result } => {
                assert_eq!(app, "desk");
                assert_eq!(result.unwrap(), Outcome::Installed { version: "1.0.0".into() });
                break;
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(saw_full_progress);
    match next(&w) {
        Event::Status(s) => assert_eq!(s[0].installed.as_deref(), Some("1.0.0")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn fremder_schluessel_meldet_fehler_und_installieren_scheitert_sauber() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let other = Env::new();
    let w = start_with(&env, &other.pubkey, Arc::new(|| {}));
    w.send(Job::Refresh);
    assert!(matches!(
        next(&w),
        Event::Catalog(Err(CoreError::Update(UpdateError::SignatureInvalid)))
    ));
    w.send(Job::Install("desk".into()));
    assert!(matches!(next(&w), Event::Catalog(Err(_)))); // erneuter Ladeversuch
    assert!(matches!(next(&w), Event::Installed { result: Err(_), .. }));
}

#[test]
fn entfernen_meldet_ergebnis_und_status() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = refreshed(&env);
    w.send(Job::Install("desk".into()));
    while !matches!(next(&w), Event::Status(_)) {}
    w.send(Job::Remove("desk".into()));
    assert!(matches!(next(&w), Event::Removed { result: Ok(()), .. }));
    match next(&w) {
        Event::Status(s) => assert_eq!(s[0].installed, None),
        other => panic!("{other:?}"),
    }
}

#[test]
fn oeffnen_einer_nicht_installierten_app_meldet_fehler() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let w = refreshed(&env);
    w.send(Job::Open("desk".into()));
    assert!(matches!(
        next(&w),
        Event::Opened { result: Err(CoreError::NotInstalled(_)), .. }
    ));
}

#[test]
fn jedes_ereignis_weckt_die_oberflaeche() {
    let env = Env::new();
    env.publish(&["1.0.0"]);
    let count = Arc::new(AtomicUsize::new(0));
    let c = count.clone();
    let w = start_with(&env, &env.pubkey, Arc::new(move || { c.fetch_add(1, Ordering::SeqCst); }));
    w.send(Job::Refresh);
    next(&w);
    next(&w);
    assert!(count.load(Ordering::SeqCst) >= 2);
}
```

- [ ] **Step 2: Test laufen lassen, muss scheitern**

Run: `cargo test -p launcher-core jobs` — Expected: Übersetzungsfehler (Modul `jobs` fehlt).

- [ ] **Step 3: Implementieren** — `lib.rs`: `pub mod jobs;` und `#[cfg(test)] mod jobs_tests;`. Neue Datei `jobs.rs`:

```rust
//! Hintergrund-Arbeiter: führt Aufträge in einem eigenen Thread aus und meldet Ereignisse.
//! Der Kern kennt keine Oberfläche; ein Wecker-Rückruf sagt ihr nur, dass es etwas zu holen gibt.
use crate::error::CoreError;
use crate::service::{AppStatus, Launcher, Outcome};
use daedyx_update::catalog::Catalog;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub enum Job {
    Refresh,
    Install(String),
    Remove(String),
    Open(String),
}

#[derive(Debug)]
pub enum Event {
    Catalog(Result<Catalog, CoreError>),
    Status(Vec<AppStatus>),
    Progress { app: String, done: u64, total: Option<u64> },
    Installed { app: String, result: Result<Outcome, CoreError> },
    Removed { app: String, result: Result<(), CoreError> },
    Opened { app: String, result: Result<(), CoreError> },
}

pub type Wake = Arc<dyn Fn() + Send + Sync>;

pub struct Worker {
    jobs: Sender<Job>,
    events: Receiver<Event>,
}

struct Emitter {
    tx: Sender<Event>,
    wake: Wake,
}

impl Emitter {
    fn emit(&self, event: Event) {
        let _ = self.tx.send(event);
        (self.wake)();
    }
}

impl Worker {
    pub fn start(launcher: Launcher, pubkey: String, wake: Wake) -> Worker {
        let (jobs_tx, jobs_rx) = channel();
        let (events_tx, events_rx) = channel();
        let out = Emitter { tx: events_tx, wake };
        std::thread::spawn(move || run(launcher, pubkey, jobs_rx, out));
        Worker { jobs: jobs_tx, events: events_rx }
    }

    pub fn send(&self, job: Job) {
        let _ = self.jobs.send(job);
    }

    pub fn try_recv(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }

    pub fn recv_timeout(&self, wait: Duration) -> Option<Event> {
        self.events.recv_timeout(wait).ok()
    }
}

pub fn percent(done: u64, total: Option<u64>) -> Option<u8> {
    total.filter(|t| *t > 0).map(|t| (done.min(t) * 100 / t) as u8)
}

fn refresh(launcher: &Launcher, pubkey: &str, catalog: &mut Option<Catalog>, out: &Emitter) {
    match launcher.load_catalog(pubkey) {
        Ok(c) => {
            *catalog = Some(c.clone());
            out.emit(Event::Catalog(Ok(c)));
            emit_status(launcher, catalog, out);
        }
        Err(e) => out.emit(Event::Catalog(Err(e))),
    }
}

fn emit_status(launcher: &Launcher, catalog: &Option<Catalog>, out: &Emitter) {
    if let Some(Ok(status)) = catalog.as_ref().map(|c| launcher.status(c)) {
        out.emit(Event::Status(status));
    }
}

fn install(launcher: &Launcher, catalog: &Catalog, app: &str, out: &Emitter) -> Result<Outcome, CoreError> {
    let mut last: Option<u64> = None;
    let mut progress = |done: u64, total: Option<u64>| {
        let key = match percent(done, total) {
            Some(p) => p as u64,
            None => done / (256 * 1024),
        };
        if last != Some(key) {
            last = Some(key);
            out.emit(Event::Progress { app: app.to_string(), done, total });
        }
    };
    launcher.install(catalog, app, &mut progress)
}

fn run(launcher: Launcher, pubkey: String, jobs: Receiver<Job>, out: Emitter) {
    let mut catalog: Option<Catalog> = None;
    while let Ok(job) = jobs.recv() {
        match job {
            Job::Refresh => refresh(&launcher, &pubkey, &mut catalog, &out),
            Job::Install(app) => {
                if catalog.is_none() {
                    refresh(&launcher, &pubkey, &mut catalog, &out);
                }
                let result = match &catalog {
                    Some(c) => install(&launcher, c, &app, &out),
                    None => Err(CoreError::Download("Der Katalog ist nicht geladen".into())),
                };
                out.emit(Event::Installed { app, result });
                emit_status(&launcher, &catalog, &out);
            }
            Job::Remove(app) => {
                let result = launcher.remove(&app);
                out.emit(Event::Removed { app, result });
                emit_status(&launcher, &catalog, &out);
            }
            Job::Open(app) => {
                let result = launcher.open(&app);
                out.emit(Event::Opened { app, result });
            }
        }
    }
}
```

- [ ] **Step 4: Tests laufen lassen**

Run: `cargo test -p launcher-core` — Expected: alles grün, 7 neue Tests in `jobs_tests`.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: Hintergrund-Arbeiter mit Fortschritt und Ereignissen im Kern"
```

---

### Task 4: Gemeinsamer Datenordner im Kern

**Files:**
- Create: `crates/launcher-core/src/paths.rs`, `crates/launcher-core/src/paths_tests.rs`
- Modify: `crates/launcher-core/Cargo.toml` (`dirs.workspace = true`), `crates/launcher-core/src/lib.rs`, `crates/launcher-cli/src/main.rs`

**Interfaces:** Produces `paths::default_data_dir() -> PathBuf` (`<LocalAppData>\Daedyx\Launcher`). Ersetzt `default_data_dir` in `launcher-cli/src/main.rs`.

- [ ] **Step 1: Failing test** — `paths_tests.rs`:

```rust
use crate::paths::default_data_dir;

#[test]
fn datenordner_endet_auf_daedyx_launcher() {
    let p = default_data_dir();
    assert!(p.ends_with("Daedyx/Launcher") || p.ends_with("Daedyx\\Launcher"), "{p:?}");
}
```

- [ ] **Step 2: Scheitern sehen:** `cargo test -p launcher-core paths` — Expected: Modul fehlt.

- [ ] **Step 3: Umsetzen** — `paths.rs`:

```rust
use std::path::PathBuf;

/// Gemeinsamer Datenordner von Kommandozeile und Oberfläche.
pub fn default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Daedyx")
        .join("Launcher")
}
```
  `lib.rs`: `pub mod paths;` und `#[cfg(test)] mod paths_tests;`. In `launcher-cli/src/main.rs` die lokale Funktion `default_data_dir` löschen, `use launcher_core::paths::default_data_dir;` ergänzen; `dirs` aus `launcher-cli/Cargo.toml` entfernen.

- [ ] **Step 4: Grün und Commit**

Run: `cargo test --workspace --features launcher-cli/test-helpers` — Expected: alles grün.

```bash
git add -A
git commit -m "refactor: Datenordner liegt im Kern"
```

---

### Task 5: Crate `launcher-ui`: Gerüst, Sprachdatei, Fehlertexte

**Files:**
- Modify: `Cargo.toml` (Arbeitsbereich: Mitglied `crates/launcher-ui`; `eframe = "0.37.0"`, `egui_kittest = "0.37.0"` unter `[workspace.dependencies]`)
- Create: `crates/launcher-ui/Cargo.toml`, `crates/launcher-ui/lang/de.json`, `crates/launcher-ui/src/lib.rs`, `src/i18n.rs`, `src/i18n_tests.rs`, `src/errors.rs`, `src/errors_tests.rs`, `src/main.rs` (zunächst Platzhalter `fn main() {}`)

**Interfaces:**
- Produces: `Texts::de() -> Texts`, `Texts::from_json(&str) -> Result<Texts, String>`, `Texts::t(&self, key) -> String` (fehlender Schlüssel gibt `⟦schlüssel⟧`), `Texts::tf(&self, key, &[(&str, &str)]) -> String` (ersetzt `{name}`), `Texts::has(&self, key) -> bool`; `errors::error_message(&Texts, &CoreError) -> String`.

- [ ] **Step 1: Gerüst anlegen** — `crates/launcher-ui/Cargo.toml`:

```toml
[package]
name = "launcher-ui"
version.workspace = true
edition.workspace = true
publish.workspace = true

[[bin]]
name = "daedyx-launcher"
path = "src/main.rs"

[dependencies]
launcher-core = { path = "../launcher-core" }
daedyx-update = { path = "../daedyx-update" }
eframe.workspace = true
clap.workspace = true
serde_json.workspace = true

[dev-dependencies]
egui_kittest.workspace = true
```

  `src/lib.rs`:

```rust
//! Oberfläche des Daedyx Launchers (egui). Zustand und Darstellung sind getrennt, damit beides testbar ist.

pub mod errors;
pub mod i18n;

#[cfg(test)]
mod errors_tests;
#[cfg(test)]
mod i18n_tests;
```

- [ ] **Step 2: Failing tests schreiben** — `i18n_tests.rs`:

```rust
use crate::i18n::Texts;

#[test]
fn texte_werden_geladen_und_platzhalter_ersetzt() {
    let t = Texts::from_json(r#"{"a":"Hallo {name}!","b":"ohne"}"#).unwrap();
    assert_eq!(t.t("b"), "ohne");
    assert_eq!(t.tf("a", &[("name", "Welt")]), "Hallo Welt!");
}

#[test]
fn fehlender_schluessel_ist_sichtbar() {
    let t = Texts::from_json("{}").unwrap();
    assert_eq!(t.t("x.y"), "⟦x.y⟧");
}

#[test]
fn kaputte_sprachdatei_wird_gemeldet() {
    assert!(Texts::from_json("nein").is_err());
}

#[test]
fn deutsche_sprachdatei_ist_gueltig_und_ihre_platzhalter_sind_ganz() {
    let t = Texts::de();
    for (key, text) in t.entries() {
        assert!(!text.trim().is_empty(), "{key} ist leer");
        let opens = text.matches('{').count();
        let closes = text.matches('}').count();
        assert_eq!(opens, closes, "{key}: Klammern passen nicht");
    }
}

/// Jeder im Code benutzte Textschlüssel muss in der Sprachdatei stehen.
#[test]
fn alle_im_code_benutzten_schluessel_existieren() {
    let t = Texts::de();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".rs") || name.ends_with("_tests.rs") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        for pat in [".t(\"", ".tf(\""] {
            for (i, _) in src.match_indices(pat) {
                let rest = &src[i + pat.len()..];
                let key = &rest[..rest.find('"').unwrap()];
                assert!(t.has(key), "{name}: Schlüssel {key:?} fehlt in de.json");
            }
        }
    }
}
```

  `errors_tests.rs`:

```rust
use crate::errors::error_message;
use crate::i18n::Texts;
use daedyx_update::error::UpdateError;
use launcher_core::error::CoreError;

fn all_variants() -> Vec<CoreError> {
    vec![
        UpdateError::CatalogInvalid("x".into()).into(),
        UpdateError::SchemaUnsupported { found: 2, expected: 1 }.into(),
        UpdateError::LauncherTooOld { need: "2.0.0".into(), have: "1.0.0".into() }.into(),
        UpdateError::SignatureInvalid.into(),
        UpdateError::ChecksumMismatch.into(),
        UpdateError::NoPackage { app: "desk".into(), platform: "windows-x64".into() }.into(),
        UpdateError::UnknownApp("desk".into()).into(),
        UpdateError::BadVersion("x".into()).into(),
        UpdateError::Io("x".into()).into(),
        CoreError::SourceNotAllowed("http://x".into()),
        CoreError::Download("x".into()),
        CoreError::InstallerFailed(5),
        CoreError::PlatformUnsupported("x".into()),
        CoreError::Busy("desk".into()),
        CoreError::NotInstalled("desk".into()),
        CoreError::NotLaunchable("x".into()),
        CoreError::Io("x".into()),
    ]
}

#[test]
fn jeder_fehler_hat_einen_vollstaendigen_deutschen_text() {
    let t = Texts::de();
    for e in all_variants() {
        let text = error_message(&t, &e);
        assert!(!text.contains('⟦'), "{e:?}: Schlüssel fehlt: {text}");
        assert!(!text.contains('{'), "{e:?}: Platzhalter nicht ersetzt: {text}");
        assert!(text.chars().count() > 20, "{e:?}: zu knapp: {text}");
    }
}

#[test]
fn sicherheitsfehler_sagen_klar_dass_nichts_passiert_ist() {
    let t = Texts::de();
    let sig = error_message(&t, &UpdateError::SignatureInvalid.into());
    assert!(sig.contains("Echtheit") && sig.contains("nichts"), "{sig}");
    let sum = error_message(&t, &UpdateError::ChecksumMismatch.into());
    assert!(sum.contains("nichts installiert"), "{sum}");
}
```

- [ ] **Step 3: Scheitern sehen:** `cargo test -p launcher-ui` — Expected: Übersetzungsfehler (`Texts`, `error_message` fehlen). (Erster Lauf lädt und baut eframe/wgpu, rund eine Minute.)

- [ ] **Step 4: `i18n.rs` schreiben:**

```rust
use std::collections::BTreeMap;

/// Alle Texte der Oberfläche stehen in `lang/de.json` (flache Schlüssel, `{name}`-Platzhalter).
pub struct Texts {
    map: BTreeMap<String, String>,
}

impl Texts {
    pub fn from_json(json: &str) -> Result<Texts, String> {
        let map: BTreeMap<String, String> =
            serde_json::from_str(json).map_err(|e| format!("Sprachdatei ungültig: {e}"))?;
        Ok(Texts { map })
    }

    pub fn de() -> Texts {
        Texts::from_json(include_str!("../lang/de.json")).expect("lang/de.json ist gültig")
    }

    pub fn has(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &String)> {
        self.map.iter()
    }

    /// Fehlt ein Schlüssel, steht er sichtbar in ⟦…⟧, damit der Fehler im Test und im Fenster auffällt.
    pub fn t(&self, key: &str) -> String {
        self.map.get(key).cloned().unwrap_or_else(|| format!("⟦{key}⟧"))
    }

    pub fn tf(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut text = self.t(key);
        for (name, value) in args {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
}
```

- [ ] **Step 5: `lang/de.json` schreiben** (alle Schlüssel, die Tasks 5–8 benutzen; bei neuen Schlüsseln hier ergänzen):

```json
{
  "app.title": "Daedyx Launcher",
  "header.title": "Deine Apps",
  "header.refresh": "Aktualisieren",
  "header.loading": "Katalog wird geladen …",
  "header.retry": "Erneut versuchen",
  "list.empty": "Der Katalog enthält noch keine Apps.",
  "state.not_installed": "Nicht installiert",
  "state.installed": "Installiert · Version {version}",
  "state.update": "Update verfügbar · {installed} → {latest}",
  "action.install": "Installieren",
  "action.update": "Aktualisieren",
  "action.open": "Öffnen",
  "action.remove": "Entfernen",
  "busy.downloading": "Lädt herunter … {percent} %",
  "busy.downloading_mb": "Lädt herunter … {mb} MB",
  "busy.installing": "Wird installiert …",
  "busy.removing": "Wird entfernt …",
  "confirm.remove.title": "{name} entfernen?",
  "confirm.remove.text": "Die App wird von diesem Computer entfernt.",
  "confirm.remove.yes": "Entfernen",
  "confirm.cancel": "Abbrechen",
  "notice.close": "Schließen",
  "notice.installed": "{name} {version} wurde installiert.",
  "notice.updated": "{name} wurde auf Version {to} aktualisiert.",
  "notice.uptodate": "{name} ist schon aktuell.",
  "notice.removed": "{name} wurde entfernt.",
  "notice.opened": "{name} wird gestartet.",
  "setup.missing": "Es ist noch keine Katalogquelle eingestellt. Starte den Launcher mit --catalog <Ordner oder Adresse> und --pubkey <Schlüsseldatei> (oder setze DAEDYX_CATALOG und DAEDYX_PUBKEY).",
  "setup.pubkey_unreadable": "Die Schlüsseldatei konnte nicht gelesen werden: {detail}",
  "error.signature": "Der Katalog konnte nicht auf Echtheit geprüft werden. Aus Sicherheitsgründen wurde nichts geladen und nichts verändert.",
  "error.checksum": "Die heruntergeladene Datei ist beschädigt oder wurde verändert. Es wurde nichts installiert. Bitte versuche es später noch einmal.",
  "error.download": "Der Download hat nicht geklappt. Prüfe die Internetverbindung und versuche es noch einmal. ({detail})",
  "error.busy": "Für {app} läuft gerade schon eine Installation oder ein Update. Warte, bis sie fertig ist.",
  "error.installer": "Das Setup wurde abgebrochen (Code {code}). Ist die App gerade geöffnet? Dann schließe sie und versuche es noch einmal.",
  "error.launcher_old": "Dieser Launcher ist zu alt für den Katalog (nötig: {need}, vorhanden: {have}). Bitte aktualisiere den Launcher.",
  "error.schema": "Der Katalog hat ein Format, das dieser Launcher nicht kennt. Bitte aktualisiere den Launcher.",
  "error.catalog_invalid": "Der Katalog ist fehlerhaft und wurde nicht verwendet. ({detail})",
  "error.no_package": "Für {app} gibt es noch kein Paket für diesen Computer ({platform}).",
  "error.unknown_app": "Die App {app} kennt der Katalog nicht.",
  "error.bad_version": "Eine Versionsnummer im Katalog ist ungültig: {version}",
  "error.platform": "Dieses Paket wird auf diesem Computer noch nicht unterstützt ({detail}).",
  "error.not_installed": "{app} ist nicht installiert.",
  "error.not_launchable": "Die App lässt sich nicht starten. ({detail})",
  "error.source": "Diese Quelle ist nicht erlaubt: {detail}. Für Internetadressen ist nur https:// zulässig.",
  "error.io": "Auf eine Datei konnte nicht zugegriffen werden. ({detail})"
}
```

- [ ] **Step 6: `errors.rs` schreiben:**

```rust
use crate::i18n::Texts;
use daedyx_update::error::UpdateError;
use launcher_core::error::CoreError;

/// Übersetzt einen Fehler des Kerns in einen verständlichen Text der Sprachdatei.
/// Die `match`-Ausdrücke sind vollständig: Kommt eine Variante hinzu, bricht die Übersetzung ab.
pub fn error_message(t: &Texts, e: &CoreError) -> String {
    match e {
        CoreError::Update(u) => match u {
            UpdateError::SignatureInvalid => t.t("error.signature"),
            UpdateError::ChecksumMismatch => t.t("error.checksum"),
            UpdateError::LauncherTooOld { need, have } => {
                t.tf("error.launcher_old", &[("need", need), ("have", have)])
            }
            UpdateError::SchemaUnsupported { .. } => t.t("error.schema"),
            UpdateError::CatalogInvalid(d) => t.tf("error.catalog_invalid", &[("detail", d)]),
            UpdateError::NoPackage { app, platform } => {
                t.tf("error.no_package", &[("app", app), ("platform", platform)])
            }
            UpdateError::UnknownApp(a) => t.tf("error.unknown_app", &[("app", a)]),
            UpdateError::BadVersion(v) => t.tf("error.bad_version", &[("version", v)]),
            UpdateError::Io(d) => t.tf("error.io", &[("detail", d)]),
        },
        CoreError::SourceNotAllowed(s) => t.tf("error.source", &[("detail", s)]),
        CoreError::Download(d) => t.tf("error.download", &[("detail", d)]),
        CoreError::InstallerFailed(code) => {
            t.tf("error.installer", &[("code", &code.to_string())])
        }
        CoreError::PlatformUnsupported(p) => t.tf("error.platform", &[("detail", p)]),
        CoreError::Busy(a) => t.tf("error.busy", &[("app", a)]),
        CoreError::NotInstalled(a) => t.tf("error.not_installed", &[("app", a)]),
        CoreError::NotLaunchable(d) => t.tf("error.not_launchable", &[("detail", d)]),
        CoreError::Io(d) => t.tf("error.io", &[("detail", d)]),
    }
}
```
  `src/main.rs`: `fn main() {}`.

- [ ] **Step 7: Tests laufen lassen**

Run: `cargo test -p launcher-ui` — Expected: 7 Tests grün. (`alle_im_code_benutzten_schluessel_existieren` greift erst, wenn `.t("…")`-Aufrufe vorkommen; er prüft `errors.rs` schon jetzt.)

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: Gerüst der Oberfläche mit deutscher Sprachdatei und Fehlertexten"
```

---

### Task 6: Zustand der Oberfläche (`model.rs`)

**Files:**
- Create: `crates/launcher-ui/src/model.rs`, `crates/launcher-ui/src/model_tests.rs`
- Modify: `crates/launcher-ui/src/lib.rs` (`pub mod model;`, `#[cfg(test)] mod model_tests;`)

**Interfaces:**
- Consumes: `launcher_core::jobs::Event`, `launcher_core::service::{AppStatus, Outcome}`, `daedyx_update::catalog::Catalog`, `Texts`, `error_message`.
- Produces:
  - `enum CardState { NotInstalled, Installed, UpdateAvailable }` (`Clone, Copy, Debug, PartialEq`)
  - `struct Card { id, name, description: String, installed: Option<String>, latest: Option<String>, state: CardState }` (`Clone, Debug, PartialEq`)
  - `enum Busy { Downloading { done: u64, total: Option<u64> }, Installing, Removing }` (`Clone, Debug, PartialEq`)
  - `enum Load { Loading, Ready, Failed(String) }` (`Debug, PartialEq`), `enum Notice { Info(String), Error(String) }` (`Debug, PartialEq`)
  - `struct Model { pub load: Load, pub cards: Vec<Card>, pub busy: BTreeMap<String, Busy>, pub notice: Option<Notice> }` mit `Model::new()`, `begin(&mut self, app: &str, busy: Busy)`, `refreshing(&mut self)`, `apply(&mut self, ev: Event, t: &Texts)`.

- [ ] **Step 1: Failing tests schreiben** — `model_tests.rs`:

```rust
use crate::i18n::Texts;
use crate::model::{Busy, CardState, Load, Model, Notice};
use daedyx_update::catalog::{App, Catalog};
use daedyx_update::error::UpdateError;
use launcher_core::error::CoreError;
use launcher_core::jobs::Event;
use launcher_core::service::{AppStatus, Outcome};

fn catalog() -> Catalog {
    Catalog {
        schema: 1,
        min_launcher_version: "0.1.0".into(),
        apps: vec![App {
            id: "notizbuch".into(),
            name: "Notizbuch".into(),
            description: "Kurze Notizen.".into(),
            license: String::new(),
            releases: vec![],
        }],
    }
}

fn status(installed: Option<&str>, latest: &str, update: bool) -> Event {
    Event::Status(vec![AppStatus {
        id: "notizbuch".into(),
        name: "Notizbuch".into(),
        installed: installed.map(String::from),
        latest: Some(latest.into()),
        update_available: update,
    }])
}

fn ready(installed: Option<&str>, update: bool) -> (Model, Texts) {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(Event::Catalog(Ok(catalog())), &t);
    m.apply(status(installed, "1.1.0", update), &t);
    (m, t)
}

#[test]
fn am_anfang_wird_geladen() {
    assert_eq!(Model::new().load, Load::Loading);
}

#[test]
fn katalog_und_status_ergeben_karten() {
    let (m, _) = ready(None, false);
    assert_eq!(m.load, Load::Ready);
    assert_eq!(m.cards.len(), 1);
    assert_eq!(m.cards[0].name, "Notizbuch");
    assert_eq!(m.cards[0].description, "Kurze Notizen.");
    assert_eq!(m.cards[0].state, CardState::NotInstalled);
}

#[test]
fn installierte_und_veraltete_apps_haben_ihren_zustand() {
    let (m, _) = ready(Some("1.1.0"), false);
    assert_eq!(m.cards[0].state, CardState::Installed);
    let (m, _) = ready(Some("1.0.0"), true);
    assert_eq!(m.cards[0].state, CardState::UpdateAvailable);
}

#[test]
fn fehler_beim_ersten_laden_ersetzt_die_ansicht() {
    let t = Texts::de();
    let mut m = Model::new();
    m.apply(Event::Catalog(Err(UpdateError::SignatureInvalid.into())), &t);
    match &m.load {
        Load::Failed(text) => assert!(text.contains("Echtheit")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn fehler_beim_neuladen_behaelt_die_karten() {
    let (mut m, t) = ready(None, false);
    m.apply(Event::Catalog(Err(CoreError::Download("weg".into()))), &t);
    assert_eq!(m.load, Load::Ready);
    assert_eq!(m.cards.len(), 1);
    assert!(matches!(m.notice, Some(Notice::Error(_))));
}

#[test]
fn fortschritt_setzt_busy_und_wechselt_bei_100_prozent_zum_installieren() {
    let (mut m, t) = ready(None, false);
    m.begin("notizbuch", Busy::Downloading { done: 0, total: None });
    m.apply(Event::Progress { app: "notizbuch".into(), done: 50, total: Some(100) }, &t);
    assert_eq!(m.busy["notizbuch"], Busy::Downloading { done: 50, total: Some(100) });
    m.apply(Event::Progress { app: "notizbuch".into(), done: 100, total: Some(100) }, &t);
    assert_eq!(m.busy["notizbuch"], Busy::Installing);
}

#[test]
fn erfolgreiche_installation_raeumt_auf_und_meldet() {
    let (mut m, t) = ready(None, false);
    m.begin("notizbuch", Busy::Installing);
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Ok(Outcome::Installed { version: "1.1.0".into() }),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    assert_eq!(m.notice, Some(Notice::Info("Notizbuch 1.1.0 wurde installiert.".into())));
}

#[test]
fn update_und_schon_aktuell_haben_eigene_hinweise() {
    let (mut m, t) = ready(Some("1.0.0"), true);
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Ok(Outcome::Updated { from: "1.0.0".into(), to: "1.1.0".into() }),
        },
        &t,
    );
    assert_eq!(m.notice, Some(Notice::Info("Notizbuch wurde auf Version 1.1.0 aktualisiert.".into())));
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Ok(Outcome::UpToDate { version: "1.1.0".into() }),
        },
        &t,
    );
    assert_eq!(m.notice, Some(Notice::Info("Notizbuch ist schon aktuell.".into())));
}

#[test]
fn fehlgeschlagene_installation_zeigt_fehlertext_und_gibt_die_karte_frei() {
    let (mut m, t) = ready(None, false);
    m.begin("notizbuch", Busy::Installing);
    m.apply(
        Event::Installed {
            app: "notizbuch".into(),
            result: Err(UpdateError::ChecksumMismatch.into()),
        },
        &t,
    );
    assert!(m.busy.is_empty());
    match m.notice {
        Some(Notice::Error(text)) => assert!(text.contains("nichts installiert")),
        other => panic!("{other:?}"),
    }
}

#[test]
fn entfernen_und_oeffnen_melden_ergebnis() {
    let (mut m, t) = ready(Some("1.0.0"), false);
    m.begin("notizbuch", Busy::Removing);
    m.apply(Event::Removed { app: "notizbuch".into(), result: Ok(()) }, &t);
    assert!(m.busy.is_empty());
    assert_eq!(m.notice, Some(Notice::Info("Notizbuch wurde entfernt.".into())));
    m.apply(Event::Opened { app: "notizbuch".into(), result: Ok(()) }, &t);
    assert_eq!(m.notice, Some(Notice::Info("Notizbuch wird gestartet.".into())));
    m.apply(
        Event::Opened { app: "notizbuch".into(), result: Err(CoreError::NotInstalled("notizbuch".into())) },
        &t,
    );
    assert!(matches!(m.notice, Some(Notice::Error(_))));
}

#[test]
fn neue_aktion_loescht_den_alten_hinweis() {
    let (mut m, _) = ready(None, false);
    m.notice = Some(Notice::Info("alt".into()));
    m.begin("notizbuch", Busy::Downloading { done: 0, total: None });
    assert_eq!(m.notice, None);
}
```

- [ ] **Step 2: Scheitern sehen:** `cargo test -p launcher-ui model` — Expected: Modul fehlt.

- [ ] **Step 3: `model.rs` schreiben:**

```rust
use crate::errors::error_message;
use crate::i18n::Texts;
use daedyx_update::catalog::Catalog;
use launcher_core::jobs::Event;
use launcher_core::service::{AppStatus, Outcome};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CardState {
    NotInstalled,
    Installed,
    UpdateAvailable,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub description: String,
    pub installed: Option<String>,
    pub latest: Option<String>,
    pub state: CardState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Busy {
    Downloading { done: u64, total: Option<u64> },
    Installing,
    Removing,
}

#[derive(Debug, PartialEq)]
pub enum Load {
    Loading,
    Ready,
    Failed(String),
}

#[derive(Debug, PartialEq)]
pub enum Notice {
    Info(String),
    Error(String),
}

/// Was die Oberfläche zeigt. Kennt egui nicht und wird nur durch Ereignisse des Kerns verändert.
pub struct Model {
    pub load: Load,
    pub cards: Vec<Card>,
    pub busy: BTreeMap<String, Busy>,
    pub notice: Option<Notice>,
    catalog: Option<Catalog>,
}

impl Default for Model {
    fn default() -> Self {
        Model::new()
    }
}

impl Model {
    pub fn new() -> Model {
        Model { load: Load::Loading, cards: Vec::new(), busy: BTreeMap::new(), notice: None, catalog: None }
    }

    /// Vor dem Auftrag an den Arbeiter aufrufen.
    pub fn begin(&mut self, app: &str, busy: Busy) {
        self.notice = None;
        self.busy.insert(app.to_string(), busy);
    }

    pub fn refreshing(&mut self) {
        self.notice = None;
        if self.cards.is_empty() {
            self.load = Load::Loading;
        }
    }

    fn name_of(&self, id: &str) -> String {
        self.cards.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or_else(|| id.to_string())
    }

    fn build_cards(&mut self, status: &[AppStatus]) {
        let Some(catalog) = &self.catalog else { return };
        self.cards = catalog
            .apps
            .iter()
            .filter_map(|app| {
                let s = status.iter().find(|s| s.id == app.id)?;
                let state = match (&s.installed, s.update_available) {
                    (None, _) => CardState::NotInstalled,
                    (Some(_), true) => CardState::UpdateAvailable,
                    (Some(_), false) => CardState::Installed,
                };
                Some(Card {
                    id: app.id.clone(),
                    name: app.name.clone(),
                    description: app.description.clone(),
                    installed: s.installed.clone(),
                    latest: s.latest.clone(),
                    state,
                })
            })
            .collect();
    }

    pub fn apply(&mut self, ev: Event, t: &Texts) {
        match ev {
            Event::Catalog(Ok(c)) => {
                self.catalog = Some(c);
                self.load = Load::Ready;
            }
            Event::Catalog(Err(e)) => {
                let text = error_message(t, &e);
                if self.cards.is_empty() {
                    self.load = Load::Failed(text);
                } else {
                    self.notice = Some(Notice::Error(text));
                }
            }
            Event::Status(s) => self.build_cards(&s),
            Event::Progress { app, done, total } => {
                let finished = total.is_some_and(|tot| tot > 0 && done >= tot);
                let busy = if finished { Busy::Installing } else { Busy::Downloading { done, total } };
                self.busy.insert(app, busy);
            }
            Event::Installed { app, result } => {
                self.busy.remove(&app);
                let name = self.name_of(&app);
                self.notice = Some(match result {
                    Ok(Outcome::Installed { version }) => {
                        Notice::Info(t.tf("notice.installed", &[("name", &name), ("version", &version)]))
                    }
                    Ok(Outcome::Updated { to, .. }) => {
                        Notice::Info(t.tf("notice.updated", &[("name", &name), ("to", &to)]))
                    }
                    Ok(Outcome::UpToDate { .. }) => Notice::Info(t.tf("notice.uptodate", &[("name", &name)])),
                    Err(e) => Notice::Error(error_message(t, &e)),
                });
            }
            Event::Removed { app, result } => {
                self.busy.remove(&app);
                let name = self.name_of(&app);
                self.notice = Some(match result {
                    Ok(()) => Notice::Info(t.tf("notice.removed", &[("name", &name)])),
                    Err(e) => Notice::Error(error_message(t, &e)),
                });
            }
            Event::Opened { app, result } => {
                let name = self.name_of(&app);
                self.notice = Some(match result {
                    Ok(()) => Notice::Info(t.tf("notice.opened", &[("name", &name)])),
                    Err(e) => Notice::Error(error_message(t, &e)),
                });
            }
        }
    }
}
```

- [ ] **Step 4: Tests laufen lassen**

Run: `cargo test -p launcher-ui` — Expected: alles grün (10 neue Tests in `model_tests`).

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: Zustand der Oberfläche aus Kern-Ereignissen"
```

---

### Task 7: Kartenansicht (`card_view.rs`) mit Oberflächentests

**Files:**
- Create: `crates/launcher-ui/src/card_view.rs`, `crates/launcher-ui/src/card_view_tests.rs`
- Modify: `crates/launcher-ui/src/lib.rs`

**Interfaces:**
- Consumes: `Card`, `CardState`, `Busy`, `Texts`.
- Produces: `enum Action { Install(String), Update(String), Open(String), Remove(String) }` (`Debug, PartialEq, Clone`); `card_view::show(ui: &mut egui::Ui, t: &Texts, card: &Card, busy: Option<&Busy>) -> Option<Action>`; `card_view::CARD_WIDTH: f32`.

- [ ] **Step 1: Failing tests schreiben** — `card_view_tests.rs`. Muster wie im Prototyp: Closure fängt `&mut Option<Action>`, Harness vor der Prüfung freigeben.

```rust
use crate::card_view::{show, Action};
use crate::i18n::Texts;
use crate::model::{Busy, Card, CardState};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

fn card(state: CardState) -> Card {
    Card {
        id: "notizbuch".into(),
        name: "Notizbuch".into(),
        description: "Kurze Notizen sammeln.".into(),
        installed: (state != CardState::NotInstalled).then(|| "1.0.0".to_string()),
        latest: Some("1.1.0".into()),
        state,
    }
}

/// Zeichnet die Karte, klickt optional einen Knopf und gibt die gemeldete Aktion zurück.
fn click(state: CardState, busy: Option<Busy>, label: Option<&str>) -> Option<Action> {
    let t = Texts::de();
    let c = card(state);
    let mut action = None;
    {
        let mut h = Harness::new_ui(|ui| {
            if let Some(a) = show(ui, &t, &c, busy.as_ref()) {
                action = Some(a);
            }
        });
        if let Some(l) = label {
            h.get_by_label(l).click();
        }
        h.run();
    }
    action
}

fn has_label(state: CardState, busy: Option<Busy>, label: &str) -> bool {
    let t = Texts::de();
    let c = card(state);
    let h = Harness::new_ui(|ui| {
        show(ui, &t, &c, busy.as_ref());
    });
    h.query_by_label(label).is_some()
}

#[test]
fn nicht_installiert_bietet_installieren() {
    assert!(has_label(CardState::NotInstalled, None, "Installieren"));
    assert!(!has_label(CardState::NotInstalled, None, "Öffnen"));
    assert_eq!(
        click(CardState::NotInstalled, None, Some("Installieren")),
        Some(Action::Install("notizbuch".into()))
    );
}

#[test]
fn installiert_bietet_oeffnen_und_entfernen() {
    assert_eq!(click(CardState::Installed, None, Some("Öffnen")), Some(Action::Open("notizbuch".into())));
    assert_eq!(click(CardState::Installed, None, Some("Entfernen")), Some(Action::Remove("notizbuch".into())));
    assert!(!has_label(CardState::Installed, None, "Installieren"));
}

#[test]
fn update_bietet_aktualisieren() {
    assert_eq!(
        click(CardState::UpdateAvailable, None, Some("Aktualisieren")),
        Some(Action::Update("notizbuch".into()))
    );
    assert!(has_label(CardState::UpdateAvailable, None, "Öffnen"));
}

#[test]
fn statustext_nennt_die_versionen() {
    assert!(has_label(CardState::Installed, None, "Installiert · Version 1.0.0"));
    assert!(has_label(CardState::UpdateAvailable, None, "Update verfügbar · 1.0.0 → 1.1.0"));
    assert!(has_label(CardState::NotInstalled, None, "Nicht installiert"));
}

#[test]
fn name_und_kurztext_stehen_auf_der_karte() {
    assert!(has_label(CardState::NotInstalled, None, "Notizbuch"));
    assert!(has_label(CardState::NotInstalled, None, "Kurze Notizen sammeln."));
}

#[test]
fn waehrend_des_downloads_gibt_es_keine_knoepfe_aber_fortschritt() {
    let busy = Some(Busy::Downloading { done: 50, total: Some(100) });
    assert!(!has_label(CardState::NotInstalled, busy.clone(), "Installieren"));
    assert!(has_label(CardState::NotInstalled, busy, "Lädt herunter … 50 %"));
}

#[test]
fn waehrend_installation_und_entfernen_steht_der_zustand_da() {
    assert!(has_label(CardState::NotInstalled, Some(Busy::Installing), "Wird installiert …"));
    assert!(has_label(CardState::Installed, Some(Busy::Removing), "Wird entfernt …"));
}
```

- [ ] **Step 2: Scheitern sehen:** `cargo test -p launcher-ui card_view` — Expected: Modul fehlt.

- [ ] **Step 3: `card_view.rs` schreiben** (API-Details gegen eframe 0.37 beim Bauen prüfen):

```rust
use crate::i18n::Texts;
use crate::model::{Busy, Card, CardState};
use eframe::egui::{self, Align2, Color32, FontId, ProgressBar, RichText, Sense, Vec2};

pub const CARD_WIDTH: f32 = 280.0;
const CARD_MIN_HEIGHT: f32 = 170.0;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Install(String),
    Update(String),
    Open(String),
    Remove(String),
}

/// Zeichnet eine Karte. Meldet höchstens eine Aktion pro Bild.
pub fn show(ui: &mut egui::Ui, t: &Texts, card: &Card, busy: Option<&Busy>) -> Option<Action> {
    let mut action = None;
    egui::Frame::group(ui.style()).inner_margin(14.0).show(ui, |ui| {
        ui.set_width(CARD_WIDTH);
        ui.set_min_height(CARD_MIN_HEIGHT);
        ui.horizontal(|ui| {
            badge(ui, card);
            ui.vertical(|ui| {
                ui.label(RichText::new(&card.name).strong().size(18.0));
                ui.label(RichText::new(state_text(t, card)).small());
            });
        });
        ui.add_space(6.0);
        ui.add(egui::Label::new(&card.description).wrap());
        ui.add_space(8.0);
        match busy {
            Some(b) => busy_view(ui, t, b),
            None => action = buttons(ui, t, card),
        }
    });
    action
}

fn state_text(t: &Texts, card: &Card) -> String {
    let installed = card.installed.as_deref().unwrap_or("");
    let latest = card.latest.as_deref().unwrap_or("");
    match card.state {
        CardState::NotInstalled => t.t("state.not_installed"),
        CardState::Installed => t.tf("state.installed", &[("version", installed)]),
        CardState::UpdateAvailable => t.tf("state.update", &[("installed", installed), ("latest", latest)]),
    }
}

/// Buchstabenmarke in einer Farbe, die aus der App-Kennung abgeleitet wird (Symbole als Bild: spätere Version).
fn badge(ui: &mut egui::Ui, card: &Card) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(44.0), Sense::hover());
    let hue = card.id.bytes().fold(7u32, |h, b| h.wrapping_mul(31).wrapping_add(b as u32)) % 360;
    let color: Color32 = egui::ecolor::Hsva::new(hue as f32 / 360.0, 0.55, 0.55, 1.0).into();
    ui.painter().circle_filled(rect.center(), 22.0, color);
    let initial = card.name.chars().next().unwrap_or('?').to_uppercase().to_string();
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, initial, FontId::proportional(22.0), Color32::WHITE);
}

fn busy_view(ui: &mut egui::Ui, t: &Texts, busy: &Busy) {
    match busy {
        Busy::Downloading { done, total } => match total.filter(|x| *x > 0) {
            Some(total) => {
                let frac = (*done as f32 / total as f32).clamp(0.0, 1.0);
                ui.add(ProgressBar::new(frac));
                ui.label(t.tf("busy.downloading", &[("percent", &format!("{:.0}", frac * 100.0))]));
            }
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(t.tf("busy.downloading_mb", &[("mb", &(done / (1024 * 1024)).to_string())]));
                });
            }
        },
        Busy::Installing => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t.t("busy.installing"));
            });
        }
        Busy::Removing => {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(t.t("busy.removing"));
            });
        }
    }
}

fn buttons(ui: &mut egui::Ui, t: &Texts, card: &Card) -> Option<Action> {
    let id = card.id.clone();
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        let primary = |ui: &mut egui::Ui, key: &str| ui.button(RichText::new(t.t(key)).strong()).clicked();
        let plain = |ui: &mut egui::Ui, key: &str| ui.button(t.t(key)).clicked();
        match card.state {
            CardState::NotInstalled => {
                if primary(ui, "action.install") {
                    action = Some(Action::Install(id.clone()));
                }
            }
            CardState::UpdateAvailable => {
                if primary(ui, "action.update") {
                    action = Some(Action::Update(id.clone()));
                }
                if plain(ui, "action.open") {
                    action = Some(Action::Open(id.clone()));
                }
                if plain(ui, "action.remove") {
                    action = Some(Action::Remove(id.clone()));
                }
            }
            CardState::Installed => {
                if primary(ui, "action.open") {
                    action = Some(Action::Open(id.clone()));
                }
                if plain(ui, "action.remove") {
                    action = Some(Action::Remove(id.clone()));
                }
            }
        }
    });
    action
}
```

- [ ] **Step 4: Tests laufen lassen**

Run: `cargo test -p launcher-ui` — Expected: alles grün (7 neue Tests in `card_view_tests`). Die `.t(\"…\")`-Schlüssel prüft `alle_im_code_benutzten_schluessel_existieren` jetzt auch für `card_view.rs`; fehlende Schlüssel in `de.json` ergänzen (`state.*`, `action.*`, `busy.*` stehen schon darin).

  Falls ein Label wegen doppelter Treffer nicht eindeutig ist (`get_by_label` meldet „multiple“), den Test auf `get_all_by_label(...).next()` umstellen und das hier festhalten.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: Karte mit Status, Fortschritt und Knöpfen"
```

---

### Task 8: Fenster, Thema und Start (`app.rs`, `theme.rs`, `main.rs`)

**Files:**
- Create: `crates/launcher-ui/src/theme.rs`, `crates/launcher-ui/src/app.rs`
- Modify: `crates/launcher-ui/src/main.rs`, `crates/launcher-ui/src/lib.rs` (`pub mod app; pub mod card_view; pub mod theme;`), `crates/launcher-ui/lang/de.json` (nur falls neue Schlüssel nötig werden)

**Interfaces:**
- Consumes: `Worker`, `Job`, `Model`, `Action`, `Texts`.
- Produces: `app::LauncherApp::new(t: Texts, worker: Result<Worker, String>) -> LauncherApp` (implementiert `eframe::App`); `theme::apply(ctx: &egui::Context)`.

Dieser Task hat keinen eigenen Einheitstest: Zustand und Karten sind in Task 6/7 getestet, das Fenster wird in Task 10 im echten Fenster geprüft. Eine Ausnahme: ein kittest-Rauchtest, dass die App mit einem fehlenden Arbeiter die Setup-Meldung zeigt.

- [ ] **Step 1: Rauchtest schreiben** — `crates/launcher-ui/src/app_tests.rs` (in `lib.rs`: `#[cfg(test)] mod app_tests;`):

```rust
use crate::app::LauncherApp;
use crate::i18n::Texts;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;

#[test]
fn ohne_katalogquelle_erklaert_das_fenster_was_fehlt() {
    let t = Texts::de();
    let msg = t.t("setup.missing");
    let mut app = LauncherApp::new(t, Err(msg.clone()));
    let mut h = Harness::new_ui(|ui| app.draw(ui));
    h.run();
    assert!(h.query_by_label_contains("--catalog").is_some());
}
```
  (`LauncherApp::draw(&mut self, ui: &mut egui::Ui)` ist die Zeichenfunktion; `eframe::App::ui` ruft sie auf, damit der Test ohne `Frame` auskommt.)

- [ ] **Step 2: Scheitern sehen:** `cargo test -p launcher-ui ohne_katalogquelle` — Expected: Modul `app` fehlt.

- [ ] **Step 3: `theme.rs` schreiben:**

```rust
use eframe::egui;

/// Hell oder dunkel folgt Windows. Zum Prüfen lässt sich das mit `DAEDYX_THEME=dark|light` überstimmen.
pub fn apply(ctx: &egui::Context) {
    let pref = match std::env::var("DAEDYX_THEME").as_deref() {
        Ok("dark") => egui::ThemePreference::Dark,
        Ok("light") => egui::ThemePreference::Light,
        _ => egui::ThemePreference::System,
    };
    ctx.set_theme(pref);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        for (text_style, font) in style.text_styles.iter_mut() {
            if matches!(text_style, egui::TextStyle::Body | egui::TextStyle::Button) {
                font.size = 15.0;
            }
        }
    });
}
```
  (`all_styles_mut` ggf. gegen die in 0.37 vorhandene Form tauschen, Compiler zeigt es.)

- [ ] **Step 4: `app.rs` schreiben:**

```rust
use crate::card_view::{self, Action, CARD_WIDTH};
use crate::i18n::Texts;
use crate::model::{Busy, Load, Model, Notice};
use eframe::egui::{self, Color32, RichText};
use launcher_core::jobs::{Job, Worker};

pub struct LauncherApp {
    t: Texts,
    model: Model,
    worker: Option<Worker>,
    confirm_remove: Option<String>,
}

impl LauncherApp {
    /// `worker` ist `Err(Text)`, wenn der Start nicht möglich war (zum Beispiel fehlende Katalogquelle).
    pub fn new(t: Texts, worker: Result<Worker, String>) -> LauncherApp {
        let mut model = Model::new();
        let worker = match worker {
            Ok(w) => {
                w.send(Job::Refresh);
                Some(w)
            }
            Err(text) => {
                model.load = Load::Failed(text);
                None
            }
        };
        LauncherApp { t, model, worker, confirm_remove: None }
    }

    fn poll(&mut self) {
        if let Some(w) = &self.worker {
            while let Some(ev) = w.try_recv() {
                self.model.apply(ev, &self.t);
            }
        }
    }

    fn send(&mut self, job: Job) {
        if let Some(w) = &self.worker {
            w.send(job);
        }
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::Install(id) | Action::Update(id) => {
                self.model.begin(&id, Busy::Downloading { done: 0, total: None });
                self.send(Job::Install(id));
            }
            Action::Open(id) => self.send(Job::Open(id)),
            Action::Remove(id) => self.confirm_remove = Some(id),
        }
    }

    fn refresh(&mut self) {
        self.model.refreshing();
        self.send(Job::Refresh);
    }

    /// Zeichnet das ganze Fenster in das gegebene `Ui`.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        self.poll();
        egui::CentralPanel::default().show(ui, |ui| {
            self.header(ui);
            self.notice(ui);
            self.body(ui);
        });
        self.confirm_dialog(ui.ctx());
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading(self.t.t("header.title"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(self.t.t("header.refresh")).clicked() {
                    self.refresh();
                }
            });
        });
        ui.separator();
    }

    fn notice(&mut self, ui: &mut egui::Ui) {
        let Some(notice) = &self.model.notice else { return };
        let (text, fill) = match notice {
            Notice::Info(t) => (t.clone(), Color32::from_rgba_unmultiplied(60, 140, 90, 50)),
            Notice::Error(t) => (t.clone(), Color32::from_rgba_unmultiplied(200, 60, 60, 60)),
        };
        let mut close = false;
        egui::Frame::new().fill(fill).corner_radius(6.0).inner_margin(10.0).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(text);
                if ui.button(self.t.t("notice.close")).clicked() {
                    close = true;
                }
            });
        });
        if close {
            self.model.notice = None;
        }
    }

    fn body(&mut self, ui: &mut egui::Ui) {
        match &self.model.load {
            Load::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(self.t.t("header.loading"));
                });
            }
            Load::Failed(text) => {
                ui.label(RichText::new(text.clone()));
                if ui.button(self.t.t("header.retry")).clicked() {
                    self.refresh();
                }
            }
            Load::Ready if self.model.cards.is_empty() => {
                ui.label(self.t.t("list.empty"));
            }
            Load::Ready => self.grid(ui),
        }
    }

    fn grid(&mut self, ui: &mut egui::Ui) {
        let mut chosen = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.set_max_width(ui.available_width().max(CARD_WIDTH));
                for card in &self.model.cards {
                    let busy = self.model.busy.get(&card.id);
                    if let Some(a) = card_view::show(ui, &self.t, card, busy) {
                        chosen = Some(a);
                    }
                }
            });
        });
        if let Some(a) = chosen {
            self.dispatch(a);
        }
    }

    fn confirm_dialog(&mut self, ctx: &egui::Context) {
        let Some(id) = self.confirm_remove.clone() else { return };
        let name = self.model.cards.iter().find(|c| c.id == id).map(|c| c.name.clone()).unwrap_or(id.clone());
        let mut done = false;
        egui::Window::new(self.t.tf("confirm.remove.title", &[("name", &name)]))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(self.t.t("confirm.remove.text"));
                ui.horizontal(|ui| {
                    if ui.button(RichText::new(self.t.t("confirm.remove.yes")).strong()).clicked() {
                        self.model.begin(&id, Busy::Removing);
                        if let Some(w) = &self.worker {
                            w.send(Job::Remove(id.clone()));
                        }
                        done = true;
                    }
                    if ui.button(self.t.t("confirm.cancel")).clicked() {
                        done = true;
                    }
                });
            });
        if done {
            self.confirm_remove = None;
        }
    }
}

impl eframe::App for LauncherApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}
```

- [ ] **Step 5: `main.rs` schreiben:**

```rust
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use clap::Parser;
use daedyx_update::catalog::current_platform;
use eframe::egui;
use launcher_core::installer::SystemInstaller;
use launcher_core::jobs::{Wake, Worker};
use launcher_core::paths::default_data_dir;
use launcher_core::service::Launcher;
use launcher_core::source::Source;
use launcher_ui::app::LauncherApp;
use launcher_ui::errors::error_message;
use launcher_ui::i18n::Texts;
use launcher_ui::theme;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "daedyx-launcher", version, about = "Daedyx Launcher")]
struct Args {
    /// Katalogquelle: Ordner oder Internetadresse
    #[arg(long, env = "DAEDYX_CATALOG")]
    catalog: Option<String>,
    /// Öffentlicher minisign-Schlüssel des Katalogs (Datei)
    #[arg(long, env = "DAEDYX_PUBKEY")]
    pubkey: Option<PathBuf>,
    /// Datenordner des Launchers
    #[arg(long, env = "DAEDYX_DATA_DIR")]
    data_dir: Option<PathBuf>,
}

fn build_worker(args: &Args, t: &Texts, wake: Wake) -> Result<Worker, String> {
    let (Some(catalog), Some(pubkey_path)) = (&args.catalog, &args.pubkey) else {
        return Err(t.t("setup.missing"));
    };
    let pubkey = std::fs::read_to_string(pubkey_path)
        .map_err(|e| t.tf("setup.pubkey_unreadable", &[("detail", &e.to_string())]))?;
    let launcher = Launcher {
        source: Source::parse(catalog).map_err(|e| error_message(t, &e))?,
        platform: current_platform().to_string(),
        data_dir: args.data_dir.clone().unwrap_or_else(default_data_dir),
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
        installer: Box::new(SystemInstaller),
    };
    Ok(Worker::start(launcher, pubkey, wake))
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let t = Texts::de();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(t.t("app.title"))
            .with_inner_size([940.0, 640.0])
            .with_min_inner_size([420.0, 360.0]),
        ..Default::default()
    };
    eframe::run_native(
        &t.t("app.title"),
        options,
        Box::new(move |cc| {
            theme::apply(&cc.egui_ctx);
            let ctx = cc.egui_ctx.clone();
            let wake: Wake = Arc::new(move || ctx.request_repaint());
            let worker = build_worker(&args, &t, wake);
            Ok(Box::new(LauncherApp::new(t, worker)))
        }),
    )
}
```

- [ ] **Step 6: Tests und Bauen**

Run: `cargo test -p launcher-ui` — Expected: alles grün (inkl. Rauchtest).
Run: `cargo build -p launcher-ui` — Expected: baut ohne Warnung. `cargo clippy -p launcher-ui --all-targets` — Expected: sauber.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: Fenster mit Karten-Raster, Hinweisleiste und Entfernen-Rückfrage"
```

---

### Task 9: Demo-Katalog mit langsamem Testserver (nur Test-Helfer)

**Files:**
- Create: `crates/launcher-cli/src/bin/demo_catalog.rs`
- Modify: `crates/launcher-cli/Cargo.toml`

**Interfaces:** Programm `demo-catalog` (nur mit Feature `test-helpers`):
- `demo-catalog generate <ordner> [--stage 1|2]` erzeugt 4 erfundene Apps (`notizbuch`, `zeichenbrett`, `stundenplan`, `kassenbuch`) mit Test-Installern (je ca. 6 MB), Testschlüssel (`test-secret.key`, `pub.key` im Ordner, bleibt bei erneutem Aufruf gleich) und signiertem Katalog. Stufe 1: je Version `1.0.0`. Stufe 2: zusätzlich `1.1.0` für `notizbuch` und `stundenplan`.
- `demo-catalog serve <ordner> [--port 8765] [--kbps 640]` liefert den Ordner auf `127.0.0.1` absichtlich langsam aus (sichtbarer Fortschritt).

- [ ] **Step 1: Cargo anpassen** — `launcher-cli/Cargo.toml`:

```toml
[[bin]]
name = "demo-catalog"
path = "src/bin/demo_catalog.rs"
required-features = ["test-helpers"]

[features]
test-helpers = ["dep:minisign"]

[dependencies]
minisign = { workspace = true, optional = true }
```
  (die Zeile `test-helpers = []` ersetzen; `minisign` bleibt zusätzlich unter `[dev-dependencies]`.)

- [ ] **Step 2: Prüfbarer Kern zuerst (Test):** die Funktion, die den Pfad einer Anfrage bereinigt, ist sicherheitsrelevant (Server liefert nur Dateien unterhalb des Ordners). Am Ende von `demo_catalog.rs` einen Test:

```rust
#[cfg(test)]
mod tests {
    use super::request_path;

    #[test]
    fn pfad_bleibt_im_ordner() {
        assert_eq!(request_path("GET /catalog.json HTTP/1.1"), Some("catalog.json".to_string()));
        assert_eq!(request_path("GET /packages/a-1.0.0.exe HTTP/1.1"), Some("packages/a-1.0.0.exe".to_string()));
        assert_eq!(request_path("GET /../geheim HTTP/1.1"), None);
        assert_eq!(request_path("GET /a/../../b HTTP/1.1"), None);
        assert_eq!(request_path("GET /c:/x HTTP/1.1"), None);
        assert_eq!(request_path("POST /x HTTP/1.1"), None);
    }
}
```

- [ ] **Step 3: Programm schreiben** (Signaturen der `minisign`-Aufrufe gegen den Compiler prüfen; Muster für `sign` steht in `service_tests`/`test_support.rs`):

```rust
//! Demo-Katalog für die Sichtprüfung: erfundene Apps, Testschlüssel, absichtlich langsamer Server.
//! Nur für Entwicklung; nichts davon gehört in die Veröffentlichung.
use clap::{Parser, Subcommand};
use daedyx_update::verify::sha256_file;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::time::Duration;

const APPS: [(&str, &str, &str); 4] = [
    ("notizbuch", "Notizbuch", "Kurze Notizen sammeln und schnell wiederfinden."),
    ("zeichenbrett", "Zeichenbrett", "Ein großes Brett für Skizzen und Ideen."),
    ("stundenplan", "Stundenplan", "Der Wochenplan mit Erinnerungen, ganz ohne Konto."),
    ("kassenbuch", "Kassenbuch", "Einnahmen und Ausgaben für den kleinen Verein."),
];
const PADDING_BYTES: usize = 6 * 1024 * 1024;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Generate {
        dir: PathBuf,
        #[arg(long, default_value_t = 1)]
        stage: u8,
    },
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
        let sk = minisign::SecretKeyBox::from_string(&text).unwrap().into_unencrypted_secret_key().unwrap();
        let pk = minisign::PublicKeyBox::from_string(&pk_text).unwrap().into_public_key().unwrap();
        return (pk, sk);
    }
    let kp = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
    std::fs::write(dir.join("pub.key"), kp.pk.to_box().unwrap().to_string()).unwrap();
    std::fs::write(&secret_path, kp.sk.to_box(None).unwrap().to_string()).unwrap();
    (kp.pk, kp.sk)
}

fn generate(dir: &Path, stage: u8) {
    std::fs::create_dir_all(dir.join("packages")).unwrap();
    let fake = std::env::current_exe().unwrap().with_file_name("fake-installer.exe");
    let base = std::fs::read(&fake).expect("fake-installer.exe fehlt: mit --features test-helpers bauen");
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
            bytes.extend(std::iter::repeat(0u8).take(PADDING_BYTES));
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
            r#"{{"id":"{id}","name":"{name}","description":"{description}","releases":[{}]}}"#,
            releases.join(",")
        ));
    }
    let json = format!(r#"{{"schema":1,"minLauncherVersion":"0.1.0","apps":[{}]}}"#, apps.join(","));
    std::fs::write(dir.join("catalog.json"), &json).unwrap();
    let (pk, sk) = keys(dir);
    let sig = minisign::sign(Some(&pk), &sk, std::io::Cursor::new(json.as_bytes()), Some("trusted"), Some("untrusted")).unwrap();
    std::fs::write(dir.join("catalog.json.minisig"), sig.to_string()).unwrap();
    println!("Demo-Katalog (Stufe {stage}) liegt in {}", dir.display());
    println!("Schlüssel: {}", dir.join("pub.key").display());
}

/// `GET /pfad HTTP/1.1` → relativer Pfad; alles, was den Ordner verlassen könnte, wird abgelehnt.
fn request_path(line: &str) -> Option<String> {
    let mut parts = line.split_whitespace();
    if parts.next()? != "GET" {
        return None;
    }
    let rel = parts.next()?.trim_start_matches('/').to_string();
    let bad = rel.is_empty() || rel.contains(':') || rel.contains('\\') || rel.split('/').any(|s| s == "..");
    (!bad).then_some(rel)
}

fn handle(mut stream: std::net::TcpStream, dir: PathBuf, chunk_pause: Duration) {
    let mut buf = [0u8; 2048];
    let n = stream.read(&mut buf).unwrap_or(0);
    let head = String::from_utf8_lossy(&buf[..n]).to_string();
    let file = request_path(head.lines().next().unwrap_or("")).and_then(|rel| std::fs::read(dir.join(rel)).ok());
    let Some(bytes) = file else {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        return;
    };
    let header = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", bytes.len());
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
```

- [ ] **Step 4: Test, Bau, Prüfen**

Run: `cargo test -p launcher-cli --features test-helpers demo` — Expected: `pfad_bleibt_im_ordner` grün.
Run: `cargo build --workspace --features launcher-cli/test-helpers` — Expected: baut. Dann zur Probe in den Temp-Ordner (nicht ins Repository):

```powershell
$d = "$env:TEMP\daedyx-demo"; .\target\debug\demo-catalog.exe generate $d --stage 1
```
Expected: Ausgabe „Demo-Katalog (Stufe 1) liegt in …“; `catalog.json`, `catalog.json.minisig`, `pub.key`, 4 Pakete von je ca. 6 MB. Eine App über die Kommandozeile installieren: `.\target\debug\daedyx.exe --catalog $d --pubkey $d\pub.key --data-dir $d\data install notizbuch` → „notizbuch 1.0.0 installiert.“

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "test: Demo-Katalog und langsamer Testserver für die Sichtprüfung"
```

---

### Task 10: Sichtprüfung im echten Fenster

**Files:**
- Create: `scripts/window-shot.ps1`, `scripts/ui-invoke.ps1`

Hier wird nichts Neues gebaut, sondern das echte Fenster bedient und angeschaut. Befunde werden behoben (mit eigenem Test, wo es um Logik geht) und die Schritte danach wiederholt.

- [ ] **Step 1: Hilfsskripte schreiben**

`scripts/window-shot.ps1` — fotografiert das Hauptfenster eines Prozesses:

```powershell
param([string]$Process = "daedyx-launcher", [string]$Out = "$env:TEMP\daedyx-shot.png")
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices;
public class W { [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r); public struct RECT { public int L,T,R,B; } }
"@
$p = Get-Process $Process | Select-Object -First 1
$r = New-Object W+RECT; [W]::GetWindowRect($p.MainWindowHandle, [ref]$r) | Out-Null
$bmp = New-Object System.Drawing.Bitmap ($r.R-$r.L), ($r.B-$r.T)
[System.Drawing.Graphics]::FromImage($bmp).CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
$bmp.Save($Out); Write-Output $Out
```

`scripts/ui-invoke.ps1` — drückt einen Knopf über Windows-UI-Automation (das ist zugleich der Beweis, dass die Oberfläche für Hilfsprogramme lesbar ist):

```powershell
param([string]$Process = "daedyx-launcher", [string]$Name, [int]$Index = 0)
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes
$p = Get-Process $Process | Select-Object -First 1
$root = [System.Windows.Automation.AutomationElement]::FromHandle($p.MainWindowHandle)
$cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty, $Name)
$found = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
if ($found.Count -le $Index) { Write-Error "Element '$Name' nicht gefunden"; exit 1 }
$found[$Index].GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
```

- [ ] **Step 2: Demo vorbereiten (Stufe 1, zwei Apps vorinstallieren)**

```powershell
$env:Path += ";$env:USERPROFILE\.cargo\bin"
cargo build --workspace --features launcher-cli/test-helpers
$d = "$env:TEMP\daedyx-demo"; Remove-Item $d -Recurse -Force -ErrorAction SilentlyContinue
.\target\debug\demo-catalog.exe generate $d --stage 1
foreach ($a in "notizbuch","stundenplan") { .\target\debug\daedyx.exe --catalog $d --pubkey $d\pub.key --data-dir $d\data install $a }
.\target\debug\demo-catalog.exe generate $d --stage 2     # jetzt gibt es Updates für beide
Start-Process .\target\debug\demo-catalog.exe -ArgumentList "serve",$d -WindowStyle Hidden
Start-Process .\target\debug\daedyx-launcher.exe -ArgumentList "--catalog","http://127.0.0.1:8765","--pubkey","$d\pub.key","--data-dir","$d\data"
```

- [ ] **Step 3: Prüfliste** (je Punkt Foto mit `scripts\window-shot.ps1`, Bedienung mit `scripts\ui-invoke.ps1`):
  1. Vier Karten sichtbar: `Notizbuch` und `Stundenplan` mit „Update verfügbar · 1.0.0 → 1.1.0“ und Knöpfen Aktualisieren / Öffnen / Entfernen; `Zeichenbrett` und `Kassenbuch` „Nicht installiert“ mit Installieren. Umlaute und „→“ korrekt.
  2. **Installieren** von `Zeichenbrett` drücken: Fortschrittsbalken wächst sichtbar (Download ca. 10 s bei 640 KB/s), danach „Wird installiert …“, dann Karte „Installiert · Version 1.0.0“ und grüner Hinweis „Zeichenbrett 1.0.0 wurde installiert.“
  3. **Aktualisieren** von `Notizbuch`: Fortschritt, danach „Installiert · Version 1.1.0“ und Hinweis „… auf Version 1.1.0 aktualisiert.“
  4. **Öffnen** bei `Zeichenbrett`: Hinweis „Zeichenbrett wird gestartet.“ (Der Test-„App“ beendet sich sofort.)
  5. **Entfernen** bei `Stundenplan`: Rückfrage erscheint; „Abbrechen“ lässt alles; „Entfernen“ → Karte „Nicht installiert“, Hinweis „… wurde entfernt.“
  6. **Fehlerfall:** `catalog.json` im Demo-Ordner um ein Leerzeichen verändern, im Fenster „Aktualisieren“ drücken: roter Hinweis „… nicht auf Echtheit geprüft … nichts verändert“, die Karten bleiben. Datei zurücksetzen (`generate --stage 2` erneut).
  7. **Fehlerfall Paket:** in `packages\kassenbuch-1.0.0.exe` ein Byte ändern, „Installieren“ drücken: Hinweis „… beschädigt … nichts installiert“; im Ordner `data\downloads` bleibt keine Datei, `data\apps\kassenbuch` existiert nicht.
  8. **Dunkel:** Fenster schließen, mit `DAEDYX_THEME=dark` neu starten, Foto. Lesbarkeit von Text, Hinweisen, Marken, Fortschrittsbalken prüfen. Hell-Zustand folgt der Windows-Einstellung (Windows steht auf hell; das Umschalten der Windows-Einstellung selbst nimmt Björn vor, daran ändert die Prüfung nichts).
  9. **Größe:** Fenster schmal ziehen (Mindestbreite 420): Karten brechen um, nichts wird abgeschnitten.
  10. **Tastatur:** Tab springt durch die Knöpfe, Enter löst aus, ein Fokusrahmen ist sichtbar.
  11. **Fehlende Quelle:** `daedyx-launcher.exe` ohne Argumente und ohne Umgebungsvariablen starten: Fenster zeigt die Anleitung (`--catalog`, `--pubkey`), kein Absturz.

- [ ] **Step 4: Befunde beheben** (Logik-Fehler mit Test zuerst in `model_tests.rs`/`card_view_tests.rs`, Aussehen direkt), Prüfliste wiederholen, bis alles passt. Prozesse beenden (`Stop-Process -Name demo-catalog,daedyx-launcher`) und den Ordner `$env:TEMP\daedyx-demo` löschen.

- [ ] **Step 5: Commit**

```bash
git add scripts
git commit -m "chore: Skripte für Fensterfoto und Bedienung per UI-Automation"
```

---

### Task 11: Abschluss — Qualität, Doku, Push

**Files:** Modify: `docs/STAND.md`; Modify: `D:\Programmierung\Werkbank\docs\superpowers\plans\2026-11-13-launcher-konzept.md` (Abschnitt „Stand der Umsetzung“, nur L2 nachziehen).

- [ ] **Step 1: Alles prüfen**

Run (PowerShell, nacheinander):
`cargo fmt --all`
`cargo clippy --workspace --all-targets --features launcher-cli/test-helpers` — Expected: ohne Warnung.
`cargo test --workspace --features launcher-cli/test-helpers` — Expected: alles grün; Anzahl der Tests notieren.
`git status` — Expected: nur beabsichtigte Dateien, keine Schlüssel, keine `.exe`, keine Demo-Daten.
Dateigrößen prüfen (`app.rs`, `card_view.rs`, `demo_catalog.rs` nahe 200 Zeilen sind in Ordnung; deutlich darüber: aufteilen).

- [ ] **Step 2: `docs/STAND.md` aktualisieren** — Abschnitt „Wo wir stehen“ um L2 ergänzen (Testzahl, was die Oberfläche kann, was geprüft wurde und womit, Binärgröße und Startzeit aus der Sichtprüfung), „Bewusst noch nicht“ auf die Liste „Nicht in L2“ dieses Plans umstellen, „Nächster Schritt“: L3, und der ehrliche Hinweis, dass Narrator/Bildschirmleser **nicht** geprüft wurden (UI-Automation und kittest zeigen: Knöpfe und Texte sind für Hilfsprogramme lesbar). Datum: Tag der Fertigstellung.

- [ ] **Step 3: Konzept in Werkbank nachziehen** — nur im Abschnitt „Stand der Umsetzung“ zwei Sätze zu L2 (gebaut, Abweichungen: Buchstabenmarke statt Symbol, Laufprüfung auf L3 verschoben). Nicht die Werkbank-Datei sonst anfassen.

- [ ] **Step 4: Committen und pushen**

```bash
git add -A
git commit -m "feat: Meilenstein L2 (Oberfläche in egui)"
git push origin main
```
Expected: Push in das private Repository `daedyx/launcher` ohne Fehler.

---

## Selbstprüfung gegen die Aufgabe
- Karten-Raster mit Symbol (Buchstabenmarke, Abweichung begründet), Name, Kurztext, Status der drei Zustände: Task 6, 7, 8.
- Installieren / Aktualisieren / Entfernen / Öffnen: Task 2 (Öffnen im Kern), 3 (Aufträge), 7 (Knöpfe), 8 (Rückfrage beim Entfernen).
- Fortschrittsanzeige beim Download aus Hintergrund-Thread: Task 3 (Arbeiter), 6 (Zustand), 7 (Balken).
- Verständliche Fehlertexte: Task 5 (vollständige Übersetzung jeder Fehlervariante, Test erzwingt es), Task 10 Prüfpunkte 6/7.
- Deutsch aus Sprachdateien: Task 5; Test, dass jeder im Code benutzte Schlüssel existiert.
- Hell/Dunkel nach Windows: Task 8 (`ThemePreference::System`), Task 10 (Dunkel-Foto per Umgebungsvariable).
- Test zuerst: jeder Task beginnt mit scheiternden Tests; Fenster-Rest über Sichtprüfung.
- Sicherheit: Oberfläche ruft nur Kernfunktionen; `open` prüft Kennung, Programmordner und relativen Pfad; Demo-Server liefert nur Dateien unterhalb des Ordners.
