use crate::error::CoreError;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

/// Katalogquelle: ein Ordner (lokaler Katalog) oder eine Internetadresse.
#[derive(Debug, Clone)]
pub enum Source {
    Dir(PathBuf),
    Http(String),
}

const MAX_CATALOG_BYTES: u64 = 5 * 1024 * 1024;

/// `http://` ist nur für den eigenen Rechner erlaubt; der Hostname muss genau passen.
fn is_local_http(url: &str) -> bool {
    for prefix in ["http://127.0.0.1", "http://localhost"] {
        if let Some(rest) = url.strip_prefix(prefix) {
            return rest.is_empty() || rest.starts_with(':') || rest.starts_with('/');
        }
    }
    false
}

fn http_allowed(url: &str) -> bool {
    url.starts_with("https://") || is_local_http(url)
}

/// Relative Pfade und Adressen dürfen den Quellordner nie verlassen.
fn check_relative(rel: &str) -> Result<(), CoreError> {
    let bad = rel.is_empty()
        || rel.starts_with('/')
        || rel.starts_with('\\')
        || rel.contains(':')
        || rel.split(['/', '\\']).any(|s| s == "..");
    if bad {
        Err(CoreError::SourceNotAllowed(rel.to_string()))
    } else {
        Ok(())
    }
}

impl Source {
    pub fn parse(s: &str) -> Result<Source, CoreError> {
        if s.starts_with("http://") || s.starts_with("https://") {
            if !http_allowed(s) {
                return Err(CoreError::SourceNotAllowed(s.to_string()));
            }
            Ok(Source::Http(s.trim_end_matches('/').to_string()))
        } else {
            Ok(Source::Dir(PathBuf::from(s)))
        }
    }

    /// Kleine Dateien (Katalog, Signatur) vollständig laden.
    pub fn fetch_bytes(&self, name: &str) -> Result<Vec<u8>, CoreError> {
        check_relative(name)?;
        match self {
            Source::Dir(root) => Ok(std::fs::read(root.join(name))?),
            Source::Http(base) => {
                let (reader, _) = http_open(&format!("{base}/{name}"))?;
                let mut buf = Vec::new();
                reader.take(MAX_CATALOG_BYTES + 1).read_to_end(&mut buf)?;
                if buf.len() as u64 > MAX_CATALOG_BYTES {
                    return Err(CoreError::Download("Datei ist zu groß".into()));
                }
                Ok(buf)
            }
        }
    }

    /// Ein Paket in eine Datei laden. `url` ist eine absolute Adresse oder relativ zur Quelle.
    pub fn fetch_to_file(
        &self,
        url: &str,
        dest: &Path,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<(), CoreError> {
        match self {
            Source::Dir(root) => {
                if url.contains("://") {
                    return Err(CoreError::SourceNotAllowed(url.to_string()));
                }
                check_relative(url)?;
                let file = std::fs::File::open(root.join(url))?;
                let total = file.metadata()?.len();
                copy_with_progress(file, dest, Some(total), progress)
            }
            Source::Http(base) => {
                let full = if url.contains("://") {
                    if !http_allowed(url) {
                        return Err(CoreError::SourceNotAllowed(url.to_string()));
                    }
                    url.to_string()
                } else {
                    check_relative(url)?;
                    format!("{base}/{url}")
                };
                let (reader, total) = http_open(&full)?;
                copy_with_progress(reader, dest, total, progress)
            }
        }
    }
}

fn http_open(url: &str) -> Result<(Box<dyn Read>, Option<u64>), CoreError> {
    let resp = ureq::get(url)
        .call()
        .map_err(|e| CoreError::Download(e.to_string()))?;
    let total = resp
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());
    Ok((Box::new(resp.into_body().into_reader()), total))
}

fn copy_with_progress(
    mut reader: impl Read,
    dest: &Path,
    total: Option<u64>,
    progress: &mut dyn FnMut(u64, Option<u64>),
) -> Result<(), CoreError> {
    let mut out = std::fs::File::create(dest)?;
    let mut buf = [0u8; 64 * 1024];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        out.write_all(buf.get(..n).unwrap_or_default())?;
        done = done.saturating_add(u64::try_from(n).unwrap_or(u64::MAX));
        progress(done, total);
    }
    out.flush()?;
    Ok(())
}
