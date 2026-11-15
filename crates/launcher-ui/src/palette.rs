//! Farben des Daedyx Launchers: Indigo als Grundfarbe, Bernstein nur als Signal (Updates,
//! Hinweise). Es gibt je eine helle und eine dunkle Palette; die Tests sichern die Lesbarkeit.
use eframe::egui::{self, Color32};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    /// Fläche hinter den Karten
    pub bg: Color32,
    pub card: Color32,
    pub border: Color32,
    pub text: Color32,
    pub text2: Color32,
    /// Hauptknopf und Auswahl (Füllung)
    pub primary: Color32,
    pub on_primary: Color32,
    /// Die Hauptfarbe als Schriftfarbe auf Karten (Links, Versionen)
    pub primary_text: Color32,
    pub sidebar: Color32,
    pub sidebar_text: Color32,
    pub sidebar_active: Color32,
    /// Bernstein: Signalfarbe (Zierde, Leiste)
    pub amber: Color32,
    pub amber_bg: Color32,
    pub amber_text: Color32,
    pub green_bg: Color32,
    pub green_text: Color32,
    pub danger_bg: Color32,
    pub danger_text: Color32,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

impl Palette {
    pub fn light() -> Palette {
        Palette {
            bg: rgb(246, 245, 251),
            card: rgb(255, 255, 255),
            border: rgb(224, 223, 240),
            text: rgb(38, 37, 94),
            text2: rgb(95, 94, 148),
            primary: rgb(52, 51, 127),
            on_primary: rgb(255, 255, 255),
            primary_text: rgb(52, 51, 127),
            sidebar: rgb(52, 51, 127),
            sidebar_text: rgb(255, 255, 255),
            sidebar_active: rgb(74, 73, 160),
            amber: rgb(232, 163, 61),
            amber_bg: rgb(251, 235, 203),
            amber_text: rgb(122, 77, 8),
            green_bg: rgb(231, 244, 236),
            green_text: rgb(30, 107, 69),
            danger_bg: rgb(251, 227, 227),
            danger_text: rgb(140, 34, 34),
        }
    }

    pub fn dark() -> Palette {
        Palette {
            bg: rgb(19, 19, 42),
            card: rgb(28, 28, 58),
            border: rgb(52, 52, 96),
            text: rgb(236, 236, 250),
            text2: rgb(165, 165, 208),
            primary: rgb(156, 155, 240),
            on_primary: rgb(19, 19, 42),
            primary_text: rgb(176, 175, 245),
            sidebar: rgb(14, 14, 32),
            sidebar_text: rgb(236, 236, 250),
            sidebar_active: rgb(40, 40, 82),
            amber: rgb(240, 176, 80),
            amber_bg: rgb(58, 45, 16),
            amber_text: rgb(245, 200, 119),
            green_bg: rgb(18, 48, 31),
            green_text: rgb(125, 214, 162),
            danger_bg: rgb(58, 22, 22),
            danger_text: rgb(242, 160, 160),
        }
    }

    pub fn of(dark_mode: bool) -> Palette {
        if dark_mode {
            Palette::dark()
        } else {
            Palette::light()
        }
    }

    /// Die Palette passend zum Thema des Fensters.
    pub fn current(ui: &egui::Ui) -> Palette {
        Palette::of(ui.visuals().dark_mode)
    }
}

fn channel(c: u8) -> f32 {
    let s = c as f32 / 255.0;
    if s <= 0.03928 {
        s / 12.92
    } else {
        ((s + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(c: Color32) -> f32 {
    0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
}

/// Kontrastverhältnis zweier Farben nach WCAG (1 bis 21).
pub fn contrast(a: Color32, b: Color32) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}
