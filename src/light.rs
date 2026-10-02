use crate::vec3::Vec3;

/// Una luz direccional (como la luna: no tiene posicion, solo una direccion
/// de incidencia) o puntual (como una ventana iluminada o un farol, con
/// posicion propia y atenuacion por distancia).
#[derive(Clone, Copy)]
pub enum Light {
    Directional {
        direction: Vec3,
        color: Vec3,
        intensity: f32,
    },
    Point {
        position: Vec3,
        color: Vec3,
        intensity: f32,
    },
}

impl Light {
    pub fn directional(direction: Vec3, intensity: f32, color: Vec3) -> Self {
        Light::Directional {
            direction: direction.normalized(),
            color,
            intensity,
        }
    }

    pub fn point(position: Vec3, intensity: f32, color: Vec3) -> Self {
        Light::Point {
            position,
            color,
            intensity,
        }
    }

    /// Direccion (hacia la luz), color, intensidad ya atenuada por distancia,
    /// y la distancia maxima que puede bloquear una sombra (infinita para
    /// una luz direccional, la distancia real para una puntual).
    pub fn sample(&self, point: Vec3) -> (Vec3, Vec3, f32, f32) {
        match *self {
            Light::Directional {
                direction,
                color,
                intensity,
            } => (direction, color, intensity, f32::INFINITY),
            Light::Point {
                position,
                color,
                intensity,
            } => {
                let offset = position - point;
                let distance = offset.length();
                let direction = if distance <= f32::EPSILON {
                    offset
                } else {
                    offset * (1.0 / distance)
                };
                // Atenuacion cuadratica clasica, con un piso para que no
                // explote a distancia casi cero junto al farol.
                let falloff = intensity / (1.0 + distance * distance * 0.08);
                (direction, color, falloff, distance)
            }
        }
    }
}
