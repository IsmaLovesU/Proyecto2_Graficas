use super::blend::blend_disc;
use crate::texture::hash2;

struct Flake {
    base_x: f32,
    y: f32,
    speed: f32,
    amplitude: f32,
    frequency: f32,
    phase: f32,
    radius: f32,
    wraps: f32,
}

/// Nieve cayendo en pantalla, independiente de la escena 3D: nunca deja de
/// nevar, sin importar si la camara se mueve o la hora del dia cambia.
pub struct Snowfall {
    flakes: Vec<Flake>,
}

const FLAKE_COUNT: usize = 160;

impl Snowfall {
    pub fn new(width: usize, height: usize) -> Self {
        let w = width as f32;
        let h = height as f32;
        let flakes = (0..FLAKE_COUNT)
            .map(|i| spawn_flake(i, w, h, 0.0, true))
            .collect();
        Self { flakes }
    }

    pub fn update(&mut self, dt: f32, width: usize, height: usize) {
        let w = width as f32;
        let h = height as f32;
        for (i, flake) in self.flakes.iter_mut().enumerate() {
            flake.y += flake.speed * dt;
            if flake.y - flake.radius > h {
                flake.wraps += 1.0;
                *flake = spawn_flake(i, w, h, flake.wraps, false);
            }
        }
    }

    pub fn draw(&self, buffer: &mut [u32], width: usize, height: usize, elapsed: f32) {
        for flake in &self.flakes {
            let x =
                flake.base_x + (elapsed * flake.frequency + flake.phase).sin() * flake.amplitude;
            blend_disc(
                buffer,
                width,
                height,
                (x, flake.y, flake.radius),
                (235.0, 242.0, 252.0),
                0.9,
            );
        }
    }
}

/// Nace un copo nuevo. `spread_initial` solo se usa al arrancar, para que el
/// primer frame ya muestre nieve repartida por toda la pantalla en vez de
/// que todos los copos empiecen pegados al borde superior.
fn spawn_flake(index: usize, width: f32, height: f32, seed: f32, spread_initial: bool) -> Flake {
    let fi = index as f32;
    let base_x = hash2(fi, seed + 0.3) * width;
    let y = if spread_initial {
        hash2(fi, seed + 0.7) * (height * 2.0) - height
    } else {
        -(4.0 + hash2(fi, seed + 0.7) * 40.0)
    };
    let speed = 35.0 + hash2(fi, seed + 1.1) * 55.0;
    let amplitude = 5.0 + hash2(fi, seed + 2.2) * 14.0;
    let frequency = 0.4 + hash2(fi, seed + 3.3) * 0.9;
    let phase = hash2(fi, seed + 4.4) * std::f32::consts::TAU;
    let radius = 1.0 + hash2(fi, seed + 5.5) * 1.6;
    Flake {
        base_x,
        y,
        speed,
        amplitude,
        frequency,
        phase,
        radius,
        wraps: seed,
    }
}
