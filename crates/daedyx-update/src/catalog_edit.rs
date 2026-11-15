//! Katalog bearbeiten: Release einer App einfügen (für das Werkzeug `daedyx-catalog`).
use crate::catalog::{is_valid_app_id, App, Catalog, Release};
use crate::error::UpdateError;
use crate::version;

/// Angaben zur App selbst. Nur gesetzte Felder ändern den Eintrag; eine neue App braucht `name`.
#[derive(Debug, Clone, Default)]
pub struct AppMeta {
    pub name: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub author: Option<String>,
    pub registry_key: Option<String>,
}

fn invalid(why: &str) -> UpdateError {
    UpdateError::CatalogInvalid(why.to_string())
}

/// Fügt `release` in den Eintrag `app_id` ein (legt ihn an, wenn es ihn noch nicht gibt).
/// Eine schon vorhandene Version wird nur mit `replace` ersetzt. Bei einem Fehler bleibt der
/// Katalog unverändert.
pub fn add_release(
    catalog: &mut Catalog,
    app_id: &str,
    release: Release,
    meta: AppMeta,
    replace: bool,
) -> Result<(), UpdateError> {
    if !is_valid_app_id(app_id) {
        return Err(invalid(&format!("ungültige App-Kennung: {app_id}")));
    }
    version::parse(&release.version)?;
    let mut work = catalog.clone();
    let index = match work.apps.iter().position(|a| a.id == app_id) {
        Some(i) => i,
        None => {
            let name = meta
                .name
                .clone()
                .ok_or_else(|| invalid("Für eine neue App ist ein Name nötig"))?;
            work.apps.push(App {
                id: app_id.to_string(),
                name,
                description: String::new(),
                license: String::new(),
                author: String::new(),
                registry_key: None,
                releases: vec![],
            });
            work.apps.len() - 1
        }
    };
    let app = &mut work.apps[index];
    if let Some(v) = meta.name {
        app.name = v;
    }
    if let Some(v) = meta.description {
        app.description = v;
    }
    if let Some(v) = meta.license {
        app.license = v;
    }
    if let Some(v) = meta.author {
        app.author = v;
    }
    if let Some(v) = meta.registry_key {
        app.registry_key = Some(v);
    }
    match app
        .releases
        .iter()
        .position(|r| r.version == release.version)
    {
        Some(_) if !replace => {
            return Err(invalid(&format!(
                "Version {} gibt es schon (mit --replace ersetzen)",
                release.version
            )))
        }
        Some(i) => app.releases[i] = release,
        None => app.releases.push(release),
    }
    *catalog = work;
    Ok(())
}
