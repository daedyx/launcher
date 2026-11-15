//! Hintergrund-Arbeiter: führt Aufträge in einem eigenen Thread aus und meldet Ereignisse.
//! Der Kern kennt keine Oberfläche; ein Wecker-Rückruf sagt ihr nur, dass es etwas zu holen gibt.
use crate::error::CoreError;
use crate::selfupdate::SelfUpdate;
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
    /// Prüft, welche Apps gerade laufen (billig, darf oft gerufen werden).
    CheckRunning,
    /// Lädt das Setup des Launchers und startet es; danach soll der Launcher sich beenden.
    UpdateSelf,
}

#[derive(Debug)]
pub enum Event {
    Catalog(Result<Catalog, CoreError>),
    Status(Vec<AppStatus>),
    /// Kennungen der Apps, die gerade laufen.
    Running(Vec<String>),
    /// Neuere Version des Launchers selbst im Katalog; kommt nach `Status`, nur wenn es eine gibt.
    SelfUpdate(Option<SelfUpdate>),
    /// Das Setup wurde gestartet (`Ok(Some)`) oder ging schief; bei `Ok(Some)` beendet sich der
    /// Launcher, damit das Setup ihn ersetzen kann.
    SelfUpdateStarted(Result<Option<SelfUpdate>, CoreError>),
    Progress {
        app: String,
        done: u64,
        total: Option<u64>,
    },
    Installed {
        app: String,
        result: Result<Outcome, CoreError>,
    },
    Removed {
        app: String,
        result: Result<(), CoreError>,
    },
    Opened {
        app: String,
        result: Result<(), CoreError>,
    },
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
        let out = Emitter {
            tx: events_tx,
            wake,
        };
        std::thread::spawn(move || run(launcher, pubkey, jobs_rx, out));
        Worker {
            jobs: jobs_tx,
            events: events_rx,
        }
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
    total
        .filter(|t| *t > 0)
        .map(|t| (done.min(t) * 100 / t) as u8)
}

fn refresh(launcher: &Launcher, pubkey: &str, catalog: &mut Option<Catalog>, out: &Emitter) {
    match launcher.load_catalog(pubkey) {
        Ok(c) => {
            *catalog = Some(c.clone());
            let update = launcher.self_update_available(&c).ok().flatten();
            out.emit(Event::Catalog(Ok(c)));
            emit_status(launcher, catalog, out);
            // Nur wenn es ein Update gibt; die Reihenfolge Katalog, Status bleibt wie zugesagt.
            if update.is_some() {
                out.emit(Event::SelfUpdate(update));
            }
        }
        Err(e) => out.emit(Event::Catalog(Err(e))),
    }
}

fn emit_status(launcher: &Launcher, catalog: &Option<Catalog>, out: &Emitter) {
    if let Some(Ok(status)) = catalog.as_ref().map(|c| launcher.status(c)) {
        out.emit(Event::Status(status));
    }
}

/// Fortschritt eines Downloads als Ereignisse (nur bei geändertem Prozentwert).
fn progress_to<'a>(out: &'a Emitter, app: &'a str) -> impl FnMut(u64, Option<u64>) + 'a {
    let mut last: Option<u64> = None;
    move |done: u64, total: Option<u64>| {
        let key = match percent(done, total) {
            Some(p) => p as u64,
            None => done / (256 * 1024),
        };
        if last != Some(key) {
            last = Some(key);
            out.emit(Event::Progress {
                app: app.to_string(),
                done,
                total,
            });
        }
    }
}

fn install(
    launcher: &Launcher,
    catalog: &Catalog,
    app: &str,
    out: &Emitter,
) -> Result<Outcome, CoreError> {
    launcher.install(catalog, app, &mut progress_to(out, app))
}

fn run(launcher: Launcher, pubkey: String, jobs: Receiver<Job>, out: Emitter) {
    launcher.clean_self_update_leftovers();
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
            Job::UpdateSelf => {
                let result = match &catalog {
                    Some(c) => launcher.start_self_update(
                        c,
                        true,
                        &mut progress_to(&out, crate::selfupdate::LAUNCHER_APP_ID),
                    ),
                    None => Err(CoreError::Download("Der Katalog ist nicht geladen".into())),
                };
                out.emit(Event::SelfUpdateStarted(result));
            }
            Job::CheckRunning => {
                if let Some(Ok(ids)) = catalog.as_ref().map(|c| launcher.running_apps(c)) {
                    out.emit(Event::Running(ids));
                }
            }
            Job::Open(app) => {
                let result = launcher.open(&app);
                out.emit(Event::Opened { app, result });
            }
        }
    }
}
