//! Hintergrund-Arbeiter: führt Aufträge in einem eigenen Thread aus und meldet Ereignisse.
//! Der Kern kennt keine Oberfläche; ein Wecker-Rückruf sagt ihr nur, dass es etwas zu holen gibt.
use crate::error::CoreError;
use crate::lineup::Lineup;
use crate::selfupdate::SelfUpdate;
use crate::service::{AppStatus, Launcher, Outcome};
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
    /// Ergebnis der Suche für alle Apps (kommt immer, auch ohne Netz; Gründe stehen je App darin).
    Lineup(Lineup),
    Status(Vec<AppStatus>),
    /// Kennungen der Apps, die gerade laufen.
    Running(Vec<String>),
    /// Geprüfte neuere Version des Launchers selbst; kommt nach `Status`, nur wenn es eine gibt.
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
    pub fn start(launcher: Launcher, wake: Wake) -> Worker {
        let (jobs_tx, jobs_rx) = channel();
        let (events_tx, events_rx) = channel();
        let out = Emitter {
            tx: events_tx,
            wake,
        };
        std::thread::spawn(move || run(launcher, jobs_rx, out));
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
    total.filter(|t| *t > 0).and_then(|t| {
        let share = u128::from(done.min(t))
            .checked_mul(100)?
            .checked_div(u128::from(t))?;
        u8::try_from(share).ok()
    })
}

fn refresh(launcher: &Launcher, lineup: &mut Option<Lineup>, out: &Emitter) {
    let l = launcher.load_lineup();
    let update = launcher.self_update_available(&l);
    *lineup = Some(l.clone());
    out.emit(Event::Lineup(l));
    emit_status(launcher, lineup, out);
    // Nur wenn es ein Update gibt; die Reihenfolge Suche, Status bleibt wie zugesagt.
    if update.is_some() {
        out.emit(Event::SelfUpdate(update));
    }
}

fn emit_status(launcher: &Launcher, lineup: &Option<Lineup>, out: &Emitter) {
    if let Some(Ok(status)) = lineup.as_ref().map(|l| launcher.status(l)) {
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
    lineup: &Lineup,
    app: &str,
    out: &Emitter,
) -> Result<Outcome, CoreError> {
    launcher.install(lineup, app, &mut progress_to(out, app))
}

fn run(launcher: Launcher, jobs: Receiver<Job>, out: Emitter) {
    launcher.clean_self_update_leftovers();
    let mut lineup: Option<Lineup> = None;
    while let Ok(job) = jobs.recv() {
        match job {
            Job::Refresh => refresh(&launcher, &mut lineup, &out),
            Job::Install(app) => {
                if lineup.is_none() {
                    refresh(&launcher, &mut lineup, &out);
                }
                let result = match &lineup {
                    Some(l) => install(&launcher, l, &app, &out),
                    None => Err(CoreError::NotOffered(app.clone())),
                };
                out.emit(Event::Installed { app, result });
                emit_status(&launcher, &lineup, &out);
            }
            Job::Remove(app) => {
                let result = launcher.remove(&app);
                out.emit(Event::Removed { app, result });
                emit_status(&launcher, &lineup, &out);
            }
            Job::UpdateSelf => {
                let result = match &lineup {
                    Some(l) => launcher.start_self_update(
                        l,
                        true,
                        &mut progress_to(&out, crate::selfupdate::LAUNCHER_APP_ID),
                    ),
                    None => Ok(None),
                };
                out.emit(Event::SelfUpdateStarted(result));
            }
            Job::CheckRunning => {
                if let Ok(ids) = launcher.running_apps() {
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
