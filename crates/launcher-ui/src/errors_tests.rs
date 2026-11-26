use crate::errors::{block_message, error_message};
use crate::i18n::{Language, Texts};
use launcher_core::daedyx_update::reason::{BlockKind, Blocked};
use launcher_core::error::CoreError;

const KINDS: [BlockKind; 8] = [
    BlockKind::Offline,
    BlockKind::SourceUnavailable,
    BlockKind::NotTrusted,
    BlockKind::NeedsManualUpdate,
    BlockKind::SourceWentBack,
    BlockKind::Interrupted,
    BlockKind::NoSpace,
    BlockKind::LocalFile,
];

fn blocked(kind: BlockKind) -> CoreError {
    CoreError::Blocked {
        app: "desk".into(),
        blocked: Blocked {
            kind,
            detail: "x".into(),
        },
    }
}

fn all_variants() -> Vec<CoreError> {
    let mut v: Vec<CoreError> = KINDS.iter().map(|k| blocked(*k)).collect();
    v.extend([
        CoreError::NotOffered("desk".into()),
        CoreError::LauncherTooOld {
            need: "2.0.0".into(),
            have: "1.0.0".into(),
        },
        CoreError::UnknownApp("board".into()),
        CoreError::AppRunning("desk".into()),
        CoreError::InstallerFailed(5),
        CoreError::PlatformUnsupported("x".into()),
        CoreError::Busy("desk".into()),
        CoreError::NotInstalled("desk".into()),
        CoreError::NotLaunchable("x".into()),
        CoreError::Io("x".into()),
    ]);
    v
}

#[test]
fn jeder_fehler_hat_einen_vollstaendigen_text_in_jeder_sprache() {
    for t in [Texts::de(), Texts::for_language(Language::En)] {
        for e in all_variants() {
            let text = error_message(&t, &e);
            assert!(!text.contains("[["), "{e:?}: Schlüssel fehlt: {text}");
            assert!(
                !text.contains('{'),
                "{e:?}: Platzhalter nicht ersetzt: {text}"
            );
            assert!(text.chars().count() > 20, "{e:?}: zu knapp: {text}");
        }
    }
}

#[test]
fn ein_abgelehnter_nachweis_sagt_klar_dass_nichts_installiert_wurde() {
    let t = Texts::de();
    let text = error_message(&t, &blocked(BlockKind::NotTrusted));
    assert!(
        text.contains("nicht sicher von Daedyx") && text.contains("nicht installiert"),
        "{text}"
    );
}

#[test]
fn von_hand_laden_nennt_die_quelle_nur_als_text_ohne_link_und_ohne_sicher() {
    let t = Texts::de();
    let text = error_message(&t, &blocked(BlockKind::NeedsManualUpdate));
    assert!(text.contains("github.com/daedyx/desk"), "{text}");
    assert!(
        !text.contains("https://") && !text.to_lowercase().contains("sicher"),
        "{text}"
    );
    assert!(
        !block_message(&t, BlockKind::Offline, "").contains("Fehler"),
        "kein Fehlerton ohne Netz"
    );
}
