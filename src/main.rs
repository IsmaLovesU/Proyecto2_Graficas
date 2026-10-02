mod block;
mod framebuffer;
mod light;
mod material;
mod overlay;
mod ray;
mod render;
mod scene;
mod skybox;
mod texture;
mod vec3;

use std::time::Instant;

use framebuffer::Framebuffer;
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use overlay::{draw_hud, Camera as OverlayCamera, Smoke, Snowfall};
use render::{combined_lights, fov_scale, render_scene, FrameContext};
use scene::{build_scene, CHIMNEY_TOP};
use skybox::Skybox;
use vec3::Vec3;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const ASPECT_RATIO: f32 = WIDTH as f32 / HEIGHT as f32;

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
const MAX_DISTANCE: f32 = 24.0;

const DEFAULT_YAW: f32 = 0.6;
const DEFAULT_PITCH: f32 = 0.32;
const DEFAULT_DISTANCE: f32 = 9.0;

// Arranca de noche cerrada, como el chalet original; de ahi el usuario puede
// llevar la hora del dia hacia el dia con los controles N/D.
const DEFAULT_TIME_OF_DAY: f32 = 0.08;
// Velocidad de scrubbing manual (N/D) y de avance automatico (T).
const DAYNIGHT_SCRUB_SPEED: f32 = 0.08;
const AUTO_CYCLE_SPEED: f32 = 0.015;

// Tamano de bloque (en pixeles) usado como vista previa mientras se orbita,
// se hace zoom o se mueve la hora del dia: 6 significa "un rayo cada 6x6
// pixeles", 36 veces mas barato que a resolucion completa.
const PREVIEW_PIXEL_SIZE: usize = 6;

const HUD_LINES: [&str; 2] = [
    "FLECHAS ORBITA   W S ZOOM   R RESET",
    "N D NOCHE DIA   T AUTO   P FOTO   ESC SALIR",
];

/// Modo sin ventana: renderiza un solo frame y lo guarda en disco. Sirve
/// para sacar capturas del diorama sin depender de que haya una pantalla
/// (por ejemplo, para verificar la escena desde una terminal remota).
fn render_to_file(path: &str) -> std::io::Result<()> {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let scene = build_scene();
    let skybox = Skybox::new();
    let lights = combined_lights(&scene, DEFAULT_TIME_OF_DAY);

    let target = Vec3::new(0.0, 1.1, 0.0);
    let eye = orbit_eye(target, DEFAULT_YAW, DEFAULT_PITCH, DEFAULT_DISTANCE);
    let (forward, right, up) = camera_basis(eye, target);
    let ctx = FrameContext::new(&scene, &skybox, &lights, DEFAULT_TIME_OF_DAY, 0.0);
    let started_at = Instant::now();
    render_scene(&mut framebuffer, &ctx, eye, forward, right, up, 1);
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
         R para reiniciar la vista, N/D para mover la hora del dia hacia la noche \
         o el dia, T para alternar el ciclo automatico, P para capturar, Esc para salir."
    );

    let mut base = Framebuffer::new(WIDTH, HEIGHT);
    let scene = build_scene();
    let skybox = Skybox::new();

    let target = Vec3::new(0.0, 1.1, 0.0);
    let mut yaw = DEFAULT_YAW;
    let mut pitch = DEFAULT_PITCH;
    let mut distance = DEFAULT_DISTANCE;
    let mut time_of_day = DEFAULT_TIME_OF_DAY;
    let mut auto_cycle = false;

    let eye = orbit_eye(target, yaw, pitch, distance);
    let (forward, right, up) = camera_basis(eye, target);
    let lights = combined_lights(&scene, time_of_day);
    let ctx = FrameContext::new(&scene, &skybox, &lights, time_of_day, 0.0);
    render_scene(&mut base, &ctx, eye, forward, right, up, 1);

    let mut display = base.buffer().to_vec();
    let mut snowfall = Snowfall::new(WIDTH, HEIGHT);
    let mut smoke = Smoke::new(CHIMNEY_TOP);

    let mut window = Window::new(
        "Isla flotante - flechas: orbitar | W/S: zoom | N/D: dia-noche | ESC: salir",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: false,
            ..WindowOptions::default()
        },
    )?;
    window.set_target_fps(30);
    let mut was_moving = false;
    let start_time = Instant::now();
    let mut last_frame = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let dt = last_frame.elapsed().as_secs_f32().min(0.1);
        last_frame = Instant::now();
        let elapsed = start_time.elapsed().as_secs_f32();

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
        if window.is_key_pressed(Key::T, KeyRepeat::No) {
            auto_cycle = !auto_cycle;
        }
        if window.is_key_down(Key::N) {
            time_of_day -= DAYNIGHT_SCRUB_SPEED * dt;
            dirty = true;
        }
        if window.is_key_down(Key::D) {
            time_of_day += DAYNIGHT_SCRUB_SPEED * dt;
            dirty = true;
        }
        if auto_cycle {
            time_of_day += AUTO_CYCLE_SPEED * dt;
            dirty = true;
        }
        time_of_day = time_of_day.rem_euclid(1.0);

        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            match base.save_bmp("captura.bmp") {
                Ok(()) => println!("Captura guardada en 'captura.bmp'"),
                Err(error) => println!("No se pudo guardar la captura: {error}"),
            }
        }

        let eye = orbit_eye(target, yaw, pitch, distance);
        let (forward, right, up) = camera_basis(eye, target);

        // Mientras se mantiene una tecla de movimiento (camara u hora del
        // dia), renderiza en bloques grandes (barato, se siente instantaneo);
        // en cuanto se suelta, una ultima pasada a resolucion completa deja
        // la imagen nitida. La nieve y el humo, en cambio, se redibujan
        // siempre a continuacion sin tocar el raytracer.
        let is_moving = dirty;
        if is_moving || was_moving {
            let lights = combined_lights(&scene, time_of_day);
            let ctx = FrameContext::new(&scene, &skybox, &lights, time_of_day, elapsed);
            let pixel_size = if is_moving { PREVIEW_PIXEL_SIZE } else { 1 };
            render_scene(&mut base, &ctx, eye, forward, right, up, pixel_size);
        }
        was_moving = is_moving;

        display.copy_from_slice(base.buffer());

        snowfall.update(dt, WIDTH, HEIGHT);
        snowfall.draw(&mut display, WIDTH, HEIGHT, elapsed);

        smoke.update(elapsed);
        let overlay_camera = OverlayCamera {
            eye,
            forward,
            right,
            up,
            fov_scale: fov_scale(),
            aspect: ASPECT_RATIO,
        };
        smoke.draw(&mut display, WIDTH, HEIGHT, &overlay_camera, elapsed);

        draw_hud(&mut display, WIDTH, HEIGHT, &HUD_LINES);

        window.update_with_buffer(&display, WIDTH, HEIGHT)?;
    }

    Ok(())
}
