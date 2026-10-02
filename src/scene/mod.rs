//! La cabana suiza sobre su isla flotante: cada pieza de la escena
//! (terreno/isla, cabana, decoracion) vive en su propio submodulo; este
//! archivo solo los junta en `build_scene`.
mod cabin;
mod decor;
mod geometry;
mod island;

use crate::block::Block;
use crate::light::Light;
use crate::material::Material;
use crate::vec3::Vec3;

pub const WOOD: usize = 0;
pub const STONE: usize = 1;
pub const SNOW: usize = 2;
pub const ICE: usize = 3;
pub const GLASS: usize = 4;
pub const LEAVES: usize = 5;

/// Boca de la chimenea, de donde sale el humo animado (ver `overlay::Smoke`).
pub const CHIMNEY_TOP: Vec3 = Vec3::new(1.875, 3.3, 0.0);

pub struct Scene {
    pub blocks: Vec<Block>,
    pub materials: Vec<Material>,
    pub lights: Vec<Light>,
}

fn build_materials() -> Vec<Material> {
    vec![
        Material::wood(),
        Material::stone(),
        Material::snow(),
        Material::ice(),
        Material::glass(),
        Material::leaves(),
    ]
}

/// Luces fijas de la cabana (ventanas, farol, brasas de la chimenea). El
/// sol/la luna son dinamicos y se calculan por frame en `render`, no aca.
fn build_lights(fx: f32, fz: f32) -> Vec<Light> {
    vec![
        Light::point(
            Vec3::new(-fx + 0.15, 1.25, 0.0),
            2.2,
            Vec3::new(1.0, 0.75, 0.45),
        ),
        Light::point(
            Vec3::new(fx - 0.15, 1.25, 0.0),
            2.2,
            Vec3::new(1.0, 0.75, 0.45),
        ),
        Light::point(
            Vec3::new(0.0, 1.7, -fz - 0.3),
            1.1,
            Vec3::new(1.0, 0.8, 0.5),
        ),
        Light::point(Vec3::new(1.87, 3.3, 0.0), 0.9, Vec3::new(1.0, 0.55, 0.25)),
    ]
}

pub fn build_scene() -> Scene {
    let mut blocks = Vec::new();

    // La laguna se corre hacia el borde +X de la isla (en vez de quedar en
    // el medio) para poder abrirla en una cascada que caiga al vacio.
    const POND_X: (f32, f32) = (3.6, 6.3);
    const POND_Z: (f32, f32) = (1.4, 3.6);
    island::add_terrain(&mut blocks, POND_X, POND_Z);
    island::add_floating_base(&mut blocks);
    island::add_waterfall(&mut blocks, POND_X, POND_Z);

    let cabin = cabin::add_cabin(&mut blocks);
    decor::add_path(&mut blocks, cabin.fz);
    decor::add_snowman(&mut blocks);
    decor::add_trees(&mut blocks);

    Scene {
        blocks,
        materials: build_materials(),
        lights: build_lights(cabin.fx, cabin.fz),
    }
}
