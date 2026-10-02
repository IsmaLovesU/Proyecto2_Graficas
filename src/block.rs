use crate::ray::{Intersect, RayIntersect};
use crate::vec3::Vec3;

/// Un bloque es una caja alineada a los ejes, no necesariamente un cubo
/// perfecto: un panel de ventana o una capa de nieve sobre el techo son
/// bloques igual de validos, solo que aplanados en un eje.
pub struct Block {
    pub min: Vec3,
    pub max: Vec3,
    pub material_id: usize,
}

impl Block {
    pub fn new(min: Vec3, max: Vec3, material_id: usize) -> Self {
        Self {
            min,
            max,
            material_id,
        }
    }

    pub fn cube(center: Vec3, size: f32, material_id: usize) -> Self {
        Self::centered(center, Vec3::splat(size * 0.5), material_id)
    }

    pub fn centered(center: Vec3, half_extents: Vec3, material_id: usize) -> Self {
        Self::new(center - half_extents, center + half_extents, material_id)
    }

    /// Coordenadas de textura dentro de la cara golpeada: las dos
    /// componentes que no son el eje de la normal, normalizadas a `[0, 1]`
    /// segun el tamano del bloque en esos ejes.
    fn face_uv(&self, hit_axis: usize, point: Vec3) -> (f32, f32) {
        let size = self.max - self.min;
        let frac = |value: f32, min: f32, span: f32| {
            if span.abs() < f32::EPSILON {
                0.0
            } else {
                (value - min) / span
            }
        };

        match hit_axis {
            0 => (
                frac(point.z, self.min.z, size.z),
                frac(point.y, self.min.y, size.y),
            ),
            1 => (
                frac(point.x, self.min.x, size.x),
                frac(point.z, self.min.z, size.z),
            ),
            _ => (
                frac(point.x, self.min.x, size.x),
                frac(point.y, self.min.y, size.y),
            ),
        }
    }
}

/// Interseccion rayo-franja para un solo eje: el intervalo `[t_near, t_far]`
/// en el que el rayo esta dentro de `[min, max]` en ese eje, junto con el
/// signo de la cara que corresponde a `t_near`. Un rayo paralelo al eje o ya
/// esta dentro de la franja para siempre o nunca puede entrar.
fn slab_intersect(origin: f32, direction: f32, min: f32, max: f32) -> Option<(f32, f32, f32)> {
    if direction.abs() < f32::EPSILON {
        if origin < min || origin > max {
            return None;
        }
        return Some((f32::NEG_INFINITY, f32::INFINITY, 0.0));
    }

    let inv_direction = 1.0 / direction;
    let mut t_min = (min - origin) * inv_direction;
    let mut t_max = (max - origin) * inv_direction;
    let mut sign_min = -1.0;
    let mut sign_max = 1.0;

    if t_min > t_max {
        std::mem::swap(&mut t_min, &mut t_max);
        std::mem::swap(&mut sign_min, &mut sign_max);
    }

    Some((t_min, t_max, sign_min))
}

impl RayIntersect for Block {
    fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<Intersect> {
        let axes = [
            (origin.x, direction.x, self.min.x, self.max.x),
            (origin.y, direction.y, self.min.y, self.max.y),
            (origin.z, direction.z, self.min.z, self.max.z),
        ];

        let mut t_near = f32::NEG_INFINITY;
        let mut t_far = f32::INFINITY;
        let mut hit_axis = 0usize;
        let mut hit_sign = 0.0f32;

        for (axis, &(o, d, axis_min, axis_max)) in axes.iter().enumerate() {
            let (t0, t1, sign) = slab_intersect(o, d, axis_min, axis_max)?;

            if t0 > t_near {
                t_near = t0;
                hit_axis = axis;
                hit_sign = sign;
            }
            if t1 < t_far {
                t_far = t1;
            }
        }

        if t_near > t_far || t_near <= f32::EPSILON {
            return None;
        }

        let distance = t_near;
        let point = origin + direction * distance;
        let normal = match hit_axis {
            0 => Vec3::new(hit_sign, 0.0, 0.0),
            1 => Vec3::new(0.0, hit_sign, 0.0),
            _ => Vec3::new(0.0, 0.0, hit_sign),
        };
        let (u, v) = self.face_uv(hit_axis, point);

        Some(Intersect {
            distance,
            point,
            normal,
            material_id: self.material_id,
            u,
            v,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_through_the_center_hits_the_block() {
        let block = Block::cube(Vec3::new(0.0, 0.0, 0.0), 2.0, 0);
        let hit = block
            .ray_intersect(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 0.0, -1.0))
            .expect("el rayo deberia tocar el bloque");

        assert!((hit.distance - 2.0).abs() < 0.0001);
        assert_eq!(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn ray_away_from_the_block_misses() {
        let block = Block::cube(Vec3::new(0.0, 0.0, 0.0), 2.0, 0);
        assert!(block
            .ray_intersect(Vec3::new(0.0, 0.0, 3.0), Vec3::new(0.0, 1.0, 0.0))
            .is_none());
    }

    #[test]
    fn non_uniform_block_reports_uv_inside_unit_range() {
        let block = Block::new(Vec3::new(-1.0, -0.1, -2.0), Vec3::new(1.0, 0.1, 2.0), 0);
        let hit = block
            .ray_intersect(Vec3::new(0.5, 5.0, 0.5), Vec3::new(0.0, -1.0, 0.0))
            .expect("el rayo deberia tocar la tapa superior");

        assert!((0.0..=1.0).contains(&hit.u));
        assert!((0.0..=1.0).contains(&hit.v));
    }
}
