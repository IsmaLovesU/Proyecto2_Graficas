use super::geometry::{add_window_wall, add_xz_frame};
use super::porch;
use super::{GLASS, SNOW, STONE, WOOD};
use crate::block::Block;
use crate::vec3::Vec3;

/// Medidas de la cabana que otras piezas de la escena necesitan para
/// ubicarse relativas a ella (el camino de piedra, el deck y las luces).
pub(super) struct CabinLayout {
    pub fx: f32,
    pub fz: f32,
    /// Borde mas alejado del deck (el punto donde deberia seguir el camino).
    pub deck_edge: f32,
}

/// Cimiento, paredes con puerta y ventanas, techo escalonado con nieve y la
/// chimenea: todo el cuerpo del chalet suizo.
pub(super) fn add_cabin(blocks: &mut Vec<Block>) -> CabinLayout {
    const FX: f32 = 1.8;
    const FZ: f32 = 1.3;
    const FOUNDATION_TOP: f32 = 0.35;
    blocks.push(Block::new(
        Vec3::new(-FX, 0.0, -FZ),
        Vec3::new(FX, FOUNDATION_TOP, FZ),
        STONE,
    ));

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
        blocks,
        (-FX, -FX + WALL_THICK),
        (-FZ, FZ),
        (FOUNDATION_TOP, WALL_TOP),
        WINDOW_Z,
        WINDOW_Y,
        WOOD,
    );
    add_window_wall(
        blocks,
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

    porch::add_window_dressing(blocks, FX, WALL_THICK, WINDOW_Z, WINDOW_Y);
    porch::add_porch(blocks, FZ, DOOR_X);
    let deck_edge = porch::add_deck(blocks, FZ);

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
        blocks,
        ROOF_1.0,
        ROOF_1.1,
        ROOF_2.0,
        ROOF_2.1,
        (ROOF_1.2 .1, ROOF_1.2 .1 + SNOW_CAP),
        SNOW,
    );
    add_xz_frame(
        blocks,
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

    // --- Chimenea, saliendo del primer escalon del techo (su boca es
    // `super::CHIMNEY_TOP`, de donde sale el humo animado) ---
    blocks.push(Block::new(
        Vec3::new(1.7, 2.05, -0.15),
        Vec3::new(2.05, 3.3, 0.15),
        STONE,
    ));

    CabinLayout {
        fx: FX,
        fz: FZ,
        deck_edge,
    }
}
