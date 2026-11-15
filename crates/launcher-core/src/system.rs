//! Fragen an das Betriebssystem: Uninstall-Eintrag einer App in der Registry und laufende Programme.
//! Hinter einem Trait, damit die Tests des Kerns ohne echte Registry und Prozesse auskommen.
use std::path::{Path, PathBuf};

/// Was das Setup einer App unter `HKCU\...\Uninstall\<Name>` einträgt.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct UninstallEntry {
    pub display_version: Option<String>,
    pub install_location: Option<PathBuf>,
}

pub trait SystemInfo: Send + Sync {
    /// Eintrag unter dem Uninstall-Schlüssel `key`, falls vorhanden (nur Windows).
    fn uninstall_entry(&self, key: &str) -> Option<UninstallEntry>;
    /// Zu jedem Ordner: Läuft gerade ein Programm, dessen Datei in diesem Ordner liegt?
    fn running(&self, dirs: &[PathBuf]) -> Vec<bool>;
    /// Startet ein Programm unabhängig vom Launcher (er darf danach beendet werden).
    fn start_detached(&self, program: &Path, args: &[&str]) -> Result<(), String>;
}

pub const UNINSTALL_BASE: &str = r"Software\Microsoft\Windows\CurrentVersion\Uninstall";

/// Das echte System. `uninstall_base` ist nur für Tests änderbar.
pub struct RealSystem {
    pub uninstall_base: String,
}

impl Default for RealSystem {
    fn default() -> Self {
        RealSystem {
            uninstall_base: UNINSTALL_BASE.to_string(),
        }
    }
}

/// Schlüsselnamen kommen aus dem signierten Katalog, dürfen aber keinen Pfad in der Registry bilden.
fn safe_key(key: &str) -> bool {
    !key.is_empty() && !key.contains(['\\', '/'])
}

/// Vergleichsform eines Pfads: kleingeschrieben, `/`, ohne `\\?\`-Vorsatz, ohne Schluss-Trenner.
fn norm(path: &Path) -> String {
    let s = path.to_string_lossy().replace('/', "\\").to_lowercase();
    s.strip_prefix(r"\\?\")
        .unwrap_or(&s)
        .trim_end_matches('\\')
        .to_string()
}

fn is_inside(exe: &Path, dir: &Path) -> bool {
    let (exe, dir) = (norm(exe), norm(dir));
    !dir.is_empty()
        && exe.len() > dir.len()
        && exe.starts_with(&dir)
        && exe[dir.len()..].starts_with('\\')
}

impl SystemInfo for RealSystem {
    #[cfg(windows)]
    fn uninstall_entry(&self, key: &str) -> Option<UninstallEntry> {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        if !safe_key(key) {
            return None;
        }
        let k = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(format!(r"{}\{}", self.uninstall_base, key))
            .ok()?;
        let version: Option<String> = k.get_value("DisplayVersion").ok();
        let location: Option<String> = k.get_value("InstallLocation").ok();
        Some(UninstallEntry {
            display_version: version
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
            // Tauri-Setups tragen den Ordner in Anführungszeichen ein.
            install_location: location
                .map(|l| l.trim().trim_matches('"').to_string())
                .filter(|l| !l.is_empty())
                .map(PathBuf::from),
        })
    }

    #[cfg(not(windows))]
    fn uninstall_entry(&self, _key: &str) -> Option<UninstallEntry> {
        None
    }

    fn start_detached(&self, program: &Path, args: &[&str]) -> Result<(), String> {
        let mut command = std::process::Command::new(program);
        command.args(args);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
            command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        }
        command.spawn().map(|_| ()).map_err(|e| e.to_string())
    }

    fn running(&self, dirs: &[PathBuf]) -> Vec<bool> {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
        // Ist die Prozessliste nicht lesbar, wissen wir es nicht: „nicht laufend“ (das Setup prüft selbst noch).
        let Ok(mut sys) = System::new() else {
            return vec![false; dirs.len()];
        };
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
        );
        let exes: Vec<PathBuf> = sys
            .processes()
            .values()
            .filter_map(|p| p.exe().map(|e| e.to_path_buf()))
            .collect();
        dirs.iter()
            .map(|d| exes.iter().any(|e| is_inside(e, d)))
            .collect()
    }
}
