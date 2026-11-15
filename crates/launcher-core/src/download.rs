use crate::error::CoreError;
use crate::source::Source;
use daedyx_update::catalog::Package;
use daedyx_update::error::UpdateError;
use daedyx_update::verify::verify_sha256;
use std::path::{Path, PathBuf};

/// Lädt ein Paket und gibt es erst frei, wenn Größe und SHA-256 stimmen.
/// Bei jedem Fehler bleibt keine Teil- oder Endatei zurück.
pub fn download_verified(
    source: &Source,
    package: &Package,
    dest_dir: &Path,
    file_name: &str,
    progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<PathBuf, CoreError> {
    std::fs::create_dir_all(dest_dir)?;
    let part = dest_dir.join(format!("{file_name}.part"));
    let done = dest_dir.join(file_name);
    let _ = std::fs::remove_file(&part);
    let _ = std::fs::remove_file(&done);

    if let Err(e) = source.fetch_to_file(&package.url, &part, progress) {
        let _ = std::fs::remove_file(&part);
        return Err(e);
    }
    let size = std::fs::metadata(&part)?.len();
    if size != package.size {
        let _ = std::fs::remove_file(&part);
        return Err(UpdateError::ChecksumMismatch.into());
    }
    if let Err(e) = verify_sha256(&part, &package.sha256) {
        let _ = std::fs::remove_file(&part);
        return Err(e.into());
    }
    std::fs::rename(&part, &done)?;
    Ok(done)
}
