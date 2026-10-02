use super::geometry::add_xz_frame;
use super::{ICE, SNOW, STONE};
use crate::block::Block;
use crate::texture::hash2;
use crate::vec3::Vec3;

/// Terreno nevado, con un hueco rectangular para la laguna helada. La laguna
/// se corre hacia el borde +X de la isla (en vez de quedar en el medio) para
/// poder abrirla en una cascada que caiga al vacio (ver `add_waterfall`).
pub(super) fn add_terrain(blocks: &mut Vec<Block>, pond_x: (f32, f32), pond_z: (f32, f32)) {
    add_xz_frame(
        blocks,
        (-7.0, 7.0),
        (-7.0, 7.0),
        pond_x,
        pond_z,
        (-0.6, 0.0),
        SNOW,
    );
    blocks.push(Block::new(
        Vec3::new(pond_x.0, -0.6, pond_z.0),
        Vec3::new(pond_x.1, 0.0, pond_z.1),
        ICE,
    ));
}

/// Rocas bordeando la laguna, como las de un rio de piedras: solo en los
/// tres lados que no desembocan en la cascada (`add_waterfall` ya se encarga
/// del lado +X, que tiene que quedar libre para el canal de salida).
pub(super) fn add_shore_rocks(blocks: &mut Vec<Block>, pond_x: (f32, f32), pond_z: (f32, f32)) {
    const STEP: f32 = 0.4;
    const MARGIN: f32 = 0.22;
    let mut seed = 0.0f32;

    let mut place = |blocks: &mut Vec<Block>, x: f32, z: f32| {
        let jitter_x = (hash2(seed, 11.0) - 0.5) * 0.2;
        let jitter_z = (hash2(seed, 23.0) - 0.5) * 0.2;
        let size = 0.12 + hash2(seed, 37.0) * 0.14;
        blocks.push(Block::cube(
            Vec3::new(x + jitter_x, size * 0.4, z + jitter_z),
            size,
            STONE,
        ));
        seed += 1.0;
    };

    // Orillas cercana y lejana (no cruzan hacia el canal de salida en +X).
    let mut x = pond_x.0 - MARGIN;
    while x <= pond_x.1 {
        place(blocks, x, pond_z.0 - MARGIN);
        place(blocks, x, pond_z.1 + MARGIN);
        x += STEP;
    }
    // Orilla del lado -X, de punta a punta.
    let mut z = pond_z.0 - MARGIN;
    while z <= pond_z.1 + MARGIN {
        place(blocks, pond_x.0 - MARGIN, z);
        z += STEP;
    }
}

/// Base rocosa que cuelga del bloque de tierra/nieve, angostandose capa por
/// capa hasta una punta: lo que hace que el terreno se lea como una isla
/// flotante y no como una caja. Cada capa se encoge un poco mas que la
/// anterior y se descentra levemente (via `hash2`) para que no se vea un
/// prisma perfecto sino algo mas organico.
pub(super) fn add_floating_base(blocks: &mut Vec<Block>) {
    const LAYERS: usize = 6;
    let mut x = (-7.0f32, 7.0f32);
    let mut z = (-7.0f32, 7.0f32);
    let mut y0 = -0.6f32;

    for layer in 0..LAYERS {
        let shrink = 1.3 + hash2(layer as f32, 1.7) * 0.6;
        let jitter_x = (hash2(layer as f32, 4.2) - 0.5) * 0.5;
        let jitter_z = (hash2(layer as f32, 8.9) - 0.5) * 0.5;
        let width = (x.1 - x.0 - shrink).max(0.3);
        let depth = (z.1 - z.0 - shrink).max(0.3);
        let cx = (x.0 + x.1) * 0.5 + jitter_x;
        let cz = (z.0 + z.1) * 0.5 + jitter_z;
        x = (cx - width * 0.5, cx + width * 0.5);
        z = (cz - depth * 0.5, cz + depth * 0.5);
        let y1 = y0;
        let y0_next = y1 - 0.5;

        blocks.push(Block::new(
            Vec3::new(x.0, y0_next, z.0),
            Vec3::new(x.1, y1, z.1),
            STONE,
        ));
        y0 = y0_next;
    }

    // Un puñado de rocas sueltas, flotando cerca de la base, como las de la
    // referencia: no caen a ningun lado, solo decoran el vacio de abajo.
    let debris = [
        Vec3::new(-4.5, -3.4, -3.0),
        Vec3::new(4.0, -3.8, 4.2),
        Vec3::new(-2.0, -4.3, 4.5),
        Vec3::new(3.3, -4.6, -4.0),
        Vec3::new(-5.2, -4.7, 1.5),
    ];
    for (index, &center) in debris.iter().enumerate() {
        let size = 0.3 + hash2(index as f32, 2.5) * 0.35;
        blocks.push(Block::cube(center, size, STONE));
    }
}

/// La laguna se desborda por el borde +X de la isla y cae como cascada,
/// angostandose y alejandose del borde a medida que baja, hasta perderse en
/// el vacio. Todos los bloques quedan marcados `flowing` para que
/// `render::shade_hit` anime el desplazamiento de la textura con el tiempo.
pub(super) fn add_waterfall(blocks: &mut Vec<Block>, pond_x: (f32, f32), pond_z: (f32, f32)) {
    const ISLAND_EDGE: f32 = 7.0;

    // Canal corto que conecta el borde de la laguna con el borde de la isla.
    blocks.push(
        Block::new(
            Vec3::new(pond_x.1, -0.6, pond_z.0),
            Vec3::new(ISLAND_EDGE, 0.0, pond_z.1),
            ICE,
        )
        .flowing(true),
    );

    // Caida: capas que bajan, se angostan y se alejan del borde (el agua se
    // dispersa en el aire antes de perderse de vista).
    const SEGMENTS: usize = 10;
    let mut z = pond_z;
    let mut top = -0.6f32;
    let mut outward = 0.15f32;
    for segment in 0..SEGMENTS {
        let height = 1.1;
        let bottom = top - height;
        let shrink = 0.18 + hash2(segment as f32, 6.1) * 0.08;
        let center_z = (z.0 + z.1) * 0.5;
        let half_depth = ((z.1 - z.0) * 0.5 - shrink).max(0.08);
        z = (center_z - half_depth, center_z + half_depth);

        blocks.push(
            Block::new(
                Vec3::new(ISLAND_EDGE + outward, bottom, z.0),
                Vec3::new(ISLAND_EDGE + outward + 0.35, top, z.1),
                ICE,
            )
            .flowing(true),
        );

        top = bottom;
        outward += 0.22;
    }
}
