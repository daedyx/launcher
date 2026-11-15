use crate::errors::error_message;
use crate::i18n::Texts;
use daedyx_update::error::UpdateError;
use launcher_core::error::CoreError;

fn all_variants() -> Vec<CoreError> {
    vec![
        UpdateError::CatalogInvalid("x".into()).into(),
        UpdateError::SchemaUnsupported {
            found: 2,
            expected: 1,
        }
        .into(),
        UpdateError::LauncherTooOld {
            need: "2.0.0".into(),
            have: "1.0.0".into(),
        }
        .into(),
        UpdateError::SignatureInvalid.into(),
        UpdateError::ChecksumMismatch.into(),
        UpdateError::NoPackage {
            app: "desk".into(),
            platform: "windows-x64".into(),
        }
        .into(),
        UpdateError::UnknownApp("desk".into()).into(),
        UpdateError::BadVersion("x".into()).into(),
        UpdateError::Io("x".into()).into(),
        UpdateError::KeyInvalid.into(),
        CoreError::AppRunning("desk".into()),
        CoreError::CatalogNotFound("C:\\x".into()),
        CoreError::ConfigIncomplete,
        CoreError::SourceNotAllowed("http://x".into()),
        CoreError::Download("x".into()),
        CoreError::InstallerFailed(5),
        CoreError::PlatformUnsupported("x".into()),
        CoreError::Busy("desk".into()),
        CoreError::NotInstalled("desk".into()),
        CoreError::NotLaunchable("x".into()),
        CoreError::Io("x".into()),
    ]
}

#[test]
fn jeder_fehler_hat_einen_vollstaendigen_deutschen_text() {
    let t = Texts::de();
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

#[test]
fn sicherheitsfehler_sagen_klar_dass_nichts_passiert_ist() {
    let t = Texts::de();
    let sig = error_message(&t, &UpdateError::SignatureInvalid.into());
    assert!(sig.contains("Echtheit") && sig.contains("nichts"), "{sig}");
    let sum = error_message(&t, &UpdateError::ChecksumMismatch.into());
    assert!(sum.contains("nichts installiert"), "{sum}");
}
