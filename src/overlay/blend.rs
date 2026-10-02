use crate::framebuffer::rgb;

/// Mezcla `color` sobre el pixel existente del framebuffer segun `alpha`
/// (0 = no cambia nada, 1 = lo reemplaza del todo). Es el unico punto donde
/// el overlay toca el buffer, asi que nieve, humo y HUD comparten la misma
/// aritmetica de mezcla.
pub(super) fn blend_pixel(buffer: &mut [u32], index: usize, color: (f32, f32, f32), alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    let alpha = alpha.min(1.0);
    let existing = buffer[index];
    let er = ((existing >> 16) & 0xff) as f32;
    let eg = ((existing >> 8) & 0xff) as f32;
    let eb = (existing & 0xff) as f32;
    let r = (er * (1.0 - alpha) + color.0 * alpha).clamp(0.0, 255.0) as u8;
    let g = (eg * (1.0 - alpha) + color.1 * alpha).clamp(0.0, 255.0) as u8;
    let b = (eb * (1.0 - alpha) + color.2 * alpha).clamp(0.0, 255.0) as u8;
    buffer[index] = rgb(r, g, b);
}

/// `circle` es `(x, y, radius)` en pixeles: agrupar esos tres juntos evita
/// que la firma de la funcion crezca mas de lo que clippy deja pasar. Dibuja
/// un disco suave (el borde se desvanece) en vez de un circulo con bordes
/// duros, para que un copo de nieve o un puff de humo no se vean como un
/// cuadrado de pixeles.
pub(super) fn blend_disc(
    buffer: &mut [u32],
    width: usize,
    height: usize,
    circle: (f32, f32, f32),
    color: (f32, f32, f32),
    strength: f32,
) {
    let (x, y, radius) = circle;
    if radius <= 0.0 || strength <= 0.0 {
        return;
    }
    let x0 = (x - radius).floor().max(0.0) as i32;
    let x1 = (x + radius).ceil().min(width as f32 - 1.0) as i32;
    let y0 = (y - radius).floor().max(0.0) as i32;
    let y1 = (y + radius).ceil().min(height as f32 - 1.0) as i32;

    for py in y0..=y1 {
        for px in x0..=x1 {
            let dx = px as f32 + 0.5 - x;
            let dy = py as f32 + 0.5 - y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > radius {
                continue;
            }
            let falloff = 1.0 - dist / radius;
            let index = py as usize * width + px as usize;
            blend_pixel(buffer, index, color, falloff * strength);
        }
    }
}
