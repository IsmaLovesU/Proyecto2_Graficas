use crate::texture::{placeholders, Texture};

/// Como interactua la superficie de un bloque con la luz. `albedo` reparte
/// cuanto le corresponde a cada componente del modelo de Phong/Whitted:
/// `[difuso, especular, reflectivo, transparente]`. Un material solido como
/// la madera o la piedra deja los ultimos dos en cero; el hielo y el vidrio
/// son los unicos con `transparente > 0`.
pub struct Material {
    pub texture: Texture,
    pub albedo: [f32; 4],
    pub specular_exponent: f32,
    pub refractive_index: f32,
}

impl Material {
    fn new(
        texture: Texture,
        albedo: [f32; 4],
        specular_exponent: f32,
        refractive_index: f32,
    ) -> Self {
        Self {
            texture,
            albedo,
            specular_exponent,
            refractive_index,
        }
    }

    pub fn wood() -> Self {
        let texture = Texture::load_or_placeholder("assets/textures/wood.bmp", placeholders::wood);
        Self::new(texture, [0.85, 0.1, 0.0, 0.0], 12.0, 1.0)
    }

    pub fn stone() -> Self {
        let texture =
            Texture::load_or_placeholder("assets/textures/stone.bmp", placeholders::stone);
        // Reflectivo en 0: una piedra mate no gana nada viendose reflejando
        // el entorno, y dejarlo en 0 exacto evita que cada pixel de piedra
        // dispare un rayo de reflexion recursivo (el suelo/paredes cubren
        // casi toda la pantalla, asi que ese detalle invisible salia carisimo).
        Self::new(texture, [0.9, 0.15, 0.0, 0.0], 18.0, 1.0)
    }

    pub fn snow() -> Self {
        let texture = Texture::load_or_placeholder("assets/textures/snow.bmp", placeholders::snow);
        Self::new(texture, [0.9, 0.2, 0.0, 0.0], 20.0, 1.0)
    }

    /// Hielo del estanque congelado: la mayor parte de la luz se refracta,
    /// y a angulos rasantes (ver `fresnel_schlick` en `main.rs`) se refleja
    /// como un espejo, tal como se ve un lago congelado real.
    pub fn ice() -> Self {
        let texture = Texture::load_or_placeholder("assets/textures/ice.bmp", placeholders::ice);
        Self::new(texture, [0.05, 0.4, 0.1, 0.85], 90.0, 1.31)
    }

    /// Vidrio de las ventanas: mas transparente que el hielo y con un
    /// indice de refraccion mas alto, el tipico 1.5 del vidrio de verdad.
    pub fn glass() -> Self {
        let texture =
            Texture::load_or_placeholder("assets/textures/glass.bmp", placeholders::glass);
        Self::new(texture, [0.02, 0.6, 0.1, 0.9], 120.0, 1.5)
    }

    /// Follaje de los pinos. No cuenta para el tope de 5 materiales de la
    /// rubrica (ya estan wood/stone/snow/ice/glass) pero suma a la escena.
    pub fn leaves() -> Self {
        let texture =
            Texture::load_or_placeholder("assets/textures/leaves.bmp", placeholders::leaves);
        Self::new(texture, [0.95, 0.05, 0.0, 0.0], 8.0, 1.0)
    }
}
