# Cabaña Suiza — Isla Flotante (Raytracer en Rust)

Diorama renderizado con un motor de raytracing escrito desde cero en Rust: una cabaña suiza
nevada sobre una isla flotante, con ciclo de día/noche, nieve cayendo, humo de chimenea y una
cascada de hielo que se derrama hacia el vacío estrellado.

Proyecto 2 del curso de Gráficas por Computadora — raytracing sobre bloques (AABBs) texturizados,
sin dependencias externas más allá de una librería mínima de ventaneo (`minifb`).

## Demo

[Video demostrativo](https://youtu.be/ckVhHafKre0)


## Características

- **Motor de raytracing propio**: intersección rayo-AABB, sombreado estilo Whitted (difuso +
  especular + sombras), reflexión y refracción combinadas con Fresnel-Schlick.
- **6 materiales** con textura, albedo, especularidad, transparencia y reflectividad propios:
  madera, piedra, nieve, hielo (refractivo, IOR 1.31), vidrio (refractivo, IOR 1.5) y hojas.
- **Refracción contextual**: la cascada y el estanque congelado usan el material de hielo.
- **Reflexión**: superficies metálicas/vidrio reflejan el entorno.
- **Skybox procedural**: cielo nocturno con estrellas y luna, cielo diurno con degradado y sol,
  mezclados dinámicamente según la hora del día.
- **Ciclo de día/noche**: automático (`T`) o manual (`N`/`D`), con luces de sol/luna recalculadas
  por frame.
- **Cámara orbital con zoom**, igual que el esquema de proyectos anteriores del curso.
- **Efectos de pantalla (overlay)**: nieve cayendo, humo animado de chimenea y un HUD con los
  controles, dibujados en espacio de pantalla para no afectar el rendimiento del raytracing.
- **Renderizado en paralelo** sobre todos los núcleos disponibles (`std::thread::scope`, sin
  crates de concurrencia), con vista previa en bloques mientras se mueve la cámara u la hora del
  día, y un pase a resolución completa al soltar la tecla.
- **Captura de pantalla** sin necesidad de ventana, vía `--screenshot=ruta.bmp`, o con la tecla `P`
  durante la ejecución.

## Controles

| Tecla         | Acción                              |
|---------------|-------------------------------------|
| Flechas       | Orbitar la cámara                   |
| `W` / `S`     | Acercar / alejar (zoom)             |
| `R`           | Reiniciar la vista                  |
| `N` / `D`     | Mover la hora del día hacia noche/día |
| `T`           | Alternar ciclo automático de día/noche |
| `P`           | Guardar una captura (`captura.bmp`) |
| `Esc`         | Salir                               |

## Cómo ejecutarlo

```bash
cargo run --release
```

Para generar una captura sin abrir ventana (útil en un entorno sin pantalla):

```bash
cargo run --release -- --screenshot=captura.bmp
```

## Estructura del proyecto

```
src/
├── main.rs            # Loop de ventana/input y cámara orbital
├── render.rs           # Núcleo del raytracer: cast_ray, shade_hit, hilos de render
├── scene/              # Construcción de la escena
│   ├── mod.rs           # Orquesta la escena completa
│   ├── island.rs        # Isla flotante, terreno y cascada
│   ├── cabin.rs          # Paredes, techo y chimenea de la cabaña
│   ├── porch.rs          # Porche, deck, barandal y detalles de ventanas
│   ├── decor.rs          # Árboles, camino, muñeco de nieve, arbustos, etc.
│   └── geometry.rs       # Helpers compartidos (perforado de huecos)
├── overlay/             # Efectos 2D sobre el framebuffer ya renderizado
│   ├── blend.rs, camera.rs, snow.rs, smoke.rs, hud.rs
├── block.rs             # AABB con mapeo de UV por cara
├── material.rs          # Definición de los materiales
├── texture.rs            # Cargador de BMP + texturas procedurales de respaldo
├── light.rs              # Luces direccionales y puntuales
├── skybox.rs             # Cielo procedural día/noche
├── vec3.rs / ray.rs / framebuffer.rs
```

## Materiales

| Material | Textura | Propiedad destacada |
|----------|---------|----------------------|
| Madera   | Vetas de madera | Difuso |
| Piedra   | Piedra rugosa   | Difuso |
| Nieve    | Nieve con brillo | Especular suave |
| Hielo    | Translúcido     | Refracción (IOR 1.31) |
| Vidrio   | Translúcido     | Refracción (IOR 1.5) |
| Hojas    | Follaje         | Difuso (material extra) |

Las texturas se cargan desde `assets/textures/*.bmp` (BMP de 24 bits sin comprimir); si no existen,
el motor genera texturas procedurales equivalentes automáticamente, así que el proyecto corre y se
ve completo sin archivos adicionales.

## Notas técnicas

- Sin dependencias externas de graficas/matemáticas: toda la matemática de vectores, intersección
  rayo-caja, texturizado y sombreado está escrita a mano.
- Sin `unsafe`, sin `Rc`/`RefCell`/`Arc`/`Mutex`: los bloques referencian sus materiales por índice
  (`material_id: usize`) en vez de punteros compartidos.
- `cargo clippy` y `cargo fmt` pasan sin advertencias.
