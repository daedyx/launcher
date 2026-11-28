//! Die Apps, die dieser Launcher kennt (plan-ohne-secrets 2.2, Auftrag U3). Die Liste steht fest im Programm und
//! ersetzt den signierten Katalog: Eine neue App heißt ein neues Launcher-Release. Je App ist die Update-Quelle fest
//! eingetragen (Besitzer, Repo, **numerische** Kennungen gegen Umbenennen und Neuanlegen des Namens, Spike P4); den
//! Workflow `release.yml` auf `main` legt der Kern fest. Kein Schlüssel, kein Token.
use daedyx_update::policy::{Platform, UpdatePolicy};

/// Besitzer aller Releases (`github.com/daedyx`), numerische Kennung am 2026-11-26 mit `gh api` gelesen (nur lesen).
pub const OWNER: &str = "daedyx";
pub const OWNER_ID: u64 = 356_409_479;

/// Eine bekannte App mit ihrer festen Update-Quelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KnownApp {
    /// Kennung im Launcher (Programmordner, `installed.json`, Oberfläche).
    pub id: &'static str,
    /// Anzeigename (Marke, nicht übersetzt).
    pub name: &'static str,
    /// Repo der Releases unter [`OWNER`] und seine numerische Kennung.
    pub repo: &'static str,
    pub repo_id: u64,
    /// Kennung im Update-Eintrag (`entry.json`, Feld `app`) und Name der gemeinsamen Sperre mit der App.
    pub update_app: &'static str,
    /// Name des Uninstall-Schlüssels der App in der Registry (findet auch eine von Hand installierte App).
    pub registry_key: Option<&'static str>,
}

/// Kennung des Launchers selbst (Selbst-Update).
pub const LAUNCHER_ID: &str = "launcher";

/// Der Launcher selbst: sein Update kommt aus dem eigenen Repo.
pub const LAUNCHER: KnownApp = KnownApp {
    id: LAUNCHER_ID,
    name: "Daedyx Launcher",
    repo: "launcher",
    repo_id: 1_474_858_940,
    update_app: "daedyx-launcher",
    registry_key: None,
};

/// Die Apps, die der Launcher anbietet, in der Reihenfolge der Anzeige.
pub const APPS: [KnownApp; 2] = [
    KnownApp {
        id: "desk",
        name: "Daedyx Desk",
        repo: "desk",
        repo_id: 1_475_265_095,
        update_app: "daedyx-desk",
        registry_key: Some("Daedyx Desk"),
    },
    KnownApp {
        id: "tasks",
        name: "Daedyx Tasks",
        repo: "tasks",
        repo_id: 1_474_858_832,
        update_app: "daedyx-tasks",
        registry_key: Some("Daedyx Tasks"),
    },
];

/// Die App mit dieser Kennung (auch der Launcher selbst).
pub fn find(id: &str) -> Option<&'static KnownApp> {
    APPS.iter()
        .chain(std::iter::once(&LAUNCHER))
        .find(|a| a.id == id)
}

impl KnownApp {
    /// Regel der Update-Quelle für die Plattform dieses Programms (`None` auf nicht unterstützten Systemen).
    pub fn policy(&self) -> Option<UpdatePolicy> {
        self.policy_for(Platform::current()?)
    }

    /// Regel der Update-Quelle für eine bestimmte Plattform.
    pub fn policy_for(&self, platform: Platform) -> Option<UpdatePolicy> {
        UpdatePolicy::new(
            OWNER,
            self.repo,
            OWNER_ID,
            self.repo_id,
            self.update_app,
            platform,
        )
        .ok()
    }
}
