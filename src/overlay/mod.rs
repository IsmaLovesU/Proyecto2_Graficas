//! Efectos que se dibujan encima del framebuffer ya trazado (nieve, humo,
//! HUD). Son baratos a proposito: mezclar un par de cientos de pixeles no
//! cuesta nada comparado con un raytrace, asi que pueden redibujarse en
//! *cada* frame a 30 fps aunque la escena 3D de fondo solo se vuelva a
//! trazar cuando la camara se mueve o cambia la hora del dia. Esa separacion
//! es lo que permite que la nieve y el humo se vean fluidos sin tener que
//! volver a trazar rayos todo el tiempo.
mod blend;
mod camera;
mod hud;
mod smoke;
mod snow;

pub use camera::Camera;
pub use hud::draw_hud;
pub use smoke::Smoke;
pub use snow::Snowfall;
