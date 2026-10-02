use super::{LEAVES, SNOW, STONE, WOOD};
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

/// Camino de piedra desde la puerta (en `-fz`) hacia afuera.
pub(super) fn add_path(blocks: &mut Vec<Block>, fz: f32) {
    for step in 0..3 {
        let z = -fz - 0.4 - step as f32 * 0.5;
        blocks.push(Block::new(
            Vec3::new(-0.25, -0.02, z - 0.2),
            Vec3::new(0.25, 0.02, z + 0.2),
            STONE,
        ));
    }
}

pub(super) fn add_snowman(blocks: &mut Vec<Block>) {
    let base = Vec3::new(0.9, 0.0, -2.6);
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
