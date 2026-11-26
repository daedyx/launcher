//! Geprüftes Setup bis zum Start festhalten (Review U3 M-U3-1, plan-update-bibliothek 6 „Härtung“): Die Datei wird
//! so geöffnet, dass andere sie nur noch lesen dürfen (kein Schreiben, Umbenennen, Löschen), und die Prüfsumme wird
//! über genau diesen Griff noch einmal berechnet. Der Aufrufer hält [`Held`], bis Installation bzw. Start zurückkommt.
//! So kann niemand die Datei zwischen Prüfung und Start tauschen. Gleiche Wirkung wie `daedyx-sys::start_verified` in
//! den Apps, hier nur mit der Standardbibliothek (kein `unsafe`).
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;

/// Warum die Datei nicht festgehalten wurde.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeldError {
    Missing,
    /// Inhalt passt nicht mehr zur geprüften Prüfsumme.
    Changed,
    Failed,
}

/// Gegen Schreiben, Umbenennen und Löschen gesperrte, noch einmal geprüfte Datei.
#[derive(Debug)]
pub struct Held {
    _file: File,
}

fn open_shared_read(path: &Path) -> std::io::Result<File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_SHARE_READ: u32 = 0x1;
        options.share_mode(FILE_SHARE_READ);
    }
    options.open(path)
}

fn digest(file: &mut File) -> Option<[u8; 32]> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        match file.read(&mut buf) {
            Ok(0) => return Some(hasher.finalize().into()),
            Ok(n) => hasher.update(buf.get(..n).unwrap_or_default()),
            Err(e) if e.kind() == ErrorKind::Interrupted => {}
            Err(_) => return None,
        }
    }
}

impl Held {
    /// Öffnet `path` gesperrt und prüft `sha256` über den gesperrten Griff.
    pub fn open(path: &Path, sha256: &[u8; 32]) -> Result<Held, HeldError> {
        let mut file = open_shared_read(path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => HeldError::Missing,
            _ => HeldError::Failed,
        })?;
        match digest(&mut file) {
            Some(got) if &got == sha256 => Ok(Held { _file: file }),
            Some(_) => Err(HeldError::Changed),
            None => Err(HeldError::Failed),
        }
    }
}
