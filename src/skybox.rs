use crate::texture::hash2;
use crate::vec3::Vec3;

/// Cielo que pasa de noche estrellada a dia despejado segun `time_of_day`
/// (`0.0`/`1.0` = medianoche, `0.5` = mediodia), generado por formula a
/// partir de la direccion del rayo: no necesita ningun archivo de textura.
/// Mirar hacia abajo (lejos de la isla flotante) siempre cae en un vacio
/// oscuro, de dia o de noche, para reforzar que no hay suelo debajo.
pub struct Skybox;

/// Dado que tanto el cielo como la luz principal de la escena (sol o luna)
/// dependen del mismo angulo, esta cuenta vive en un solo lugar y la usan
/// `skybox` y `main` (para la luz dinamica) por igual.
pub fn celestial_direction(time_of_day: f32) -> Vec3 {
    let angle = (time_of_day - 0.25) * std::f32::consts::TAU;
    Vec3::new(0.35, angle.sin(), -0.4).normalized()
}

/// `0.0` = noche cerrada, `1.0` = pleno dia. Usa la elevacion del cuerpo
/// celeste con un borde suave alrededor del horizonte (amanecer/atardecer).
pub fn day_amount(time_of_day: f32) -> f32 {
    let angle = (time_of_day - 0.25) * std::f32::consts::TAU;
    (angle.sin() * 2.0 + 0.5).clamp(0.0, 1.0)
}

impl Skybox {
    pub fn new() -> Self {
        Self
    }

    pub fn sample(&self, direction: Vec3, time_of_day: f32) -> Vec3 {
        let direction = direction.normalized();
        let day = day_amount(time_of_day);

        let night_horizon = Vec3::new(45.0, 40.0, 70.0);
        let night_zenith = Vec3::new(8.0, 10.0, 28.0);
        let day_horizon = Vec3::new(196.0, 210.0, 230.0);
        let day_zenith = Vec3::new(70.0, 130.0, 225.0);
        let horizon = night_horizon * (1.0 - day) + day_horizon * day;
        let zenith = night_zenith * (1.0 - day) + day_zenith * day;

        let sky = if direction.y >= 0.0 {
            let t = direction.y;
            horizon * (1.0 - t) + zenith * t
        } else {
            // Mirando hacia abajo: nada sostiene la isla, asi que el color
            // cae rapido hacia un vacio casi negro en vez de "horizonte".
            let t = (-direction.y).min(1.0);
            let void_color = Vec3::splat(3.0);
            horizon * (1.0 - t) + void_color * t
        };

        let stars = self.star_field(direction) * (1.0 - day) * direction.y.max(0.0);
        let body = self.celestial_body(direction, time_of_day, day);

        sky + Vec3::splat(stars) + body
    }

    fn star_field(&self, direction: Vec3) -> f32 {
        let scaled = direction * 400.0;
        let brightness = hash2(scaled.x + scaled.y, scaled.y + scaled.z);
        if brightness > 0.9965 {
            220.0
        } else {
            0.0
        }
    }

    /// Luna fria de noche, sol calido de dia: mismo disco, distinto color,
    /// mezclado segun `day`.
    fn celestial_body(&self, direction: Vec3, time_of_day: f32, day: f32) -> Vec3 {
        let body_direction = celestial_direction(time_of_day);
        let alignment = direction.dot(body_direction);
        if alignment <= 0.985 {
            return Vec3::splat(0.0);
        }

        let moon_disc = Vec3::splat(255.0);
        let sun_disc = Vec3::new(255.0, 250.0, 230.0);
        let disc_color = moon_disc * (1.0 - day) + sun_disc * day;

        if alignment > 0.9985 {
            disc_color
        } else {
            let falloff = (alignment - 0.985) / (0.9985 - 0.985);
            let moon_halo = Vec3::new(120.0, 130.0, 160.0);
            let sun_halo = Vec3::new(255.0, 220.0, 150.0);
            (moon_halo * (1.0 - day) + sun_halo * day) * falloff
        }
    }
}
