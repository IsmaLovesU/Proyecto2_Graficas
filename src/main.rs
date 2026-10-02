mod block;
mod framebuffer;
mod light;
mod material;
mod ray;
mod scene;
mod skybox;
mod texture;
mod vec3;

use framebuffer::{rgb, Framebuffer};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use ray::{Intersect, RayIntersect};
use scene::{build_scene, Scene};
use skybox::Skybox;
use vec3::Vec3;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
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
// termine viendose casi negra.
const EXPOSURE: f32 = 1.25;

fn to_rgb(color: Vec3) -> u32 {
    let channel = |value: f32| (value * EXPOSURE).clamp(0.0, 255.0) as u8;
    rgb(channel(color.x), channel(color.y), channel(color.z))
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

fn cast_ray(origin: Vec3, direction: Vec3, scene: &Scene, skybox: &Skybox, depth: u32) -> Vec3 {
    match nearest_intersection(origin, direction, scene) {
        Some(hit) => shade_hit(&hit, origin, direction, scene, skybox, depth),
        None => skybox.sample(direction),
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

fn shade_hit(
    hit: &Intersect,
    _ray_origin: Vec3,
    ray_dir: Vec3,
    scene: &Scene,
    skybox: &Skybox,
    depth: u32,
) -> Vec3 {
    let material = &scene.materials[hit.material_id];
    let base_color = material.texture.sample(hit.u, hit.v);
    let view_dir = -ray_dir;

    let ambient = base_color * AMBIENT_STRENGTH;
    let mut diffuse_light = Vec3::splat(0.0);
    let mut specular_light = Vec3::splat(0.0);

    for light in &scene.lights {
        let (light_dir, light_color, intensity, max_distance) = light.sample(hit.point);
        let n_dot_l = hit.normal.dot(light_dir);
        if n_dot_l <= 0.0 {
            continue;
        }

        // Sombra: si algo bloquea el camino hacia la luz antes de llegar a
        // ella (importa la distancia para las luces puntuales de las
        // ventanas, si no cualquier pared "detras" de la luz contaria).
        let shadow_origin = hit.point + hit.normal * SHADOW_BIAS;
        if is_occluded(shadow_origin, light_dir, max_distance, scene) {
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
                refracted_color = cast_ray(refract_origin, refract_dir, scene, skybox, depth + 1);
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
        cast_ray(reflect_origin, reflect_dir, scene, skybox, depth + 1)
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
/// Eso da una vista previa mucho mas barata mientras se orbita o se hace
/// zoom, y al soltar la tecla se vuelve a pedir con `pixel_size = 1`.
#[allow(clippy::too_many_arguments)]
fn render_scene(
    framebuffer: &mut Framebuffer,
    scene: &Scene,
    skybox: &Skybox,
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
                    chunk, row_start, width, height, scene, skybox, eye, forward, right, up,
                    pixel_size,
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
    scene: &Scene,
    skybox: &Skybox,
    eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    pixel_size: usize,
) {
    let aspect_ratio = width as f32 / height as f32;
    let fov_scale = (60.0_f32.to_radians() / 2.0).tan();
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
                (2.0 * (sample_col as f32 + 0.5) / width as f32 - 1.0) * aspect_ratio * fov_scale;
            let screen_y = (1.0 - 2.0 * (sample_row as f32 + 0.5) / height as f32) * fov_scale;
            let direction = (right * screen_x + up * screen_y + forward).normalized();
            let color = to_rgb(cast_ray(eye, direction, scene, skybox, 0));

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

const WORLD_UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

fn orbit_eye(target: Vec3, yaw: f32, pitch: f32, distance: f32) -> Vec3 {
    Vec3::new(
        target.x + distance * pitch.cos() * yaw.sin(),
        target.y + distance * pitch.sin(),
        target.z + distance * pitch.cos() * yaw.cos(),
    )
}

fn camera_basis(eye: Vec3, target: Vec3) -> (Vec3, Vec3, Vec3) {
    let forward = (target - eye).normalized();
    let right = forward.cross(WORLD_UP).normalized();
    let up = right.cross(forward).normalized();
    (forward, right, up)
}

const ORBIT_SPEED: f32 = 0.03;
const ZOOM_SPEED: f32 = 0.1;
const PITCH_LIMIT: f32 = 1.4;
const MIN_DISTANCE: f32 = 3.0;
const MAX_DISTANCE: f32 = 16.0;

const DEFAULT_YAW: f32 = 0.6;
const DEFAULT_PITCH: f32 = 0.32;
const DEFAULT_DISTANCE: f32 = 9.0;

// Tamano de bloque (en pixeles) usado como vista previa mientras se orbita o
// se hace zoom: 6 significa "un rayo cada 6x6 pixeles", 36 veces mas barato
// que a resolucion completa.
const PREVIEW_PIXEL_SIZE: usize = 6;

/// Modo sin ventana: renderiza un solo frame y lo guarda en disco. Sirve
/// para sacar capturas del diorama sin depender de que haya una pantalla
/// (por ejemplo, para verificar la escena desde una terminal remota).
fn render_to_file(path: &str) -> std::io::Result<()> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let scene = build_scene();
    let skybox = Skybox::new();

    let target = Vec3::new(0.0, 1.1, 0.0);
    let eye = orbit_eye(target, DEFAULT_YAW, DEFAULT_PITCH, DEFAULT_DISTANCE);
    let (forward, right, up) = camera_basis(eye, target);
    let started_at = std::time::Instant::now();
    render_scene(
        &mut framebuffer,
        &scene,
        &skybox,
        eye,
        forward,
        right,
        up,
        1,
    );
    println!(
        "Render a resolucion completa: {:.1} ms",
        started_at.elapsed().as_secs_f64() * 1000.0
    );

    framebuffer.save_bmp(path)
}

fn main() -> Result<(), minifb::Error> {
    if let Some(path) =
        std::env::args().find_map(|arg| arg.strip_prefix("--screenshot=").map(String::from))
    {
        render_to_file(&path).expect("no se pudo guardar la captura");
        println!("Captura guardada en '{path}'");
        return Ok(());
    }

    println!(
        "Controles: flechas para orbitar la camara, W/S para acercar o alejar, \
         R para reiniciar la vista, P para capturar, Esc para salir."
    );

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let scene = build_scene();
    let skybox = Skybox::new();

    let target = Vec3::new(0.0, 1.1, 0.0);
    let mut yaw = DEFAULT_YAW;
    let mut pitch = DEFAULT_PITCH;
    let mut distance = DEFAULT_DISTANCE;

    let eye = orbit_eye(target, yaw, pitch, distance);
    let (forward, right, up) = camera_basis(eye, target);
    render_scene(
        &mut framebuffer,
        &scene,
        &skybox,
        eye,
        forward,
        right,
        up,
        1,
    );

    let mut window = Window::new(
        "Cabana suiza - flechas: orbitar | W/S: zoom | R: reset | ESC: salir",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(30);
    let mut was_moving = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut dirty = false;

        if window.is_key_down(Key::Left) {
            yaw -= ORBIT_SPEED;
            dirty = true;
        }
        if window.is_key_down(Key::Right) {
            yaw += ORBIT_SPEED;
            dirty = true;
        }
        if window.is_key_down(Key::Up) {
            pitch = (pitch + ORBIT_SPEED).min(PITCH_LIMIT);
            dirty = true;
        }
        if window.is_key_down(Key::Down) {
            pitch = (pitch - ORBIT_SPEED).max(-PITCH_LIMIT);
            dirty = true;
        }
        if window.is_key_down(Key::W) {
            distance = (distance - ZOOM_SPEED).max(MIN_DISTANCE);
            dirty = true;
        }
        if window.is_key_down(Key::S) {
            distance = (distance + ZOOM_SPEED).min(MAX_DISTANCE);
            dirty = true;
        }
        if window.is_key_down(Key::R) {
            yaw = DEFAULT_YAW;
            pitch = DEFAULT_PITCH;
            distance = DEFAULT_DISTANCE;
            dirty = true;
        }
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            match framebuffer.save_bmp("captura.bmp") {
                Ok(()) => println!("Captura guardada en 'captura.bmp'"),
                Err(error) => println!("No se pudo guardar la captura: {error}"),
            }
        }

        // Mientras se mantiene una tecla de movimiento, renderiza en bloques
        // grandes (barato, se siente instantaneo); en cuanto se suelta, una
        // ultima pasada a resolucion completa deja la imagen nitida.
        let is_moving = dirty;
        if is_moving || was_moving {
            let eye = orbit_eye(target, yaw, pitch, distance);
            let (forward, right, up) = camera_basis(eye, target);
            let pixel_size = if is_moving { PREVIEW_PIXEL_SIZE } else { 1 };
            render_scene(
                &mut framebuffer,
                &scene,
                &skybox,
                eye,
                forward,
                right,
                up,
                pixel_size,
            );
        }
        was_moving = is_moving;

        window.update_with_buffer(framebuffer.buffer(), WIDTH, HEIGHT)?;
    }

    Ok(())
}
