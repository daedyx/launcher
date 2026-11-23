use crate::error::CoreError;
use crate::service::Launcher;
use crate::state::State;
use daedyx_update::catalog::is_valid_app_id;
use daedyx_update::error::UpdateError;
use std::path::{Component, Path, PathBuf};

fn not_launchable(why: &str) -> CoreError {
    CoreError::NotLaunchable(why.to_string())
}

/// Das Startprogramm steht in `app.json` (`exe`) und muss ein relativer Pfad im Installationsordner sein.
pub fn launch_target(install_dir: &Path) -> Result<PathBuf, CoreError> {
    let bytes = std::fs::read(install_dir.join("app.json"))
        .map_err(|_| not_launchable("app.json fehlt"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| not_launchable("app.json ist beschädigt"))?;
    let rel = value
        .get("exe")
        .and_then(|v| v.as_str())
        .ok_or_else(|| not_launchable("app.json nennt kein Startprogramm"))?;
    let path = Path::new(rel);
    let safe = !rel.is_empty()
        && !rel.contains(':')
        && path.components().all(|c| matches!(c, Component::Normal(_)));
    if !safe {
        return Err(not_launchable(
            "Das Startprogramm liegt nicht im Installationsordner",
        ));
    }
    let target = install_dir.join(path);
    if !target.is_file() {
        return Err(not_launchable("Das Startprogramm wurde nicht gefunden"));
    }
    Ok(target)
}

impl Launcher {
    /// Startet die installierte App und lässt sie unabhängig vom Launcher weiterlaufen.
    pub fn open(&self, app_id: &str) -> Result<(), CoreError> {
        if !is_valid_app_id(app_id) {
            return Err(UpdateError::UnknownApp(app_id.to_string()).into());
        }
        let state = State::load(&self.data_dir)?;
        let loc = self.locate(&state, app_id);
        if loc.version.is_none() {
            return Err(CoreError::NotInstalled(app_id.to_string()));
        }
        let dir = loc.dir;
        spawn_detached(&launch_target(&dir)?, &dir)
    }
}

#[cfg(windows)]
#[allow(clippy::disallowed_methods, reason = "SICHERHEIT-AUSNAHMEN A-003")]
fn spawn_detached(target: &Path, dir: &Path) -> Result<(), CoreError> {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    std::process::Command::new(target)
        .current_dir(dir)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn()
        .map(|_| ())
        .map_err(|e| CoreError::Io(e.to_string()))
}

#[cfg(not(windows))]
fn spawn_detached(_target: &Path, _dir: &Path) -> Result<(), CoreError> {
    Err(CoreError::PlatformUnsupported(
        "Öffnen (kommt mit L5)".into(),
    ))
}
