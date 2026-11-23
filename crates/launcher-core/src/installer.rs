use crate::error::CoreError;
use daedyx_update::catalog::PackageKind;
use std::path::Path;

/// Führt Installation und Deinstallation aus. Die Tests ersetzen das durch eine Attrappe.
pub trait Installer: Send + Sync {
    fn install(&self, package: &Path, kind: PackageKind, dir: &Path) -> Result<(), CoreError>;
    fn uninstall(&self, dir: &Path) -> Result<(), CoreError>;
}

/// Der echte Installer des Systems. Windows: NSIS-Setup still pro Benutzer.
/// macOS (DMG) kommt in Meilenstein L5.
pub struct SystemInstaller;

impl Installer for SystemInstaller {
    fn install(&self, package: &Path, kind: PackageKind, dir: &Path) -> Result<(), CoreError> {
        match kind {
            PackageKind::NsisSilent => run_nsis_install(package, dir),
            other => Err(CoreError::PlatformUnsupported(format!("{other:?}"))),
        }
    }

    fn uninstall(&self, dir: &Path) -> Result<(), CoreError> {
        run_nsis_uninstall(dir)?;
        if dir.exists() {
            std::fs::remove_dir_all(dir)?;
        }
        Ok(())
    }
}

#[cfg(windows)]
#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-002")]
fn run_nsis_install(package: &Path, dir: &Path) -> Result<(), CoreError> {
    use std::os::windows::process::CommandExt;
    // NSIS verlangt `/D=` als letztes Argument und ohne Anführungszeichen, auch bei Leerzeichen.
    let status = std::process::Command::new(package)
        .raw_arg("/S")
        .raw_arg(format!("/D={}", dir.display()))
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(CoreError::InstallerFailed(status.code().unwrap_or(-1)))
    }
}

#[cfg(not(windows))]
fn run_nsis_install(_package: &Path, _dir: &Path) -> Result<(), CoreError> {
    Err(CoreError::PlatformUnsupported("NsisSilent".into()))
}

#[cfg(windows)]
#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-002")]
fn run_nsis_uninstall(dir: &Path) -> Result<(), CoreError> {
    use std::os::windows::process::CommandExt;
    let uninstaller = dir.join("uninstall.exe");
    if !uninstaller.exists() {
        return Ok(());
    }
    // `_?=` lässt den Deinstaller am Platz laufen, sodass wir auf sein Ende warten können.
    let status = std::process::Command::new(&uninstaller)
        .raw_arg("/S")
        .raw_arg(format!("_?={}", dir.display()))
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(CoreError::InstallerFailed(status.code().unwrap_or(-1)))
    }
}

#[cfg(not(windows))]
fn run_nsis_uninstall(_dir: &Path) -> Result<(), CoreError> {
    Err(CoreError::PlatformUnsupported("NsisSilent".into()))
}
