//! Start beim Anmelden: ein Wert unter `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
//! (pro Benutzer, ohne Administratorrecht). Gestartet wird mit `--autostart` (Fenster minimiert).
use std::path::Path;

pub const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "DaedyxLauncher";

pub struct Autostart {
    /// Nur für Tests änderbar.
    pub run_key: String,
}

impl Default for Autostart {
    fn default() -> Self {
        Autostart {
            run_key: RUN_KEY.to_string(),
        }
    }
}

impl Autostart {
    #[cfg(windows)]
    pub fn is_enabled(&self) -> bool {
        use winreg::enums::HKEY_CURRENT_USER;
        use winreg::RegKey;
        RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(&self.run_key)
            .and_then(|k| k.get_value::<String, _>(VALUE_NAME))
            .is_ok()
    }

    #[cfg(windows)]
    pub fn set(&self, enabled: bool, exe: &Path) -> Result<(), String> {
        use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if enabled {
            let (key, _) = hkcu
                .create_subkey(&self.run_key)
                .map_err(|e| e.to_string())?;
            let command = format!("\"{}\" --autostart", exe.display());
            key.set_value(VALUE_NAME, &command)
                .map_err(|e| e.to_string())
        } else {
            match hkcu.open_subkey_with_flags(&self.run_key, KEY_SET_VALUE) {
                Ok(key) => match key.delete_value(VALUE_NAME) {
                    Ok(()) => Ok(()),
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                    Err(e) => Err(e.to_string()),
                },
                Err(_) => Ok(()),
            }
        }
    }

    #[cfg(not(windows))]
    pub fn is_enabled(&self) -> bool {
        false
    }

    #[cfg(not(windows))]
    pub fn set(&self, _enabled: bool, _exe: &Path) -> Result<(), String> {
        Err("Start beim Anmelden gibt es nur unter Windows".into())
    }
}
