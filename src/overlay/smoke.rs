use super::blend::blend_disc;
use super::camera::Camera;
use crate::texture::hash2;
use crate::vec3::Vec3;

struct Puff {
    spawned_at: f32,
    lifetime: f32,
    seed: f32,
}

/// Humo que sale de la chimenea: cada puff nace en la boca de la chimenea,
/// sube y se dispersa en el aire, y se desvanece antes de reciclarse.
pub struct Smoke {
    anchor: Vec3,
    puffs: Vec<Puff>,
}

const PUFF_COUNT: usize = 14;
const PUFF_SPACING: f32 = 0.35;

impl Smoke {
    pub fn new(anchor: Vec3) -> Self {
        let puffs = (0..PUFF_COUNT)
            .map(|i| Puff {
                spawned_at: -(i as f32) * PUFF_SPACING,
                lifetime: 2.6 + hash2(i as f32, 9.1) * 1.4,
                seed: i as f32,
            })
            .collect();
        Self { anchor, puffs }
    }

    pub fn update(&mut self, elapsed: f32) {
        for puff in &mut self.puffs {
            let age = elapsed - puff.spawned_at;
            if age >= puff.lifetime {
                puff.spawned_at = elapsed;
                puff.lifetime = 2.6 + hash2(puff.seed, elapsed) * 1.4;
                puff.seed += 1.0;
            }
        }
    }

    pub fn draw(
        &self,
        buffer: &mut [u32],
        width: usize,
        height: usize,
        camera: &Camera,
        elapsed: f32,
    ) {
        let w = width as f32;
        let h = height as f32;
        for puff in &self.puffs {
            let age = (elapsed - puff.spawned_at).max(0.0);
            let life_t = (age / puff.lifetime).clamp(0.0, 1.0);
            if life_t <= 0.0 {
                continue;
            }

            // Sube y se mece en el aire, cada vez mas lejos del eje de la
            // chimenea a medida que envejece (el humo real se dispersa).
            let rise = age * 0.55;
            let sway = (elapsed * 0.6 + puff.seed * 2.3).sin() * 0.12 * age;
            let drift = (elapsed * 0.4 + puff.seed * 1.7).cos() * 0.08 * age;
            let world = self.anchor + Vec3::new(sway, rise, drift);

            let Some((px, py, depth)) = camera.project(world, w, h) else {
                continue;
            };

            let world_radius = 0.1 + life_t * 0.3;
            let pixel_radius = (world_radius / depth / camera.fov_scale) * (h * 0.5);
            let fade_in = (age / 0.4).min(1.0);
            let alpha = fade_in * (1.0 - life_t) * 0.35;
            let tone = 205.0 - life_t * 25.0;

            blend_disc(
                buffer,
                width,
                height,
                (px, py, pixel_radius),
                (tone, tone, tone + 6.0),
                alpha,
            );
        }
    }
}
