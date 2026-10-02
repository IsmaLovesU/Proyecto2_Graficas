use super::{SNOW, WOOD};
use crate::block::Block;
use crate::vec3::Vec3;

/// Contraventanas a los lados de cada ventana y una jardinera con nieve
/// encima bajo el alfeizar: el detalle que hace que la cabana se sienta
/// habitada en vez de una caja con huecos.
pub(super) fn add_window_dressing(
    blocks: &mut Vec<Block>,
    fx: f32,
    wall_thick: f32,
    window_z: (f32, f32),
    window_y: (f32, f32),
) {
    let shutter_depth = wall_thick * 0.5 + 0.03;

    for &wall_x in &[-fx, fx] {
        let sign = wall_x.signum();
        let outer_x = wall_x + sign * shutter_depth;
        let (x0, x1) = (wall_x.min(outer_x), wall_x.max(outer_x));

        for &(z0, z1) in &[
            (window_z.0 - 0.16, window_z.0 - 0.01),
            (window_z.1 + 0.01, window_z.1 + 0.16),
        ] {
            blocks.push(Block::new(
                Vec3::new(x0, window_y.0, z0),
                Vec3::new(x1, window_y.1, z1),
                WOOD,
            ));
        }

        let box_bottom = window_y.0 - 0.16;
        let box_top = window_y.0 - 0.03;
        blocks.push(Block::new(
            Vec3::new(x0, box_bottom, window_z.0 - 0.05),
            Vec3::new(x1, box_top, window_z.1 + 0.05),
            WOOD,
        ));
        blocks.push(Block::new(
            Vec3::new(x0, box_top, window_z.0 - 0.05),
            Vec3::new(x1, box_top + 0.025, window_z.1 + 0.05),
            SNOW,
        ));
    }
}

/// Alero chiquito sobre la puerta con dos postes: convierte la entrada en
/// un porche en vez de una puerta pelada contra la pared. Cubre solo el
/// primer tramo del deck (ver `add_deck`), como en una cabana de verdad.
pub(super) fn add_porch(blocks: &mut Vec<Block>, fz: f32, door_x: (f32, f32)) {
    const PORCH_HALF_WIDTH: f32 = 0.5;
    const ROOF_Y: (f32, f32) = (1.68, 1.82);
    let front_z = -fz - 0.6;
    let wall_z = -fz + 0.05;

    blocks.push(Block::new(
        Vec3::new(-PORCH_HALF_WIDTH - 0.08, ROOF_Y.0, front_z),
        Vec3::new(PORCH_HALF_WIDTH + 0.08, ROOF_Y.1, wall_z),
        WOOD,
    ));
    blocks.push(Block::new(
        Vec3::new(-PORCH_HALF_WIDTH - 0.12, ROOF_Y.1, front_z - 0.04),
        Vec3::new(PORCH_HALF_WIDTH + 0.12, ROOF_Y.1 + 0.05, wall_z + 0.03),
        SNOW,
    ));

    let door_mid = (door_x.0 + door_x.1) * 0.5;
    let post_offset = PORCH_HALF_WIDTH - 0.06;
    for &post_x in &[door_mid - post_offset, door_mid + post_offset] {
        blocks.push(Block::new(
            Vec3::new(post_x - 0.05, 0.0, front_z + 0.05),
            Vec3::new(post_x + 0.05, ROOF_Y.0, front_z + 0.15),
            WOOD,
        ));
    }
}

const DECK_HALF_WIDTH: f32 = 0.9;
const DECK_DEPTH: f32 = 1.1;
const DECK_FLOOR_Y: (f32, f32) = (0.0, 0.07);
const RAIL_Y: (f32, f32) = (0.36, 0.44);
const POST_SPACING: f32 = 0.3;

/// Plataforma de madera que se extiende desde la puerta, con una baranda de
/// balaustres en el borde exterior y en los dos laterales (abierta contra
/// la pared, donde esta la puerta) — el deck de la referencia. Devuelve el
/// borde mas alejado de la pared, para que el camino de piedra siga desde
/// ahi en vez de aparecer debajo del piso del deck.
pub(super) fn add_deck(blocks: &mut Vec<Block>, fz: f32) -> f32 {
    let outer_z = -fz - DECK_DEPTH;
    let wall_z = -fz + 0.05;

    blocks.push(Block::new(
        Vec3::new(-DECK_HALF_WIDTH, DECK_FLOOR_Y.0, outer_z),
        Vec3::new(DECK_HALF_WIDTH, DECK_FLOOR_Y.1, wall_z),
        WOOD,
    ));

    // Pasamanos: borde frontal mas los dos laterales.
    blocks.push(Block::new(
        Vec3::new(-DECK_HALF_WIDTH, RAIL_Y.0, outer_z),
        Vec3::new(DECK_HALF_WIDTH, RAIL_Y.1, outer_z + 0.05),
        WOOD,
    ));
    blocks.push(Block::new(
        Vec3::new(-DECK_HALF_WIDTH, RAIL_Y.0, outer_z),
        Vec3::new(-DECK_HALF_WIDTH + 0.05, RAIL_Y.1, -fz),
        WOOD,
    ));
    blocks.push(Block::new(
        Vec3::new(DECK_HALF_WIDTH - 0.05, RAIL_Y.0, outer_z),
        Vec3::new(DECK_HALF_WIDTH, RAIL_Y.1, -fz),
        WOOD,
    ));

    add_rail_posts(blocks, (-DECK_HALF_WIDTH, DECK_HALF_WIDTH), outer_z, true);
    add_rail_posts(blocks, (outer_z, -fz), -DECK_HALF_WIDTH, false);
    add_rail_posts(blocks, (outer_z, -fz), DECK_HALF_WIDTH, false);

    outer_z
}

/// Balaustres cada `POST_SPACING` a lo largo de un borde del deck.
/// `along_x` decide si `range` recorre X (borde frontal) o Z (laterales).
fn add_rail_posts(blocks: &mut Vec<Block>, range: (f32, f32), fixed: f32, along_x: bool) {
    let span = range.1 - range.0;
    let count = (span / POST_SPACING).round().max(1.0) as usize;
    for i in 0..=count {
        let t = range.0 + span * (i as f32 / count as f32);
        let (x, z) = if along_x { (t, fixed) } else { (fixed, t) };
        blocks.push(Block::centered(
            Vec3::new(x, RAIL_Y.1 * 0.5, z),
            Vec3::new(0.025, RAIL_Y.1 * 0.5, 0.025),
            WOOD,
        ));
    }
}
