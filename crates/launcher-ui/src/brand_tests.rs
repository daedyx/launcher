use crate::brand::{ico_bytes, mark_strokes, render_rgba};

#[test]
fn zeichnung_bleibt_im_rahmen_und_hat_d_und_irrgartenlinie() {
    let strokes = mark_strokes();
    assert_eq!(strokes.len(), 2, "D-Umriss und Irrgartenlinie");
    for line in &strokes {
        assert!(line.points.len() >= 8);
        for p in &line.points {
            assert!(
                (4.0..=68.0).contains(&p.0) && (4.0..=68.0).contains(&p.1),
                "{p:?}"
            );
        }
    }
}

fn pixel(rgba: &[u8], size: usize, x: usize, y: usize) -> [u8; 4] {
    let i = (y * size + x) * 4;
    [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]]
}

#[test]
fn pixelbild_hat_rundes_quadrat_weisses_d_und_bernstein() {
    let size = 64;
    let img = render_rgba(size);
    assert_eq!(img.len(), size * size * 4);
    // Ecke außerhalb des runden Quadrats ist durchsichtig, die Mitte undurchsichtig
    assert_eq!(pixel(&img, size, 0, 0)[3], 0);
    assert_eq!(pixel(&img, size, size / 2, 8)[3], 255);
    // Es gibt weiße (D-Umriss) und bernsteinfarbene (Linie) Pixel
    let mut white = 0;
    let mut amber = 0;
    for y in 0..size {
        for x in 0..size {
            let [r, g, b, a] = pixel(&img, size, x, y);
            if a == 255 && r > 240 && g > 240 && b > 240 {
                white += 1;
            }
            if a == 255 && r > 200 && (130..190).contains(&g) && b < 110 {
                amber += 1;
            }
        }
    }
    assert!(white > 60, "weiße Pixel: {white}");
    assert!(amber > 30, "bernsteinfarbene Pixel: {amber}");
}

#[test]
fn symboldatei_hat_gueltigen_aufbau() {
    let sizes = [16usize, 32, 48];
    let ico = ico_bytes(&sizes);
    assert_eq!(&ico[0..2], &[0, 0]);
    assert_eq!(&ico[2..4], &[1, 0], "Typ Symbol");
    assert_eq!(u16::from_le_bytes([ico[4], ico[5]]) as usize, sizes.len());
    let mut end = 6 + 16 * sizes.len();
    for (i, size) in sizes.iter().enumerate() {
        let e = 6 + 16 * i;
        assert_eq!(ico[e] as usize, *size);
        let len = u32::from_le_bytes(ico[e + 8..e + 12].try_into().unwrap()) as usize;
        let offset = u32::from_le_bytes(ico[e + 12..e + 16].try_into().unwrap()) as usize;
        assert_eq!(offset, end, "Bild {i} folgt lückenlos");
        // Farbbild (BGRA) plus Maske, Kopf 40 Bytes
        assert_eq!(len, 40 + size * size * 4 + size * size.div_ceil(32) * 4);
        end += len;
    }
    assert_eq!(ico.len(), end);
}
