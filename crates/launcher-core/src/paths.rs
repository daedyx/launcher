use std::path::PathBuf;

/// Gemeinsamer Datenordner von Kommandozeile und Oberfläche.
pub fn default_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Daedyx")
        .join("Launcher")
}
