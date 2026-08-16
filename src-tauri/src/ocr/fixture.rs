//! Deterministic high-contrast "HELLO" PNG used as the OCR fixture.

const GLYPH_W: usize = 5;
const GLYPH_H: usize = 7;
const SCALE: usize = 12;
const PAD: usize = 24;
const GAP: usize = 8;

const H: [&str; 7] = [
    "#   #", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
];
const E: [&str; 7] = [
    "#####", "#    ", "#    ", "#### ", "#    ", "#    ", "#####",
];
const L: [&str; 7] = [
    "#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####",
];
const O: [&str; 7] = [
    " ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
];

/// Known substring the fixture spells. Tests assert OCR contains this.
pub const FIXTURE_SUBSTRING: &str = "HELLO";

/// Rasterize a black-on-white HELLO bitmap as PNG bytes.
pub fn hello_png_bytes() -> Vec<u8> {
    let glyphs = [H, E, L, L, O];
    let inner_w = glyphs.len() * GLYPH_W * SCALE + (glyphs.len() - 1) * GAP;
    let inner_h = GLYPH_H * SCALE;
    let width = (inner_w + PAD * 2) as u32;
    let height = (inner_h + PAD * 2) as u32;
    let mut img = image::RgbaImage::from_pixel(width, height, image::Rgba([255, 255, 255, 255]));
    for (gi, glyph) in glyphs.iter().enumerate() {
        let ox = PAD + gi * (GLYPH_W * SCALE + GAP);
        for (row, line) in glyph.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                if ch != '#' {
                    continue;
                }
                for dy in 0..SCALE {
                    for dx in 0..SCALE {
                        img.put_pixel(
                            (ox + col * SCALE + dx) as u32,
                            (PAD + row * SCALE + dy) as u32,
                            image::Rgba([0, 0, 0, 255]),
                        );
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .expect("encode fixture png");
    out
}

/// Write the fixture next to the given path (parent dirs created).
pub fn write_hello_png(path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, hello_png_bytes())
}
