use crate::error::CoreError;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Sperrdatei je App: verhindert, dass Launcher und App (oder zwei Launcher) gleichzeitig
/// installieren. Eine Sperre, die älter als eine Stunde ist, gilt als übrig geblieben.
#[derive(Debug)]
pub struct Lock {
    path: PathBuf,
}

const STALE_AFTER: Duration = Duration::from_secs(60 * 60);

impl Lock {
    pub fn acquire(dir: &Path, app_id: &str) -> Result<Lock, CoreError> {
        std::fs::create_dir_all(dir)?;
        let path = dir.join(format!("{app_id}.lock"));
        for _ in 0..2 {
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(_) => return Ok(Lock { path }),
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    let age = std::fs::metadata(&path)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| SystemTime::now().duration_since(t).ok());
                    match age {
                        Some(a) if a > STALE_AFTER => {
                            let _ = std::fs::remove_file(&path);
                        }
                        _ => return Err(CoreError::Busy(app_id.to_string())),
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }
        Err(CoreError::Busy(app_id.to_string()))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
