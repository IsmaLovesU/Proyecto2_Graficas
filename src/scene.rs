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

pub struct Scene {
    pub blocks: Vec<Block>,
    pub materials: Vec<Material>,
    pub lights: Vec<Light>,
}

pub fn build_materials() -> Vec<Material> {
    vec![
        Material::wood(),
        Material::stone(),
        Material::snow(),
        Material::ice(),
        Material::glass(),
        Material::leaves(),
    ]
}

/// Agrega un "marco" de 4 bloques que cubre `outer` menos el hueco `inner`
/// (ambos en el plano X-Z), a una altura `y0..y1`. Sirve tanto para el hueco
/// de la laguna en el suelo como para el hueco de una ventana en una pared,
/// evitando que dos superficies queden exactamente coplanares (eso produce
/// parpadeo, porque el renderer no tiene forma de decidir cual gana).
#[allow(clippy::too_many_arguments)]
fn add_xz_frame(
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
fn add_window_wall(
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

pub fn build_scene() -> Scene {
    let mut blocks = Vec::new();

    // --- Terreno nevado, con un hueco rectangular para la laguna helada ---
    add_xz_frame(
        &mut blocks,
        (-7.0, 7.0),
        (-7.0, 7.0),
        (2.5, 4.7),
        (1.4, 3.6),
        (-0.6, 0.0),
        SNOW,
    );
    blocks.push(Block::new(
        Vec3::new(2.5, -0.6, 1.4),
        Vec3::new(4.7, 0.0, 3.6),
        ICE,
    ));

    // --- Cimiento de piedra ---
    const FX: f32 = 1.8;
    const FZ: f32 = 1.3;
    const FOUNDATION_TOP: f32 = 0.35;
    blocks.push(Block::new(
        Vec3::new(-FX, 0.0, -FZ),
        Vec3::new(FX, FOUNDATION_TOP, FZ),
        STONE,
    ));

    // --- Paredes ---
    const WALL_TOP: f32 = 2.0;
    const WALL_THICK: f32 = 0.2;

    // Pared trasera: solida, sin huecos.
    blocks.push(Block::new(
        Vec3::new(-FX, FOUNDATION_TOP, FZ - WALL_THICK),
        Vec3::new(FX, WALL_TOP, FZ),
        WOOD,
    ));

    // Pared frontal, partida por el hueco de la puerta.
    const DOOR_X: (f32, f32) = (-0.3, 0.3);
    const DOOR_TOP: f32 = 1.5;
    blocks.push(Block::new(
        Vec3::new(-FX, FOUNDATION_TOP, -FZ),
        Vec3::new(DOOR_X.0, WALL_TOP, -FZ + WALL_THICK),
        WOOD,
    ));
    blocks.push(Block::new(
        Vec3::new(DOOR_X.1, FOUNDATION_TOP, -FZ),
        Vec3::new(FX, WALL_TOP, -FZ + WALL_THICK),
        WOOD,
    ));
    blocks.push(Block::new(
        Vec3::new(DOOR_X.0, DOOR_TOP, -FZ),
        Vec3::new(DOOR_X.1, WALL_TOP, -FZ + WALL_THICK),
        WOOD,
    ));
    // La puerta misma, al ras de la pared.
    blocks.push(Block::new(
        Vec3::new(DOOR_X.0, FOUNDATION_TOP, -FZ),
        Vec3::new(DOOR_X.1, DOOR_TOP, -FZ + WALL_THICK),
        WOOD,
    ));

    // Paredes laterales, cada una con su ventana.
    const WINDOW_Z: (f32, f32) = (-0.35, 0.35);
    const WINDOW_Y: (f32, f32) = (0.95, 1.55);

    add_window_wall(
        &mut blocks,
        (-FX, -FX + WALL_THICK),
        (-FZ, FZ),
        (FOUNDATION_TOP, WALL_TOP),
        WINDOW_Z,
        WINDOW_Y,
        WOOD,
    );
    add_window_wall(
        &mut blocks,
        (FX - WALL_THICK, FX),
        (-FZ, FZ),
        (FOUNDATION_TOP, WALL_TOP),
        WINDOW_Z,
        WINDOW_Y,
        WOOD,
    );

    // Vidrio de cada ventana, mas la cruz de madera del marco (sobresale un
    // poco de la pared para que se note contra el vidrio).
    for &wall_x in &[-FX, FX] {
        let sign = wall_x.signum();
        blocks.push(Block::new(
            Vec3::new(wall_x, WINDOW_Y.0, WINDOW_Z.0),
            Vec3::new(wall_x + sign * WALL_THICK, WINDOW_Y.1, WINDOW_Z.1),
            GLASS,
        ));
        let mid_y = (WINDOW_Y.0 + WINDOW_Y.1) * 0.5;
        blocks.push(Block::centered(
            Vec3::new(wall_x, mid_y, 0.0),
            Vec3::new(WALL_THICK * 0.5 + 0.02, mid_y - WINDOW_Y.0, 0.02),
            WOOD,
        ));
        blocks.push(Block::centered(
            Vec3::new(wall_x, mid_y, 0.0),
            Vec3::new(
                WALL_THICK * 0.5 + 0.02,
                0.02,
                (WINDOW_Z.1 - WINDOW_Z.0) * 0.5,
            ),
            WOOD,
        ));
    }

    // --- Techo escalonado con nieve encima, estilo chalet suizo ---
    const ROOF_1: ((f32, f32), (f32, f32), (f32, f32)) = ((-2.2, 2.2), (-1.7, 1.7), (2.0, 2.25));
    const ROOF_2: ((f32, f32), (f32, f32), (f32, f32)) = ((-1.6, 1.6), (-1.2, 1.2), (2.25, 2.55));
    const ROOF_3: ((f32, f32), (f32, f32), (f32, f32)) = ((-0.9, 0.9), (-0.7, 0.7), (2.55, 2.85));
    const SNOW_CAP: f32 = 0.07;

    for &(x, z, y) in &[ROOF_1, ROOF_2, ROOF_3] {
        blocks.push(Block::new(
            Vec3::new(x.0, y.0, z.0),
            Vec3::new(x.1, y.1, z.1),
            WOOD,
        ));
    }
    add_xz_frame(
        &mut blocks,
        ROOF_1.0,
        ROOF_1.1,
        ROOF_2.0,
        ROOF_2.1,
        (ROOF_1.2 .1, ROOF_1.2 .1 + SNOW_CAP),
        SNOW,
    );
    add_xz_frame(
        &mut blocks,
        ROOF_2.0,
        ROOF_2.1,
        ROOF_3.0,
        ROOF_3.1,
        (ROOF_2.2 .1, ROOF_2.2 .1 + SNOW_CAP),
        SNOW,
    );
    blocks.push(Block::new(
        Vec3::new(ROOF_3.0 .0, ROOF_3.2 .1, ROOF_3.1 .0),
        Vec3::new(ROOF_3.0 .1, ROOF_3.2 .1 + SNOW_CAP, ROOF_3.1 .1),
        SNOW,
    ));

    // --- Chimenea, saliendo del primer escalon del techo ---
    blocks.push(Block::new(
        Vec3::new(1.7, 2.05, -0.15),
        Vec3::new(2.05, 3.3, 0.15),
        STONE,
    ));

    // --- Camino de piedra hasta la puerta ---
    for step in 0..3 {
        let z = -FZ - 0.4 - step as f32 * 0.5;
        blocks.push(Block::new(
            Vec3::new(-0.25, -0.02, z - 0.2),
            Vec3::new(0.25, 0.02, z + 0.2),
            STONE,
        ));
    }

    // --- Munequito de nieve junto al camino ---
    let snowman_base = Vec3::new(0.9, 0.0, -2.6);
    blocks.push(Block::cube(
        snowman_base + Vec3::new(0.0, 0.22, 0.0),
        0.42,
        SNOW,
    ));
    blocks.push(Block::cube(
        snowman_base + Vec3::new(0.0, 0.55, 0.0),
        0.3,
        SNOW,
    ));
    blocks.push(Block::cube(
        snowman_base + Vec3::new(0.0, 0.78, 0.0),
        0.2,
        SNOW,
    ));
    blocks.push(Block::cube(
        snowman_base + Vec3::new(0.11, 0.55, 0.14),
        0.05,
        STONE,
    ));
    blocks.push(Block::cube(
        snowman_base + Vec3::new(-0.11, 0.55, 0.14),
        0.05,
        STONE,
    ));

    // --- Pinos alrededor ---
    add_pine_tree(&mut blocks, Vec3::new(-2.8, 0.0, -1.0));
    add_pine_tree(&mut blocks, Vec3::new(-2.4, 0.0, 1.6));
    add_pine_tree(&mut blocks, Vec3::new(2.9, 0.0, -1.8));

    // --- Luces: luna fria arriba, una de relleno para que el lado en sombra
    // no se pierda en negro puro, ventanas y farol calidos, y brasas en la
    // chimenea ---
    let lights = vec![
        Light::directional(Vec3::new(0.35, 0.55, -0.4), 0.85, Vec3::new(0.6, 0.65, 0.9)),
        Light::directional(Vec3::new(-0.4, 0.3, 0.5), 0.3, Vec3::new(0.5, 0.55, 0.75)),
        Light::point(
            Vec3::new(-FX + 0.15, 1.25, 0.0),
            2.2,
            Vec3::new(1.0, 0.75, 0.45),
        ),
        Light::point(
            Vec3::new(FX - 0.15, 1.25, 0.0),
            2.2,
            Vec3::new(1.0, 0.75, 0.45),
        ),
        Light::point(
            Vec3::new(0.0, 1.7, -FZ - 0.3),
            1.1,
            Vec3::new(1.0, 0.8, 0.5),
        ),
        Light::point(Vec3::new(1.87, 3.3, 0.0), 0.9, Vec3::new(1.0, 0.55, 0.25)),
    ];

    Scene {
        blocks,
        materials: build_materials(),
        lights,
    }
}
