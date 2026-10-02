use crate::vec3::Vec3;

/// Camara "congelada" del frame actual, solo para proyectar puntos del
/// mundo a pantalla (el humo de la chimenea necesita saber donde cae en
/// pantalla, no trazar rayos). Usa la misma base ortonormal y el mismo FOV
/// que `render_rows`, para que lo que se dibuja encima coincida con lo que
/// se trazo.
pub struct Camera {
    pub eye: Vec3,
    pub forward: Vec3,
    pub right: Vec3,
    pub up: Vec3,
    pub fov_scale: f32,
    pub aspect: f32,
}

impl Camera {
    /// Pixel `(x, y)` donde cae `point`, mas su profundidad (para escalar el
    /// tamano con la distancia). `None` si queda detras de la camara.
    pub fn project(&self, point: Vec3, width: f32, height: f32) -> Option<(f32, f32, f32)> {
        let local = point - self.eye;
        let depth = local.dot(self.forward);
        if depth <= 0.05 {
            return None;
        }
        let ndc_x = local.dot(self.right) / depth / (self.fov_scale * self.aspect);
        let ndc_y = local.dot(self.up) / depth / self.fov_scale;
        let px = (ndc_x + 1.0) * 0.5 * width;
        let py = (1.0 - ndc_y) * 0.5 * height;
        Some((px, py, depth))
    }
}
