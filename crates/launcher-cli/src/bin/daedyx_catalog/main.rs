//! `daedyx-catalog`: Werkzeug für den Katalog (Schlüssel erzeugen, Release einfügen, signieren, prüfen).
//! Läuft auf Björns Rechner; der geheime Schlüssel verlässt ihn nie, das Passwort tippt Björn selbst.
mod keys;
#[cfg(test)]
mod keys_tests;

use clap::{Parser, Subcommand};
use daedyx_update::catalog::{Catalog, Release, SCHEMA_VERSION};
use daedyx_update::catalog_edit::{add_release, AppMeta};
use daedyx_update::verify::verified_catalog;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "daedyx-catalog",
    version,
    about = "Katalog des Daedyx Launchers bearbeiten und signieren"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Schlüsselpaar erzeugen (Passwort wird zweimal abgefragt)
    Keygen {
        /// Ordner für `catalog.key` (geheim) und `catalog.pub` (öffentlich)
        #[arg(long)]
        out: PathBuf,
        /// Ohne Passwort (nur für Testschlüssel!)
        #[arg(long)]
        no_password: bool,
    },
    /// Eine Release (`catalog-entry.json` von release.ps1) in den Eintrag einer App einfügen
    AddRelease {
        #[arg(long)]
        catalog: PathBuf,
        #[arg(long)]
        app: String,
        #[arg(long)]
        entry: PathBuf,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        license: Option<String>,
        #[arg(long)]
        author: Option<String>,
        /// Name des Uninstall-Schlüssels in der Registry (bei Desk: "Daedyx Desk")
        #[arg(long)]
        registry_key: Option<String>,
        /// Adresse des Pakets im Katalog ersetzen (für einen lokalen Katalogordner: relativ,
        /// zum Beispiel `packages/Daedyx.Desk_0.2.0_x64-setup.exe`)
        #[arg(long)]
        package_url: Option<String>,
        /// Schon vorhandene Version ersetzen
        #[arg(long)]
        replace: bool,
    },
    /// `catalog.json` signieren und `catalog.json.minisig` daneben schreiben
    Sign {
        #[arg(long)]
        catalog: PathBuf,
        /// Datei mit dem geheimen Schlüssel
        #[arg(long, conflicts_with = "key_env", required_unless_present = "key_env")]
        key: Option<PathBuf>,
        /// Oder: Name einer Umgebungsvariable, die den Inhalt der Schlüsseldatei enthält (Pipeline).
        /// Das Passwort steht dann in `DAEDYX_KEY_PASSWORD`; es wird nichts abgefragt.
        #[arg(long)]
        key_env: Option<String>,
        /// Öffentlicher Schlüssel zur Gegenprobe (sonst `catalog.pub` neben dem geheimen Schlüssel)
        #[arg(long)]
        public: Option<PathBuf>,
    },
    /// Signatur und Inhalt eines Katalogs prüfen
    Verify {
        #[arg(long)]
        catalog: PathBuf,
        #[arg(long)]
        public: PathBuf,
    },
}

/// Liest eine Textdatei; ein UTF-8-BOM (Windows-Editoren, PowerShell) wird übersprungen.
fn read(path: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(text.strip_prefix('\u{feff}').unwrap_or(&text).to_string())
}

fn write_new(path: &Path, text: &str) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "{} gibt es schon, nichts überschrieben",
            path.display()
        ));
    }
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))
}

fn sig_path(catalog: &Path) -> PathBuf {
    let mut name = catalog.file_name().unwrap_or_default().to_os_string();
    name.push(".minisig");
    catalog.with_file_name(name)
}

fn ask_password(prompt: &str) -> Result<String, String> {
    rpassword::prompt_password(prompt).map_err(|e| e.to_string())
}

fn keygen(out: &Path, no_password: bool) -> Result<String, String> {
    let password = if no_password {
        None
    } else {
        let first = ask_password("Passwort für den Katalog-Schlüssel (du tippst es selbst): ")?;
        if first.is_empty() {
            return Err("Ein leeres Passwort ist nicht erlaubt (für Tests: --no-password)".into());
        }
        if first != ask_password("Passwort wiederholen: ")? {
            return Err("Die Passwörter sind nicht gleich".into());
        }
        Some(first)
    };
    std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let (key_path, pub_path) = (out.join("catalog.key"), out.join("catalog.pub"));
    if key_path.exists() || pub_path.exists() {
        return Err("Im Ordner liegt schon ein Schlüssel, nichts überschrieben".into());
    }
    let (public, secret) = keys::generate(password)?;
    write_new(&key_path, &secret)?;
    write_new(&pub_path, &public)?;
    Ok(format!(
        "Geheimer Schlüssel: {}\nÖffentlicher Schlüssel: {}\n\nDen geheimen Schlüssel an zwei Orten sichern, nie in Git oder in einen Chat.\nGeht er verloren, lässt sich kein Katalog mehr signieren.\nDen öffentlichen Schlüssel trägst du so beim Launcher ein:\n  daedyx config set --pubkey \"{}\"",
        key_path.display(),
        pub_path.display(),
        pub_path.display()
    ))
}

#[allow(clippy::too_many_arguments)]
fn add(
    catalog: &Path,
    app: &str,
    entry: &Path,
    meta: AppMeta,
    package_url: Option<String>,
    replace: bool,
) -> Result<String, String> {
    let mut cat = if catalog.exists() {
        Catalog::parse(read(catalog)?.as_bytes()).map_err(|e| e.to_string())?
    } else {
        Catalog {
            schema: SCHEMA_VERSION,
            min_launcher_version: "0.1.0".into(),
            apps: vec![],
        }
    };
    let mut release: Release = serde_json::from_str(&read(entry)?)
        .map_err(|e| format!("{} ist keine Release: {e}", entry.display()))?;
    if let Some(url) = package_url {
        for package in release.packages.values_mut() {
            package.url = url.clone();
        }
    }
    let version = release.version.clone();
    add_release(&mut cat, app, release, meta, replace).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(&cat).map_err(|e| e.to_string())?;
    Catalog::parse(json.as_bytes()).map_err(|e| e.to_string())?;
    std::fs::write(catalog, json).map_err(|e| e.to_string())?;
    // Die alte Signatur passt nicht mehr zum geänderten Katalog.
    let _ = std::fs::remove_file(sig_path(catalog));
    Ok(format!(
        "{app} {version} ist im Katalog. Jetzt signieren:\n  daedyx-catalog sign --catalog \"{}\" --key <Pfad zu catalog.key>",
        catalog.display()
    ))
}

/// Passwort für einen verschlüsselten Schlüssel: aus `DAEDYX_KEY_PASSWORD` (Pipeline), sonst
/// verdeckte Eingabe im Terminal.
fn key_password() -> Result<String, String> {
    match std::env::var("DAEDYX_KEY_PASSWORD") {
        Ok(p) if !p.is_empty() => Ok(p),
        _ => ask_password("Passwort des Katalog-Schlüssels: "),
    }
}

fn sign(
    catalog: &Path,
    key: Option<&Path>,
    key_env: Option<&str>,
    public: Option<&Path>,
) -> Result<String, String> {
    let data = std::fs::read(catalog).map_err(|e| format!("{}: {e}", catalog.display()))?;
    Catalog::parse(&data).map_err(|e| format!("Der Katalog ist ungültig: {e}"))?;
    let key_text = match (key, key_env) {
        (Some(path), _) => read(path)?,
        (None, Some(var)) => std::env::var(var)
            .map_err(|_| format!("Die Umgebungsvariable {var} ist nicht gesetzt"))?,
        (None, None) => return Err("Schlüssel fehlt (--key oder --key-env)".into()),
    };
    let secret = keys::load_secret(&key_text, key_password)?;
    let pub_file = public.map(Path::to_path_buf).or_else(|| {
        key.and_then(Path::parent)
            .map(|d| d.join("catalog.pub"))
            .filter(|p| p.exists())
    });
    let pub_text = pub_file.as_deref().map(read).transpose()?;
    let pk = pub_text.as_deref().map(keys::load_public).transpose()?;
    let sig = keys::sign(&secret, pk.as_ref(), &data)?;
    if let Some(text) = &pub_text {
        // Gegenprobe: die frische Signatur muss zum öffentlichen Schlüssel passen.
        verified_catalog(text, &data, &sig).map_err(|_| {
            "Die Gegenprobe schlug fehl: Schlüssel gehören nicht zusammen".to_string()
        })?;
    }
    let target = sig_path(catalog);
    std::fs::write(&target, sig).map_err(|e| e.to_string())?;
    Ok(format!(
        "Signiert: {}{}",
        target.display(),
        if pub_text.is_some() {
            " (Gegenprobe bestanden)"
        } else {
            " (ohne Gegenprobe)"
        }
    ))
}

fn verify(catalog: &Path, public: &Path) -> Result<String, String> {
    let data = std::fs::read(catalog).map_err(|e| format!("{}: {e}", catalog.display()))?;
    let sig = read(&sig_path(catalog))?;
    let cat = verified_catalog(&read(public)?, &data, &sig).map_err(|e| e.to_string())?;
    let releases: usize = cat.apps.iter().map(|a| a.releases.len()).sum();
    Ok(format!(
        "Signatur gültig: {} App(s), {releases} Release(s).",
        cat.apps.len()
    ))
}

fn run(cli: Cli) -> Result<String, String> {
    match cli.command {
        Command::Keygen { out, no_password } => keygen(&out, no_password),
        Command::AddRelease {
            catalog,
            app,
            entry,
            name,
            description,
            license,
            author,
            registry_key,
            package_url,
            replace,
        } => add(
            &catalog,
            &app,
            &entry,
            AppMeta {
                name,
                description,
                license,
                author,
                registry_key,
            },
            package_url,
            replace,
        ),
        Command::Sign {
            catalog,
            key,
            key_env,
            public,
        } => sign(
            &catalog,
            key.as_deref(),
            key_env.as_deref(),
            public.as_deref(),
        ),
        Command::Verify { catalog, public } => verify(&catalog, &public),
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(text) => {
            println!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Fehler: {e}");
            ExitCode::FAILURE
        }
    }
}
