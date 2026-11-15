use crate::i18n::Texts;
use daedyx_update::error::UpdateError;
use launcher_core::error::CoreError;

/// Übersetzt einen Fehler des Kerns in einen verständlichen Text der Sprachdatei.
/// Die `match`-Ausdrücke sind vollständig: Kommt eine Variante hinzu, bricht die Übersetzung ab.
pub fn error_message(t: &Texts, e: &CoreError) -> String {
    match e {
        CoreError::Update(u) => match u {
            UpdateError::SignatureInvalid => t.t("error.signature"),
            UpdateError::KeyInvalid => t.t("error.key_invalid"),
            UpdateError::ChecksumMismatch => t.t("error.checksum"),
            UpdateError::LauncherTooOld { need, have } => {
                t.tf("error.launcher_old", &[("need", need), ("have", have)])
            }
            UpdateError::SchemaUnsupported { .. } => t.t("error.schema"),
            UpdateError::CatalogInvalid(d) => t.tf("error.catalog_invalid", &[("detail", d)]),
            UpdateError::NoPackage { app, platform } => {
                t.tf("error.no_package", &[("app", app), ("platform", platform)])
            }
            UpdateError::UnknownApp(a) => t.tf("error.unknown_app", &[("app", a)]),
            UpdateError::BadVersion(v) => t.tf("error.bad_version", &[("version", v)]),
            UpdateError::Io(d) => t.tf("error.io", &[("detail", d)]),
        },
        CoreError::SourceNotAllowed(s) => t.tf("error.source", &[("detail", s)]),
        CoreError::Download(d) => t.tf("error.download", &[("detail", d)]),
        CoreError::InstallerFailed(code) => t.tf("error.installer", &[("code", &code.to_string())]),
        CoreError::PlatformUnsupported(p) => t.tf("error.platform", &[("detail", p)]),
        CoreError::Busy(a) => t.tf("error.busy", &[("app", a)]),
        CoreError::AppRunning(a) => t.tf("error.app_running", &[("app", a)]),
        CoreError::CatalogNotFound(p) => t.tf("error.catalog_not_found", &[("detail", p)]),
        CoreError::ConfigIncomplete => t.t("setup.missing"),
        CoreError::NotInstalled(a) => t.tf("error.not_installed", &[("app", a)]),
        CoreError::NotLaunchable(d) => t.tf("error.not_launchable", &[("detail", d)]),
        CoreError::Io(d) => t.tf("error.io", &[("detail", d)]),
    }
}
