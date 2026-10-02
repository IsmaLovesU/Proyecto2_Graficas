use crate::block::Block;
use crate::vec3::Vec3;

/// Agrega un "marco" de 4 bloques que cubre `outer` menos el hueco `inner`
/// (ambos en el plano X-Z), a una altura `y0..y1`. Sirve tanto para el hueco
/// de la laguna en el suelo como para el hueco de una ventana en una pared,
/// evitando que dos superficies queden exactamente coplanares (eso produce
/// parpadeo, porque el renderer no tiene forma de decidir cual gana).
#[allow(clippy::too_many_arguments)]
pub(super) fn add_xz_frame(
    blocks: &mut Vec<Block>,
    outer_x: (f32, f32),
    outer_z: (f32, f32),
    inner_x: (f32, f32),
    inner_z: (f32, f32),
    y: (f32, f32),
    material_id: usize,
) {
    let (ox0, ox1) = outer_x;
    let (oz0, oz1) = outer_z;
    let (ix0, ix1) = inner_x;
    let (iz0, iz1) = inner_z;

    // Franjas que cubren todo el ancho en X (por delante y detras del hueco).
    blocks.push(Block::new(
        Vec3::new(ox0, y.0, oz0),
        Vec3::new(ox1, y.1, iz0),
        material_id,
    ));
    blocks.push(Block::new(
        Vec3::new(ox0, y.0, iz1),
        Vec3::new(ox1, y.1, oz1),
        material_id,
    ));
    // Franjas que solo cubren el alto entre iz0..iz1, a los lados del hueco.
    blocks.push(Block::new(
        Vec3::new(ox0, y.0, iz0),
        Vec3::new(ix0, y.1, iz1),
        material_id,
    ));
    blocks.push(Block::new(
        Vec3::new(ix1, y.0, iz0),
        Vec3::new(ox1, y.1, iz1),
        material_id,
    ));
}

/// Igual que `add_xz_frame` pero para un hueco de ventana en una pared que
/// corre a lo largo de Z con un espesor fijo en X (paredes izquierda/derecha
/// de la cabana).
pub(super) fn add_window_wall(
    blocks: &mut Vec<Block>,
    x: (f32, f32),
    wall_z: (f32, f32),
    wall_y: (f32, f32),
    window_z: (f32, f32),
    window_y: (f32, f32),
    material_id: usize,
) {
    let (x0, x1) = x;
    blocks.push(Block::new(
        Vec3::new(x0, wall_y.0, wall_z.0),
        Vec3::new(x1, wall_y.1, window_z.0),
        material_id,
    ));
    blocks.push(Block::new(
        Vec3::new(x0, wall_y.0, window_z.1),
        Vec3::new(x1, wall_y.1, wall_z.1),
        material_id,
    ));
    blocks.push(Block::new(
        Vec3::new(x0, wall_y.0, window_z.0),
        Vec3::new(x1, window_y.0, window_z.1),
        material_id,
    ));
    blocks.push(Block::new(
        Vec3::new(x0, window_y.1, window_z.0),
        Vec3::new(x1, wall_y.1, window_z.1),
        material_id,
    ));
}
