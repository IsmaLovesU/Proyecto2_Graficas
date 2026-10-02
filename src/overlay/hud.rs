use super::blend::blend_pixel;

const GLYPH_ROWS: usize = 5;
const GLYPH_COLS: usize = 3;

/// Fuente de 3x5 bits (solo mayusculas + espacio, lo unico que necesita el
/// HUD de controles) para no depender de ninguna libreria de texto. Cada
/// fila usa los 3 bits bajos como columnas, de izquierda a derecha.
fn glyph(ch: char) -> [u8; GLYPH_ROWS] {
    match ch {
        'A' => [2, 5, 7, 5, 5],
        'B' => [6, 5, 6, 5, 6],
        'C' => [3, 4, 4, 4, 3],
        'D' => [6, 5, 5, 5, 6],
        'E' => [7, 4, 7, 4, 7],
        'F' => [7, 4, 7, 4, 4],
        'H' => [5, 5, 7, 5, 5],
        'I' => [7, 2, 2, 2, 7],
        'L' => [4, 4, 4, 4, 7],
        'M' => [5, 7, 5, 5, 5],
        'N' => [5, 7, 7, 5, 5],
        'O' => [2, 5, 5, 5, 2],
        'P' => [6, 5, 6, 4, 4],
        'R' => [6, 5, 6, 5, 5],
        'S' => [3, 4, 2, 1, 6],
        'T' => [7, 2, 2, 2, 2],
        'U' => [5, 5, 5, 5, 2],
        'W' => [5, 5, 5, 7, 5],
        'Z' => [7, 1, 2, 4, 7],
        _ => [0, 0, 0, 0, 0],
    }
}

const HUD_SCALE: usize = 2;
const HUD_MARGIN: usize = 6;
const HUD_CHAR_ADVANCE: usize = (GLYPH_COLS + 1) * HUD_SCALE;
const HUD_LINE_HEIGHT: usize = (GLYPH_ROWS + 2) * HUD_SCALE;

/// Dibuja las lineas de ayuda en la esquina inferior izquierda, sobre un
/// panel semitransparente para que se lean encima de cualquier fondo.
pub fn draw_hud(buffer: &mut [u32], width: usize, height: usize, lines: &[&str]) {
    let longest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let panel_width = longest * HUD_CHAR_ADVANCE + HUD_MARGIN;
    let panel_height = lines.len() * HUD_LINE_HEIGHT + HUD_MARGIN / 2;
    let panel_left = HUD_MARGIN;
    let panel_top = height.saturating_sub(panel_height + HUD_MARGIN);

    for py in panel_top..(panel_top + panel_height).min(height) {
        for px in panel_left..(panel_left + panel_width).min(width) {
            blend_pixel(buffer, py * width + px, (8.0, 9.0, 14.0), 0.55);
        }
    }

    for (line_index, line) in lines.iter().enumerate() {
        let line_top = panel_top + HUD_MARGIN / 2 + line_index * HUD_LINE_HEIGHT;
        for (char_index, ch) in line.chars().enumerate() {
            let glyph_rows = glyph(ch.to_ascii_uppercase());
            let char_left = panel_left + HUD_MARGIN / 2 + char_index * HUD_CHAR_ADVANCE;
            draw_glyph(buffer, width, height, (char_left, line_top), &glyph_rows);
        }
    }
}

fn draw_glyph(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    origin: (usize, usize),
    rows: &[u8; GLYPH_ROWS],
) {
    let (left, top) = origin;
    for (row, bits) in rows.iter().enumerate() {
        for col in 0..GLYPH_COLS {
            if bits & (1 << (GLYPH_COLS - 1 - col)) == 0 {
                continue;
            }
            let px0 = left + col * HUD_SCALE;
            let py0 = top + row * HUD_SCALE;
            for dy in 0..HUD_SCALE {
                for dx in 0..HUD_SCALE {
                    let px = px0 + dx;
                    let py = py0 + dy;
                    if px < width && py < height {
                        blend_pixel(buffer, py * width + px, (232.0, 236.0, 244.0), 0.95);
                    }
                }
            }
        }
    }
}
