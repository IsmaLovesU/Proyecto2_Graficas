use crate::vec3::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Intersect {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material_id: usize,
    pub u: f32,
    pub v: f32,
    /// Si la superficie debe animarse desplazando su textura con el tiempo
    /// (la cascada de agua). El resto de los bloques la dejan en `false`.
    pub flowing: bool,
}

pub trait RayIntersect {
    fn ray_intersect(&self, origin: Vec3, direction: Vec3) -> Option<Intersect>;
}
