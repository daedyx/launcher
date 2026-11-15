use crate::error::UpdateError;
use crate::version;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PackageKind {
    NsisSilent,
    Dmg,
}

impl PackageKind {
    pub fn file_extension(self) -> &'static str {
        match self {
            PackageKind::NsisSilent => "exe",
            PackageKind::Dmg => "dmg",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Package {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub kind: PackageKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Release {
    pub version: String,
    pub date: String,
    #[serde(default)]
    pub notes: String,
    pub packages: BTreeMap<String, Package>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct App {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: String,
    /// Wer die App herausgibt (Anzeige in der Oberfläche); fehlt es, bleibt das Feld leer.
    #[serde(default)]
    pub author: String,
    /// Name des Uninstall-Schlüssels der App in der Windows-Registry (`HKCU\...\Uninstall\<Name>`);
    /// der Launcher findet darüber eine schon vorhandene Installation und ihre Version.
    #[serde(
        default,
        rename = "registryKey",
        skip_serializing_if = "Option::is_none"
    )]
    pub registry_key: Option<String>,
    pub releases: Vec<Release>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Catalog {
    pub schema: u32,
    #[serde(rename = "minLauncherVersion")]
    pub min_launcher_version: String,
    pub apps: Vec<App>,
}

/// App-Kennungen werden in Dateinamen verwendet und dürfen nur aus a-z, 0-9 und `-` bestehen.
pub fn is_valid_app_id(id: &str) -> bool {
    let mut chars = id.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

impl Catalog {
    pub fn parse(json: &[u8]) -> Result<Catalog, UpdateError> {
        // Ein UTF-8-BOM am Anfang (Windows-Editoren) ist erlaubt.
        let json = json.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(json);
        let catalog: Catalog =
            serde_json::from_slice(json).map_err(|e| UpdateError::CatalogInvalid(e.to_string()))?;
        if catalog.schema != SCHEMA_VERSION {
            return Err(UpdateError::SchemaUnsupported {
                found: catalog.schema,
                expected: SCHEMA_VERSION,
            });
        }
        for app in &catalog.apps {
            if !is_valid_app_id(&app.id) {
                return Err(UpdateError::CatalogInvalid(format!(
                    "ungültige App-Kennung: {}",
                    app.id
                )));
            }
        }
        Ok(catalog)
    }

    pub fn app(&self, id: &str) -> Result<&App, UpdateError> {
        self.apps
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| UpdateError::UnknownApp(id.to_string()))
    }

    pub fn check_launcher(&self, launcher_version: &str) -> Result<(), UpdateError> {
        let need = version::parse(&self.min_launcher_version)?;
        let have = version::parse(launcher_version)?;
        if have < need {
            return Err(UpdateError::LauncherTooOld {
                need: self.min_launcher_version.clone(),
                have: launcher_version.to_string(),
            });
        }
        Ok(())
    }
}

impl App {
    /// Die Veröffentlichung mit der höchsten Versionsnummer.
    pub fn latest(&self) -> Result<Option<&Release>, UpdateError> {
        let mut best: Option<(&Release, semver::Version)> = None;
        for release in &self.releases {
            let v = version::parse(&release.version)?;
            match &best {
                Some((_, bv)) if *bv >= v => {}
                _ => best = Some((release, v)),
            }
        }
        Ok(best.map(|(r, _)| r))
    }
}

impl Release {
    pub fn package_for(&self, app_id: &str, platform: &str) -> Result<&Package, UpdateError> {
        self.packages
            .get(platform)
            .ok_or_else(|| UpdateError::NoPackage {
                app: app_id.to_string(),
                platform: platform.to_string(),
            })
    }
}

pub fn current_platform() -> &'static str {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "macos-x64"
    } else {
        "unsupported"
    }
}
