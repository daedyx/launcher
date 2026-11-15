use std::path::Path;

/// Die installierte Version fragt der Launcher bei der App selbst ab: Jede Daedyx-App legt
/// im Installationsordner eine `app.json` mit `{"version": "..."}` ab. Später (echte
/// Windows-Installer) kommen der Deinstallationseintrag der Registry und die macOS-Info.plist dazu.
pub fn read_installed_version(install_dir: &Path) -> Option<String> {
    let bytes = std::fs::read(install_dir.join("app.json")).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value.get("version")?.as_str().map(|s| s.to_string())
}
