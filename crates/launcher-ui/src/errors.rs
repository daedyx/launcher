use crate::i18n::Texts;
use launcher_core::apps;
use launcher_core::daedyx_update::reason::BlockKind;
use launcher_core::error::CoreError;

/// Text zu einem Sperrgrund der Update-Bibliothek (plan-update-bibliothek 7). `source` ist die erwartete Quelle für
/// „von Hand laden“: nur als Text, ohne Link und ohne „sicher“ (Review M-U1-1).
pub fn block_message(t: &Texts, kind: BlockKind, source: &str) -> String {
    match kind {
        BlockKind::Offline => t.t("error.blocked.offline"),
        BlockKind::SourceUnavailable => t.t("error.blocked.source"),
        BlockKind::NotTrusted => t.t("error.blocked.not_trusted"),
        BlockKind::NeedsManualUpdate => t.tf("error.blocked.manual", &[("quelle", source)]),
        BlockKind::SourceWentBack => t.t("error.blocked.went_back"),
        BlockKind::Interrupted => t.t("error.blocked.interrupted"),
        BlockKind::NoSpace => t.t("error.blocked.no_space"),
        BlockKind::LocalFile => t.t("error.blocked.local_file"),
    }
}

/// Übersetzt einen Fehler des Kerns in einen verständlichen Text der Sprachdatei.
/// Die `match`-Ausdrücke sind vollständig: Kommt eine Variante hinzu, bricht die Übersetzung ab.
pub fn error_message(t: &Texts, e: &CoreError) -> String {
    match e {
        CoreError::Blocked { app, blocked } => {
            let source = apps::find(app)
                .and_then(|a| a.policy())
                .map(|p| p.expected_source())
                .unwrap_or_default();
            block_message(t, blocked.kind, &source)
        }
        CoreError::NotOffered(a) => t.tf("error.not_offered", &[("app", a)]),
        CoreError::LauncherTooOld { need, have } => {
            t.tf("error.launcher_old", &[("need", need), ("have", have)])
        }
        CoreError::UnknownApp(a) => t.tf("error.unknown_app", &[("app", a)]),
        CoreError::InstallerFailed(code) => t.tf("error.installer", &[("code", &code.to_string())]),
        CoreError::PlatformUnsupported(p) => t.tf("error.platform", &[("detail", p)]),
        CoreError::Busy(a) => t.tf("error.busy", &[("app", a)]),
        CoreError::AppRunning(a) => t.tf("error.app_running", &[("app", a)]),
        CoreError::NotInstalled(a) => t.tf("error.not_installed", &[("app", a)]),
        CoreError::NotLaunchable(d) => t.tf("error.not_launchable", &[("detail", d)]),
        CoreError::Io(d) => t.tf("error.io", &[("detail", d)]),
    }
}
