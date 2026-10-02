use super::{GLASS, LEAVES, SNOW, STONE, WOOD};
use crate::block::Block;
use crate::vec3::Vec3;

fn add_pine_tree(blocks: &mut Vec<Block>, base: Vec3) {
    blocks.push(Block::centered(
        base + Vec3::new(0.0, 0.25, 0.0),
        Vec3::new(0.09, 0.25, 0.09),
        WOOD,
    ));
    blocks.push(Block::cube(base + Vec3::new(0.0, 0.65, 0.0), 0.62, LEAVES));
    blocks.push(Block::cube(base + Vec3::new(0.0, 1.05, 0.0), 0.46, LEAVES));
    blocks.push(Block::cube(base + Vec3::new(0.0, 1.4, 0.0), 0.3, LEAVES));
    // Un gorrito de nieve sobre la copa: el detalle que vende el "cozy".
    blocks.push(Block::centered(
        base + Vec3::new(0.0, 1.58, 0.0),
        Vec3::new(0.14, 0.05, 0.14),
        SNOW,
    ));
}

pub(super) fn add_trees(blocks: &mut Vec<Block>) {
    add_pine_tree(blocks, Vec3::new(-2.8, 0.0, -1.0));
    add_pine_tree(blocks, Vec3::new(-2.4, 0.0, 1.6));
    add_pine_tree(blocks, Vec3::new(2.9, 0.0, -1.8));
}

/// Camino de piedra que sigue desde el borde del deck (`deck_edge`) hacia
/// afuera, continuando el recorrido de madera en piedra.
pub(super) fn add_path(blocks: &mut Vec<Block>, deck_edge: f32) {
    for step in 0..3 {
        let z = deck_edge - 0.35 - step as f32 * 0.5;
        blocks.push(Block::new(
            Vec3::new(-0.25, -0.02, z - 0.2),
            Vec3::new(0.25, 0.02, z + 0.2),
            STONE,
        ));
    }
}

pub(super) fn add_snowman(blocks: &mut Vec<Block>) {
    let base = Vec3::new(0.9, 0.0, -2.9);
    blocks.push(Block::cube(base + Vec3::new(0.0, 0.22, 0.0), 0.42, SNOW));
    blocks.push(Block::cube(base + Vec3::new(0.0, 0.55, 0.0), 0.3, SNOW));
    blocks.push(Block::cube(base + Vec3::new(0.0, 0.78, 0.0), 0.2, SNOW));
    blocks.push(Block::cube(base + Vec3::new(0.11, 0.55, 0.14), 0.05, STONE));
    blocks.push(Block::cube(
        base + Vec3::new(-0.11, 0.55, 0.14),
        0.05,
        STONE,
    ));
}

/// Casita de perro en miniatura, con el mismo lenguaje que la cabana grande
/// (paredes de madera, hueco de puerta, techito con nieve encima).
pub(super) fn add_doghouse(blocks: &mut Vec<Block>, base: Vec3) {
    const HALF_X: f32 = 0.27;
    const HALF_Z: f32 = 0.22;
    const WALL_TOP: f32 = 0.3;
    const THICK: f32 = 0.045;
    const DOOR_HALF: f32 = 0.09;
    const DOOR_TOP: f32 = 0.22;

    // Pared trasera, solida.
    blocks.push(Block::new(
        base + Vec3::new(-HALF_X, 0.0, HALF_Z - THICK),
        base + Vec3::new(HALF_X, WALL_TOP, HALF_Z),
        WOOD,
    ));
    // Paredes laterales.
    blocks.push(Block::new(
        base + Vec3::new(-HALF_X, 0.0, -HALF_Z),
        base + Vec3::new(-HALF_X + THICK, WALL_TOP, HALF_Z),
        WOOD,
    ));
    blocks.push(Block::new(
        base + Vec3::new(HALF_X - THICK, 0.0, -HALF_Z),
        base + Vec3::new(HALF_X, WALL_TOP, HALF_Z),
        WOOD,
    ));
    // Pared frontal, con un hueco de puerta cuadrado.
    blocks.push(Block::new(
        base + Vec3::new(-HALF_X, 0.0, -HALF_Z),
        base + Vec3::new(-DOOR_HALF, WALL_TOP, -HALF_Z + THICK),
        WOOD,
    ));
    blocks.push(Block::new(
        base + Vec3::new(DOOR_HALF, 0.0, -HALF_Z),
        base + Vec3::new(HALF_X, WALL_TOP, -HALF_Z + THICK),
        WOOD,
    ));
    blocks.push(Block::new(
        base + Vec3::new(-DOOR_HALF, DOOR_TOP, -HALF_Z),
        base + Vec3::new(DOOR_HALF, WALL_TOP, -HALF_Z + THICK),
        WOOD,
    ));

    // Techito a dos aguas en miniatura, con su propio gorro de nieve.
    blocks.push(Block::new(
        base + Vec3::new(-HALF_X - 0.05, WALL_TOP, -HALF_Z - 0.05),
        base + Vec3::new(HALF_X + 0.05, WALL_TOP + 0.09, HALF_Z + 0.05),
        WOOD,
    ));
    blocks.push(Block::new(
        base + Vec3::new(-HALF_X - 0.08, WALL_TOP + 0.09, -HALF_Z - 0.07),
        base + Vec3::new(HALF_X + 0.08, WALL_TOP + 0.14, HALF_Z + 0.07),
        SNOW,
    ));
}

/// Pila de lena apoyada contra la pared: logs aproximados como prismas
/// cortos, apilados de a dos filas.
pub(super) fn add_woodpile(blocks: &mut Vec<Block>, base: Vec3) {
    const LOG_LENGTH: f32 = 0.5;
    const LOG_RADIUS: f32 = 0.055;
    const ROWS: usize = 2;
    const LOGS_PER_ROW: usize = 4;

    for row in 0..ROWS {
        let y_center = LOG_RADIUS + row as f32 * LOG_RADIUS * 2.05;
        for log in 0..LOGS_PER_ROW {
            let x_offset = (log as f32 - (LOGS_PER_ROW as f32 - 1.0) * 0.5) * (LOG_RADIUS * 2.1);
            blocks.push(Block::centered(
                base + Vec3::new(x_offset, y_center, 0.0),
                Vec3::new(LOG_RADIUS, LOG_RADIUS, LOG_LENGTH * 0.5),
                WOOD,
            ));
        }
    }
}

/// Farol de camino: poste de madera con un techito de piedra y un vidrio
/// que hace de lampara. Devuelve el punto donde debe ir la luz calida.
pub(super) fn add_lamp_post(blocks: &mut Vec<Block>, base: Vec3) -> Vec3 {
    const POST_HEIGHT: f32 = 1.1;
    const POST_RADIUS: f32 = 0.035;

    blocks.push(Block::centered(
        base + Vec3::new(0.0, POST_HEIGHT * 0.5, 0.0),
        Vec3::new(POST_RADIUS, POST_HEIGHT * 0.5, POST_RADIUS),
        WOOD,
    ));
    blocks.push(Block::centered(
        base + Vec3::new(0.0, POST_HEIGHT + 0.03, 0.0),
        Vec3::new(0.09, 0.025, 0.09),
        STONE,
    ));

    let lantern_center = base + Vec3::new(0.0, POST_HEIGHT - 0.08, 0.0);
    blocks.push(Block::cube(lantern_center, 0.13, GLASS));
    lantern_center
}

/// Arbusto redondeado: tres bloques de follaje superpuestos en vez de uno
/// solo, para que la silueta no se vea como un cubo perfecto, con nieve
/// acumulada encima.
fn add_bush(blocks: &mut Vec<Block>, base: Vec3) {
    blocks.push(Block::cube(base + Vec3::new(0.0, 0.14, 0.0), 0.3, LEAVES));
    blocks.push(Block::cube(
        base + Vec3::new(0.08, 0.22, 0.05),
        0.22,
        LEAVES,
    ));
    blocks.push(Block::cube(
        base + Vec3::new(-0.09, 0.2, -0.06),
        0.2,
        LEAVES,
    ));
    blocks.push(Block::centered(
        base + Vec3::new(0.0, 0.34, 0.0),
        Vec3::new(0.16, 0.03, 0.16),
        SNOW,
    ));
}

pub(super) fn add_bushes(blocks: &mut Vec<Block>) {
    add_bush(blocks, Vec3::new(2.8, 0.0, 0.5));
    add_bush(blocks, Vec3::new(-2.0, 0.0, -2.6));
    add_bush(blocks, Vec3::new(-3.0, 0.0, 0.8));
}
