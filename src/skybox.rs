use crate::texture::hash2;
use crate::vec3::Vec3;

/// Cielo nocturno de invierno, generado por formula a partir de la
/// direccion del rayo: no necesita ningun archivo de textura. Degradado
/// frio de horizonte a cenit, estrellas dispersas y una luna con halo suave.
pub struct Skybox {
    moon_direction: Vec3,
}

impl Skybox {
    pub fn new() -> Self {
        Self {
            moon_direction: Vec3::new(0.35, 0.55, -0.4).normalized(),
        }
    }

    pub fn sample(&self, direction: Vec3) -> Vec3 {
        let direction = direction.normalized();

        // t=0 en el horizonte (tono violeta calido), t=1 en el cenit (azul
        // casi negro): asi el cielo se siente frio arriba y con un poco de
        // resplandor abajo, como el atardecer que ya se apago.
        let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
        let horizon = Vec3::new(45.0, 40.0, 70.0);
        let zenith = Vec3::new(8.0, 10.0, 28.0);
        let sky = horizon * (1.0 - t) + zenith * t;

        let stars = self.star_field(direction) * (direction.y.max(0.0));
        let moon = self.moon_glow(direction);

        sky + Vec3::splat(stars) + moon
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

    fn moon_glow(&self, direction: Vec3) -> Vec3 {
        let alignment = direction.dot(self.moon_direction);
        if alignment > 0.9985 {
            Vec3::splat(255.0)
        } else if alignment > 0.985 {
            // Halo: se apaga suave desde el borde del disco hacia afuera.
            let falloff = (alignment - 0.985) / (0.9985 - 0.985);
            Vec3::new(120.0, 130.0, 160.0) * falloff
        } else {
            Vec3::splat(0.0)
        }
    }
}
