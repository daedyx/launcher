//! Marke: das Labyrinth-D (ein D mit Irrgartenlinie, nach Daedalus). Eine Zeichnung für alles:
//! die Seitenleiste (egui), das Fenster-Symbol (Pixelbild) und die Symboldatei des Setups.
//! Gezeichnet wird in einem Raster von 72 mal 72 Einheiten.
use eframe::egui::{self, Color32, Pos2, Stroke};

pub const INDIGO: Color32 = Color32::from_rgb(52, 51, 127);
pub const AMBER: Color32 = Color32::from_rgb(232, 163, 61);
const GRID: f32 = 72.0;
const STROKE_WIDTH: f32 = 3.0;

/// Eine Linie der Marke: Punkte, Farbe.
#[derive(Debug, Clone)]
pub struct MarkLine {
    pub points: Vec<(f32, f32)>,
    pub color: Color32,
}

/// Halbkreis rechts von (`cx`, `cy`) mit Radius `r` von oben nach unten.
fn half_circle(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    (0..=12)
        .map(|i| {
            let a = -std::f32::consts::FRAC_PI_2 + std::f32::consts::PI * i as f32 / 12.0;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// D-Umriss (weiß) und Irrgartenlinie (Bernstein).
pub fn mark_strokes() -> Vec<MarkLine> {
    let mut d = vec![(20.0, 18.0), (38.0, 18.0)];
    d.extend(half_circle(38.0, 36.0, 18.0));
    d.extend([(38.0, 54.0), (20.0, 54.0), (20.0, 18.0)]);
    let mut maze = vec![(28.0, 26.0), (38.0, 26.0)];
    maze.extend(half_circle(38.0, 36.0, 10.0));
    maze.extend([(38.0, 46.0), (28.0, 46.0), (28.0, 37.0), (36.0, 37.0)]);
    vec![
        MarkLine {
            points: d,
            color: Color32::WHITE,
        },
        MarkLine {
            points: maze,
            color: AMBER,
        },
    ]
}

/// Zeichnet die Marke in `rect`. Mit `background` steht sie auf einem indigofarbenen runden
/// Quadrat (App-Symbol), sonst frei (Seitenleiste, die selbst indigo ist).
pub fn paint_mark(painter: &egui::Painter, rect: egui::Rect, background: bool) {
    let scale = rect.width() / GRID;
    let at = |p: (f32, f32)| Pos2::new(rect.min.x + p.0 * scale, rect.min.y + p.1 * scale);
    if background {
        painter.rect_filled(
            egui::Rect::from_min_max(at((4.0, 4.0)), at((68.0, 68.0))),
            egui::CornerRadius::same((14.0 * scale) as u8),
            INDIGO,
        );
    }
    for line in mark_strokes() {
        let points: Vec<Pos2> = line.points.iter().map(|p| at(*p)).collect();
        painter.add(egui::Shape::line(
            points,
            Stroke::new(STROKE_WIDTH * scale, line.color),
        ));
    }
}

fn dist_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 == 0.0 {
        0.0
    } else {
        (((p.0 - a.0) * abx + (p.1 - a.1) * aby) / len2).clamp(0.0, 1.0)
    };
    let (cx, cy) = (a.0 + t * abx, a.1 + t * aby);
    ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt()
}

fn near_line(p: (f32, f32), line: &MarkLine) -> bool {
    line.points
        .windows(2)
        .any(|w| matches!(w, [a, b] if dist_to_segment(p, *a, *b) <= STROKE_WIDTH / 2.0))
}

fn in_rounded_square(p: (f32, f32)) -> bool {
    let (min, max, r) = (4.0_f32, 68.0_f32, 14.0_f32);
    let cx = p.0.clamp(min + r, max - r);
    let cy = p.1.clamp(min + r, max - r);
    p.0 >= min
        && p.0 <= max
        && p.1 >= min
        && p.1 <= max
        && ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt() <= r
}

/// Pixelbild (RGBA, `size` mal `size`) des App-Symbols; 4 mal 4 Stützpunkte je Pixel glätten die Kanten.
pub fn render_rgba(size: usize) -> Vec<u8> {
    let strokes = mark_strokes();
    let mut out = vec![0u8; size * size * 4];
    const N: usize = 4;
    for y in 0..size {
        for x in 0..size {
            let (mut r, mut g, mut b, mut cover) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for sy in 0..N {
                for sx in 0..N {
                    let p = (
                        (x as f32 + (sx as f32 + 0.5) / N as f32) * GRID / size as f32,
                        (y as f32 + (sy as f32 + 0.5) / N as f32) * GRID / size as f32,
                    );
                    if !in_rounded_square(p) {
                        continue;
                    }
                    let color = strokes
                        .iter()
                        .rev()
                        .find(|l| near_line(p, l))
                        .map(|l| l.color)
                        .unwrap_or(INDIGO);
                    r += color.r() as f32;
                    g += color.g() as f32;
                    b += color.b() as f32;
                    cover += 1.0;
                }
            }
            let i = (y * size + x) * 4;
            if let Some(px) = out.get_mut(i..i + 4).filter(|_| cover > 0.0) {
                px.copy_from_slice(&[
                    (r / cover).round() as u8,
                    (g / cover).round() as u8,
                    (b / cover).round() as u8,
                    (255.0 * cover / (N * N) as f32).round() as u8,
                ]);
            }
        }
    }
    out
}

/// Inhalt einer Windows-Symboldatei (`.ico`) mit den Größen in `sizes` (jeweils höchstens 255).
pub fn ico_bytes(sizes: &[usize]) -> Vec<u8> {
    let mut images: Vec<Vec<u8>> = Vec::new();
    for &size in sizes {
        let rgba = render_rgba(size);
        let mut dib = Vec::new();
        // BITMAPINFOHEADER: Höhe doppelt (Farbbild plus Maske)
        dib.extend(40u32.to_le_bytes());
        dib.extend((size as i32).to_le_bytes());
        dib.extend((size as i32 * 2).to_le_bytes());
        dib.extend(1u16.to_le_bytes());
        dib.extend(32u16.to_le_bytes());
        dib.extend([0u8; 24]);
        // Zeilen von unten nach oben, Farben als BGRA
        for y in (0..size).rev() {
            for x in 0..size {
                let i = (y * size + x) * 4;
                if let Some(&[r, g, b, a]) = rgba.get(i..i + 4) {
                    dib.extend([b, g, r, a]);
                }
            }
        }
        // Maske (bei 32 Bit unbenutzt, alles 0), Zeilen auf 4 Bytes aufgefüllt
        dib.extend(vec![0u8; size * size.div_ceil(32) * 4]);
        images.push(dib);
    }
    let mut out = vec![0, 0, 1, 0];
    out.extend((sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len();
    for (size, img) in sizes.iter().zip(&images) {
        out.extend([*size as u8, *size as u8, 0, 0]);
        out.extend(1u16.to_le_bytes());
        out.extend(32u16.to_le_bytes());
        out.extend((img.len() as u32).to_le_bytes());
        out.extend((offset as u32).to_le_bytes());
        offset += img.len();
    }
    for img in images {
        out.extend(img);
    }
    out
}
