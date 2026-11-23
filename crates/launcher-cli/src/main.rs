use clap::{Parser, Subcommand};
use launcher_core::config::{build_launcher, Config};
use launcher_core::error::CoreError;
use launcher_core::paths::default_data_dir;
use launcher_core::service::{AppStatus, Outcome};
use serde::Serialize;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "daedyx", version, about = "Daedyx Launcher (Kommandozeile)")]
struct Cli {
    /// Katalogquelle: Ordner oder Internetadresse (sonst aus der gespeicherten Einstellung)
    #[arg(long, env = "DAEDYX_CATALOG")]
    catalog: Option<String>,
    /// Öffentlicher minisign-Schlüssel des Katalogs (Datei; sonst aus der gespeicherten Einstellung)
    #[arg(long, env = "DAEDYX_PUBKEY")]
    pubkey: Option<PathBuf>,
    /// Datenordner des Launchers
    #[arg(long, env = "DAEDYX_DATA_DIR")]
    data_dir: Option<PathBuf>,
    /// Ausgabe als JSON
    #[arg(long)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Apps des Katalogs mit Stand anzeigen
    List,
    /// App installieren (oder auf die neueste Version bringen)
    Install { app: String },
    /// Installierte Apps aktualisieren (ohne Angabe: alle mit Update)
    Update { app: Option<String> },
    /// App entfernen
    Remove { app: String },
    /// App starten
    Launch { app: String },
    /// Launcher selbst aktualisieren (lädt das geprüfte Setup und startet es still)
    SelfUpdate {
        /// Nur prüfen, ob es ein Update gibt
        #[arg(long)]
        check: bool,
        /// Das Setup startet den Launcher danach nicht neu (für Skripte und Tests)
        #[arg(long)]
        no_launch: bool,
    },
    /// Gespeicherte Einstellungen (Katalogquelle, Schlüsseldatei) anzeigen oder ändern
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Gespeicherte Einstellungen anzeigen
    Show,
    /// Einstellungen prüfen und dauerhaft speichern
    Set {
        #[arg(long)]
        catalog: Option<String>,
        #[arg(long)]
        pubkey: Option<PathBuf>,
    },
}

#[derive(Serialize)]
struct Done {
    app: String,
    #[serde(flatten)]
    outcome: Outcome,
}

fn list_text(status: &[AppStatus]) -> String {
    status
        .iter()
        .map(|s| {
            format!(
                "{:<12} {:<20} installiert: {:<10} neueste: {:<10}{}",
                s.id,
                s.name,
                s.installed.as_deref().unwrap_or("-"),
                s.latest.as_deref().unwrap_or("-"),
                match (s.update_available, s.running) {
                    (true, true) => " (Update verfügbar, läuft)",
                    (true, false) => " (Update verfügbar)",
                    (false, true) => " (läuft)",
                    (false, false) => "",
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn done_text(done: &[Done]) -> String {
    if done.is_empty() {
        return "Alles aktuell.".to_string();
    }
    done.iter()
        .map(|d| match &d.outcome {
            Outcome::Installed { version } => format!("{} {} installiert.", d.app, version),
            Outcome::Updated { from, to } => {
                format!("{} von {} auf {} aktualisiert.", d.app, from, to)
            }
            Outcome::UpToDate { version } => format!("{} {} ist schon aktuell.", d.app, version),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn to_json<T: Serialize + ?Sized>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".into())
}

fn config_command(
    cli: &Cli,
    data_dir: PathBuf,
    action: &ConfigAction,
) -> Result<String, CoreError> {
    let saved = Config::load(&data_dir);
    match action {
        ConfigAction::Show => Ok(if cli.json {
            to_json(&saved)
        } else {
            format!(
                "Katalogquelle: {}\nSchlüsseldatei: {}\nDatenordner: {}",
                saved.catalog.as_deref().unwrap_or("-"),
                saved
                    .pubkey
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "-".into()),
                data_dir.display()
            )
        }),
        ConfigAction::Set { catalog, pubkey } => {
            let new = saved.with_overrides(catalog.clone(), pubkey.clone());
            // Erst prüfen (Quelle erlaubt, Ordner da, Schlüssel gültig), dann speichern.
            build_launcher(&new, data_dir.clone(), env!("CARGO_PKG_VERSION"))?;
            new.save(&data_dir)?;
            Ok(if cli.json {
                to_json(&new)
            } else {
                "Einstellungen gespeichert.".to_string()
            })
        }
    }
}

fn run(cli: &Cli) -> Result<String, CoreError> {
    let data_dir = cli.data_dir.clone().unwrap_or_else(default_data_dir);
    if let Command::Config { action } = &cli.command {
        return config_command(cli, data_dir, action);
    }
    let config = Config::load(&data_dir).with_overrides(cli.catalog.clone(), cli.pubkey.clone());
    let (launcher, pubkey) = build_launcher(&config, data_dir, env!("CARGO_PKG_VERSION"))?;
    let catalog = launcher.load_catalog(&pubkey)?;
    let mut progress = |_: u64, _: Option<u64>| {};

    match &cli.command {
        Command::List => {
            let status = launcher.status(&catalog)?;
            Ok(if cli.json {
                to_json(&status)
            } else {
                list_text(&status)
            })
        }
        Command::Install { app } => {
            let outcome = launcher.install(&catalog, app, &mut progress)?;
            let done = [Done {
                app: app.clone(),
                outcome,
            }];
            Ok(if cli.json {
                to_json(&done[..])
            } else {
                done_text(&done)
            })
        }
        Command::Update { app } => {
            let status = launcher.status(&catalog)?;
            let ids: Vec<String> = match app {
                Some(id) => {
                    let installed = status.iter().any(|s| &s.id == id && s.installed.is_some());
                    if !installed {
                        return Err(CoreError::NotInstalled(id.clone()));
                    }
                    vec![id.clone()]
                }
                None => status
                    .into_iter()
                    .filter(|s| s.update_available)
                    .map(|s| s.id)
                    .collect(),
            };
            let mut done = Vec::new();
            for id in ids {
                let outcome = launcher.install(&catalog, &id, &mut progress)?;
                done.push(Done { app: id, outcome });
            }
            Ok(if cli.json {
                to_json(&done)
            } else {
                done_text(&done)
            })
        }
        // Wird schon vor dem Laden des Katalogs behandelt; hier nur der Vollständigkeit halber.
        Command::Config { action } => config_command(
            cli,
            cli.data_dir.clone().unwrap_or_else(default_data_dir),
            action,
        ),
        Command::SelfUpdate { check, no_launch } => {
            let available = launcher.self_update_available(&catalog)?;
            let update = if *check {
                available
            } else {
                launcher.start_self_update(&catalog, !*no_launch, &mut progress)?
            };
            Ok(match (update, cli.json) {
                (Some(u), true) => {
                    serde_json::json!({ "update": u.version, "started": !*check }).to_string()
                }
                (Some(u), false) if *check => format!("Update auf {} verfügbar.", u.version),
                (Some(u), false) => format!("Update auf {} wurde gestartet.", u.version),
                (None, true) => serde_json::json!({ "update": null }).to_string(),
                (None, false) => "Der Launcher ist aktuell.".to_string(),
            })
        }
        Command::Launch { app } => {
            launcher.open(app)?;
            Ok(if cli.json {
                serde_json::json!({ "app": app, "result": "launched" }).to_string()
            } else {
                format!("{app} wird gestartet.")
            })
        }
        Command::Remove { app } => {
            launcher.remove(app)?;
            Ok(if cli.json {
                serde_json::json!({ "app": app, "result": "removed" }).to_string()
            } else {
                format!("{app} wurde entfernt.")
            })
        }
    }
}

#[allow(
    clippy::print_stdout,
    clippy::print_stderr,
    reason = "SICHERHEIT-AUSNAHMEN A-010"
)]
fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(out) => {
            println!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            if cli.json {
                println!("{}", serde_json::json!({ "error": e.to_string() }));
            } else {
                eprintln!("Fehler: {e}");
            }
            ExitCode::FAILURE
        }
    }
}
