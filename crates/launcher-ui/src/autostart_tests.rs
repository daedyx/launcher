use crate::autostart::Autostart;
use std::path::Path;

#[cfg(windows)]
mod windows_only {
    use super::*;
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    struct TestKey(String);
    impl Drop for TestKey {
        fn drop(&mut self) {
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
        }
    }

    fn test_key() -> TestKey {
        TestKey(format!(
            r"Software\DaedyxLauncherTest\Run{}",
            std::process::id()
        ))
    }

    #[test]
    fn anmelde_start_wird_gesetzt_gelesen_und_entfernt() {
        let key = test_key();
        let a = Autostart {
            run_key: key.0.clone(),
        };
        assert!(!a.is_enabled());
        a.set(
            true,
            Path::new(r"C:\Programme\Daedyx Launcher\daedyx-launcher.exe"),
        )
        .unwrap();
        assert!(a.is_enabled());
        let value: String = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey(&key.0)
            .unwrap()
            .get_value("DaedyxLauncher")
            .unwrap();
        // Pfad in Anführungszeichen (Leerzeichen!) und der Schalter für den stillen Start
        assert_eq!(
            value,
            r#""C:\Programme\Daedyx Launcher\daedyx-launcher.exe" --autostart"#
        );
        a.set(false, Path::new("egal")).unwrap();
        assert!(!a.is_enabled());
        // Ausschalten ohne Eintrag ist kein Fehler
        a.set(false, Path::new("egal")).unwrap();
    }
}
