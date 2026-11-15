use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static BRAND_INSTALLED: AtomicBool = AtomicBool::new(false);

/// Schrift für die Marke und große Überschriften (Bricolage Grotesque). Ist sie nicht geladen
/// (zum Beispiel in Tests), fällt sie auf die Hauptschrift zurück, statt abzustürzen.
pub fn brand() -> FontFamily {
    if BRAND_INSTALLED.load(Ordering::Relaxed) {
        FontFamily::Name("brand".into())
    } else {
        FontFamily::Proportional
    }
}

/// Space Grotesk (Hauptschrift), Space Mono (kleine Beschriftungen) und Bricolage Grotesque
/// (Marke), alle SIL Open Font License. Die Lizenztexte liegen neben den Dateien in `assets/fonts`.
pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    for (name, data) in [
        (
            "space-grotesk",
            &include_bytes!("../assets/fonts/SpaceGrotesk.ttf")[..],
        ),
        (
            "space-mono",
            &include_bytes!("../assets/fonts/SpaceMono-Regular.ttf")[..],
        ),
        (
            "bricolage",
            &include_bytes!("../assets/fonts/BricolageGrotesque.ttf")[..],
        ),
    ] {
        fonts
            .font_data
            .insert(name.into(), Arc::new(FontData::from_static(data)));
    }
    // Die eingebauten Schriften bleiben dahinter als Reserve für fehlende Zeichen.
    for (family, name) in [
        (FontFamily::Proportional, "space-grotesk"),
        (FontFamily::Monospace, "space-mono"),
    ] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.insert(0, name.into());
        }
    }
    let mut brand_list = vec!["bricolage".to_string()];
    brand_list.extend(
        fonts
            .families
            .get(&FontFamily::Proportional)
            .cloned()
            .unwrap_or_default(),
    );
    fonts
        .families
        .insert(FontFamily::Name("brand".into()), brand_list);
    BRAND_INSTALLED.store(true, Ordering::Relaxed);
    ctx.set_fonts(fonts);
}
