use crate::framebuffer::{rgb, Framebuffer};
use crate::light::Light;
use crate::ray::{Intersect, RayIntersect};
use crate::scene::Scene;
use crate::skybox::{celestial_direction, day_amount, Skybox};
use crate::vec3::Vec3;
use crate::ASPECT_RATIO;

const AMBIENT_STRENGTH: f32 = 0.16;
const SHADOW_BIAS: f32 = 1e-3;
const MAX_DEPTH: u32 = 3;
// Por debajo de esto, seguir el rebote no cambia el pixel de forma visible
// (menos de un paso de los 256 niveles de un canal de color), asi que ni
// vale la pena pagar otro `cast_ray` recursivo por esa contribucion.
const MIN_CONTRIBUTION: f32 = 0.02;
// Nuestra iluminacion no es fisicamente exacta (no hay radiometria real
// detras de las intensidades que elegimos a mano), asi que un empujon de
// exposicion antes de recortar a 8 bits evita que la escena de noche
// termine viendose casi negra. De dia el sol ya aporta de sobra, asi que
// ese mismo empujon se reduce (`day_exposure`) para que la nieve no termine
// quemada a blanco puro.
const NIGHT_EXPOSURE: f32 = 1.25;
const DAY_EXPOSURE: f32 = 0.85;
// Velocidad a la que se desplaza la textura de los bloques "flowing" (la
// cascada), para que se lea como agua cayendo y no como hielo quieto.
const FLOW_SPEED: f32 = 0.8;

pub fn fov_scale() -> f32 {
    (60.0_f32.to_radians() / 2.0).tan()
}

fn day_exposure(time_of_day: f32) -> f32 {
    let day = day_amount(time_of_day);
    NIGHT_EXPOSURE * (1.0 - day) + DAY_EXPOSURE * day
}

fn to_rgb(color: Vec3, exposure: f32) -> u32 {
    let channel = |value: f32| (value * exposure).clamp(0.0, 255.0) as u8;
    rgb(channel(color.x), channel(color.y), channel(color.z))
}

/// Todo lo que necesita un rayo para resolverse en un frame dado: la escena
/// estatica, el cielo, las luces de ese instante (las dinamicas sol/luna ya
/// mezcladas con las fijas de la cabana) y el reloj de la animacion. Agrupar
/// esto evita ir arrastrando media docena de parametros sueltos por cada
/// funcion de render.
pub struct FrameContext<'a> {
    scene: &'a Scene,
    skybox: &'a Skybox,
    lights: &'a [Light],
    time_of_day: f32,
    elapsed: f32,
}

impl<'a> FrameContext<'a> {
    pub fn new(
        scene: &'a Scene,
        skybox: &'a Skybox,
        lights: &'a [Light],
        time_of_day: f32,
        elapsed: f32,
    ) -> Self {
        Self {
            scene,
            skybox,
            lights,
            time_of_day,
            elapsed,
        }
    }
}

/// Luz direccional principal (sol o luna segun la hora del dia) mas una de
/// relleno, para que el lado en sombra no se pierda en negro puro. Se
/// recalculan cada frame a partir de `time_of_day`: por eso no viven en
/// `Scene`, que solo se construye una vez.
fn celestial_lights(time_of_day: f32) -> [Light; 2] {
    let direction = celestial_direction(time_of_day);
    let day = day_amount(time_of_day);

    let night_key = Vec3::new(0.6, 0.65, 0.9);
    let day_key = Vec3::new(1.0, 0.95, 0.85);
    let key_color = night_key * (1.0 - day) + day_key * day;
    // El tope de dia se quedo corto a proposito (antes llegaba a 1.5): junto
    // con `DAY_EXPOSURE` mas bajo, evita que la nieve se queme a blanco puro
    // al mediodia.
    let key_intensity = 0.85 + day * 0.25;

    let fill_direction = Vec3::new(-0.4, 0.3, 0.5);
    let night_fill = Vec3::new(0.5, 0.55, 0.75);
    let day_fill = Vec3::new(0.75, 0.8, 0.9);
    let fill_color = night_fill * (1.0 - day) + day_fill * day;
    let fill_intensity = 0.3 + day * 0.1;

    [
        Light::directional(direction, key_intensity, key_color),
        Light::directional(fill_direction, fill_intensity, fill_color),
    ]
}

pub fn combined_lights(scene: &Scene, time_of_day: f32) -> Vec<Light> {
    let mut lights = Vec::with_capacity(scene.lights.len() + 2);
    lights.extend(celestial_lights(time_of_day));
    lights.extend_from_slice(&scene.lights);
    lights
}

fn nearest_intersection(origin: Vec3, direction: Vec3, scene: &Scene) -> Option<Intersect> {
    scene
        .blocks
        .iter()
        .filter_map(|block| block.ray_intersect(origin, direction))
        .min_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Para una sombra no importa cual es el obstaculo mas cercano, solo si hay
/// alguno antes de llegar a la luz: `any()` corta apenas encuentra el
/// primero, en vez de recorrer los bloques que quedan como hace
/// `nearest_intersection` (que si necesita el minimo real).
fn is_occluded(origin: Vec3, direction: Vec3, max_distance: f32, scene: &Scene) -> bool {
    scene.blocks.iter().any(|block| {
        block
            .ray_intersect(origin, direction)
            .is_some_and(|hit| hit.distance < max_distance - SHADOW_BIAS)
    })
}

fn cast_ray(origin: Vec3, direction: Vec3, ctx: &FrameContext, depth: u32) -> Vec3 {
    match nearest_intersection(origin, direction, ctx.scene) {
        Some(hit) => shade_hit(&hit, direction, ctx, depth),
        None => ctx.skybox.sample(direction, ctx.time_of_day),
    }
}

/// Aproximacion de Schlick a las ecuaciones de Fresnel: cuanto de la luz que
/// llega a una superficie transparente se refleja en vez de refractarse.
/// Sube fuerte a angulos rasantes, que es justo lo que hace que el hielo de
/// una laguna se vea como espejo cerca del horizonte y transparente de frente.
fn fresnel_schlick(cos_theta: f32, ior: f32) -> f32 {
    let r0 = ((1.0 - ior) / (1.0 + ior)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cos_theta).clamp(0.0, 1.0).powi(5)
}

fn shade_hit(hit: &Intersect, ray_dir: Vec3, ctx: &FrameContext, depth: u32) -> Vec3 {
    let material = &ctx.scene.materials[hit.material_id];
    // Los bloques de la cascada desplazan su textura con el tiempo para que
    // se lea como agua cayendo en vez de hielo quieto.
    let v_sample = if hit.flowing {
        (hit.v + ctx.elapsed * FLOW_SPEED).rem_euclid(1.0)
    } else {
        hit.v
    };
    let base_color = material.texture.sample(hit.u, v_sample);
    let view_dir = -ray_dir;

    let ambient = base_color * AMBIENT_STRENGTH;
    let mut diffuse_light = Vec3::splat(0.0);
    let mut specular_light = Vec3::splat(0.0);

    for light in ctx.lights {
        let (light_dir, light_color, intensity, max_distance) = light.sample(hit.point);
        let n_dot_l = hit.normal.dot(light_dir);
        if n_dot_l <= 0.0 {
            continue;
        }

        // Sombra: si algo bloquea el camino hacia la luz antes de llegar a
        // ella (importa la distancia para las luces puntuales de las
        // ventanas, si no cualquier pared "detras" de la luz contaria).
        let shadow_origin = hit.point + hit.normal * SHADOW_BIAS;
        if is_occluded(shadow_origin, light_dir, max_distance, ctx.scene) {
            continue;
        }

        diffuse_light = diffuse_light + light_color * (n_dot_l * intensity);

        let reflected = (-light_dir).reflect(hit.normal).normalized();
        let spec_angle = reflected.dot(view_dir).max(0.0);
        specular_light = specular_light
            + light_color * (spec_angle.powf(material.specular_exponent) * intensity);
    }

    let [diffuse_albedo, specular_albedo, reflective_albedo, transparent_albedo] = material.albedo;
    let diffuse_term = (base_color * diffuse_albedo) * diffuse_light;
    let specular_term = Vec3::splat(255.0) * specular_albedo * specular_light;
    let local_color = ambient + diffuse_term + specular_term;

    if depth >= MAX_DEPTH {
        return local_color;
    }

    let cos_theta = (-ray_dir).dot(hit.normal).abs();
    let (mut reflectivity, mut transparency) = if transparent_albedo > 0.0 {
        let fresnel = fresnel_schlick(cos_theta, material.refractive_index);
        (
            reflective_albedo.max(fresnel),
            (1.0 - fresnel) * transparent_albedo,
        )
    } else {
        (reflective_albedo, 0.0)
    };

    let mut refracted_color = Vec3::splat(0.0);
    if transparency > MIN_CONTRIBUTION {
        match ray_dir.refract(hit.normal, material.refractive_index) {
            Some(refract_dir) => {
                let bias_vec = hit.normal * SHADOW_BIAS;
                let refract_origin = if refract_dir.dot(hit.normal) < 0.0 {
                    hit.point - bias_vec
                } else {
                    hit.point + bias_vec
                };
                refracted_color = cast_ray(refract_origin, refract_dir, ctx, depth + 1);
            }
            None => {
                // Reflexion interna total: no hay rayo refractado posible,
                // asi que esa energia tambien se va a reflexion.
                reflectivity += transparency;
                transparency = 0.0;
            }
        }
    }

    let reflected_color = if reflectivity > MIN_CONTRIBUTION {
        let reflect_dir = ray_dir.reflect(hit.normal).normalized();
        let reflect_origin = hit.point + hit.normal * SHADOW_BIAS;
        cast_ray(reflect_origin, reflect_dir, ctx, depth + 1)
    } else {
        Vec3::splat(0.0)
    };

    let local_weight = (1.0 - reflectivity - transparency).max(0.0);
    local_color * local_weight + reflected_color * reflectivity + refracted_color * transparency
}

/// Reparte las filas del framebuffer entre varios hilos con
/// `std::thread::scope` (de la libreria estandar, no hace falta ningun
/// crate). Cada hilo solo lee la escena y escribe su propia porcion del
/// buffer, asi que no hace falta `Mutex` ni `Arc` para nada.
///
/// `pixel_size` es el tamano (en pixeles) de cada "bloque" que se calcula
/// con un solo rayo: en 1 renderiza a resolucion completa; en, digamos, 4,
/// dispara un rayo cada 4x4 pixeles y repite ese color en todo el bloque.
/// Eso da una vista previa mucho mas barata mientras se orbita, se hace
/// zoom o se mueve la hora del dia, y al soltar la tecla se vuelve a pedir
/// con `pixel_size = 1`.
#[allow(clippy::too_many_arguments)]
pub fn render_scene(
    framebuffer: &mut Framebuffer,
    ctx: &FrameContext,
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    pixel_size: usize,
) {
    let width = framebuffer.width;
    let height = framebuffer.height;
    let pixel_size = pixel_size.max(1);

    let thread_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let base_rows = height.div_ceil(thread_count).max(1);
    // Redondeamos hacia arriba al multiplo de `pixel_size` mas cercano para
    // que ningun bloque de vista previa quede partido entre dos hilos.
    let rows_per_chunk = base_rows.div_ceil(pixel_size).max(1) * pixel_size;

    let pixels = framebuffer.pixels_mut();
    std::thread::scope(|threads| {
        for (chunk_index, chunk) in pixels.chunks_mut(rows_per_chunk * width).enumerate() {
            let row_start = chunk_index * rows_per_chunk;
            threads.spawn(move || {
                render_rows(
                    chunk, row_start, width, height, ctx, eye, forward, right, up, pixel_size,
                );
            });
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn render_rows(
    chunk: &mut [u32],
    row_start: usize,
    width: usize,
    height: usize,
    ctx: &FrameContext,
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    pixel_size: usize,
) {
    let fov_scale = fov_scale();
    let exposure = day_exposure(ctx.time_of_day);
    let chunk_rows = chunk.len() / width;

    let mut local_y = 0;
    while local_y < chunk_rows {
        let block_height = pixel_size.min(chunk_rows - local_y);
        let mut x = 0;
        while x < width {
            let block_width = pixel_size.min(width - x);

            // Un solo rayo por bloque, disparado desde su centro.
            let sample_row = (row_start + local_y + block_height / 2).min(height - 1);
            let sample_col = (x + block_width / 2).min(width - 1);
            let screen_x =
                (2.0 * (sample_col as f32 + 0.5) / width as f32 - 1.0) * ASPECT_RATIO * fov_scale;
            let screen_y = (1.0 - 2.0 * (sample_row as f32 + 0.5) / height as f32) * fov_scale;
            let direction = (right * screen_x + up * screen_y + forward).normalized();
            let color = to_rgb(cast_ray(eye, direction, ctx, 0), exposure);

            for by in 0..block_height {
                let row_offset = (local_y + by) * width;
                for bx in 0..block_width {
                    chunk[row_offset + x + bx] = color;
                }
            }

            x += block_width;
        }
        local_y += block_height;
    }
}
