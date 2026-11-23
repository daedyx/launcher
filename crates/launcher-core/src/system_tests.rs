use crate::system::{RealSystem, SystemInfo};
use std::path::PathBuf;

#[test]
fn unsichere_schluesselnamen_werden_nicht_gelesen() {
    let sys = RealSystem::default();
    for key in ["", "..\\x", "a\\b", "a/b"] {
        assert_eq!(sys.uninstall_entry(key), None, "{key:?}");
    }
}

#[cfg(windows)]
mod windows_only {
    use super::*;
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    /// Test-Wurzel in der Registry, die nach dem Test wieder verschwindet.
    struct TestRoot(String);

    impl TestRoot {
        fn new() -> TestRoot {
            let path = format!(r"Software\DaedyxLauncherTest\{}", std::process::id());
            TestRoot(path)
        }
        fn system(&self) -> RealSystem {
            RealSystem {
                uninstall_base: self.0.clone(),
            }
        }
        fn write(&self, key: &str, version: Option<&str>, location: Option<&str>) {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (k, _) = hkcu.create_subkey(format!(r"{}\{}", self.0, key)).unwrap();
            if let Some(v) = version {
                k.set_value("DisplayVersion", &v).unwrap();
            }
            if let Some(l) = location {
                k.set_value("InstallLocation", &l).unwrap();
            }
        }
    }

    impl Drop for TestRoot {
        fn drop(&mut self) {
            let _ = RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(&self.0);
        }
    }

    #[test]
    fn uninstall_eintrag_wird_gelesen_anfuehrungszeichen_entfernt() {
        let root = TestRoot::new();
        root.write(
            "Daedyx Desk",
            Some("0.1.0"),
            Some("\"C:\\Users\\x\\AppData\\Local\\Daedyx Desk\""),
        );
        let e = root.system().uninstall_entry("Daedyx Desk").unwrap();
        assert_eq!(e.display_version.as_deref(), Some("0.1.0"));
        assert_eq!(
            e.install_location,
            Some(PathBuf::from(r"C:\Users\x\AppData\Local\Daedyx Desk"))
        );
    }

    #[test]
    fn fehlender_eintrag_ist_none_und_fehlende_werte_sind_none() {
        let root = TestRoot::new();
        assert_eq!(root.system().uninstall_entry("Gibt es nicht"), None);
        root.write("Nur Version", Some("2.0.0"), None);
        let e = root.system().uninstall_entry("Nur Version").unwrap();
        assert_eq!(e.display_version.as_deref(), Some("2.0.0"));
        assert_eq!(e.install_location, None);
    }

    /// Kopie eines harmlosen Systemprogramms, das eine Weile läuft, in einem Wegwerfordner.
    #[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-005")]
    fn start_helper(dir: &std::path::Path) -> std::process::Child {
        let system32 = PathBuf::from(std::env::var("SystemRoot").unwrap()).join("System32");
        let exe = dir.join("hilfsprogramm.exe");
        std::fs::copy(system32.join("PING.EXE"), &exe).unwrap();
        std::process::Command::new(exe)
            .args(["-n", "30", "127.0.0.1"])
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap()
    }

    #[test]
    fn laufendes_programm_im_ordner_wird_erkannt() {
        let dir = tempfile::tempdir().unwrap();
        let other = tempfile::tempdir().unwrap();
        let mut child = start_helper(dir.path());
        std::thread::sleep(std::time::Duration::from_millis(400));
        let sys = RealSystem::default();
        let result = sys.running(&[dir.path().to_path_buf(), other.path().to_path_buf()]);
        assert_eq!(result, [true, false]);
        child.kill().unwrap();
        child.wait().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert_eq!(sys.running(&[dir.path().to_path_buf()]), [false]);
    }

    #[test]
    fn ordnername_mit_gleichem_anfang_zaehlt_nicht() {
        // `...\app` darf nicht auf ein Programm in `...\app-alt` zutreffen.
        let base = tempfile::tempdir().unwrap();
        let alt = base.path().join("app-alt");
        std::fs::create_dir_all(&alt).unwrap();
        let mut child = start_helper(&alt);
        std::thread::sleep(std::time::Duration::from_millis(400));
        let sys = RealSystem::default();
        assert_eq!(sys.running(&[base.path().join("app")]), [false]);
        assert_eq!(sys.running(std::slice::from_ref(&alt)), [true]);
        child.kill().unwrap();
        child.wait().unwrap();
    }
}

#[cfg(windows)]
#[test]
fn programm_wird_unabhaengig_gestartet_und_fehler_werden_gemeldet() {
    let sys = RealSystem::default();
    let whoami = PathBuf::from(std::env::var("SystemRoot").unwrap()).join(r"System32\whoami.exe");
    assert!(sys.start_detached(&whoami, &["/?"]).is_ok());
    let missing = PathBuf::from(r"C:\gibt-es-nicht\x.exe");
    assert!(sys.start_detached(&missing, &[]).is_err());
}
