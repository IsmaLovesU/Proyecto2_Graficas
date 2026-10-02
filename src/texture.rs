use std::fs;

use crate::vec3::Vec3;

/// Textura muestreable por coordenadas `(u, v)` en `[0, 1]`. Se puede cargar
/// desde un BMP de 24 bits sin comprimir (el unico formato que vale la pena
/// parsear a mano) o generar en memoria con un patron procedural, para poder
/// avanzar el motor antes de tener las texturas finales.
pub struct Texture {
    width: usize,
    height: usize,
    texels: Vec<Vec3>,
}

impl Texture {
    pub fn from_fn(width: usize, height: usize, paint: impl Fn(usize, usize) -> Vec3) -> Self {
        let mut texels = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                texels.push(paint(x, y));
            }
        }
        Self {
            width,
            height,
            texels,
        }
    }

    /// Intenta cargar un BMP en `path`; si el archivo todavia no existe usa
    /// `placeholder` en su lugar. Asi el proyecto corre y se ve completo
    /// mientras se consiguen las texturas reales, y el dia que aparezca el
    /// archivo en disco no hay que tocar una sola linea de codigo.
    pub fn load_or_placeholder(path: &str, placeholder: impl FnOnce() -> Self) -> Self {
        match Self::from_bmp(path) {
            Some(texture) => texture,
            None => {
                println!("[texturas] no encontre '{path}', uso un patron de relleno");
                placeholder()
            }
        }
    }

    /// Parseo minimo de BMP de 24 bits sin comprimir (BI_RGB). No soporta
    /// paletas, RLE ni canal alfa: para eso ya existen crates, y este
    /// proyecto no puede usar ninguno.
    pub fn from_bmp(path: &str) -> Option<Self> {
        let bytes = fs::read(path).ok()?;
        if bytes.len() < 54 || &bytes[0..2] != b"BM" {
            return None;
        }

        let data_offset = read_u32(&bytes, 10) as usize;
        let width = read_i32(&bytes, 18);
        let height_raw = read_i32(&bytes, 22);
        let bits_per_pixel = read_u16(&bytes, 28);
        let compression = read_u32(&bytes, 30);

        if bits_per_pixel != 24 || compression != 0 || width <= 0 || height_raw == 0 {
            return None;
        }

        let width = width as usize;
        let height = height_raw.unsigned_abs() as usize;
        let bottom_up = height_raw > 0;
        // Cada fila de un BMP se rellena con ceros hasta multiplo de 4 bytes.
        let row_size = (width * 3 + 3) & !3;

        let mut texels = vec![Vec3::splat(0.0); width * height];
        for row in 0..height {
            let src_row = if bottom_up { height - 1 - row } else { row };
            let row_start = data_offset + src_row * row_size;
            for col in 0..width {
                let pixel_start = row_start + col * 3;
                if pixel_start + 2 >= bytes.len() {
                    return None;
                }
                // BMP guarda los canales en orden BGR.
                let b = bytes[pixel_start] as f32;
                let g = bytes[pixel_start + 1] as f32;
                let r = bytes[pixel_start + 2] as f32;
                texels[row * width + col] = Vec3::new(r, g, b);
            }
        }

        Some(Self {
            width,
            height,
            texels,
        })
    }

    /// Muestreo por vecino mas cercano; con las resoluciones chicas que usa
    /// este proyecto (texturas de bloque, no fotos) no hace falta bilinear.
    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let u = u.rem_euclid(1.0);
        let v = v.rem_euclid(1.0);
        let x = ((u * self.width as f32) as usize).min(self.width - 1);
        let y = ((v * self.height as f32) as usize).min(self.height - 1);
        self.texels[y * self.width + x]
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn read_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

/// Ruido pseudoaleatorio determinista sin `rand` (no podemos traer esa
/// dependencia): un hash trigonometrico barato pero suficiente para romper
/// la uniformidad de los patrones de relleno.
pub fn hash2(x: f32, y: f32) -> f32 {
    let n = (x * 12.9898 + y * 78.233).sin() * 43_758.547;
    n.fract().abs()
}

pub mod placeholders {
    use super::{hash2, Texture};
    use crate::vec3::Vec3;

    const SIZE: usize = 64;

    /// Tablones horizontales cafes, como los de un chalet suizo.
    pub fn wood() -> Texture {
        Texture::from_fn(SIZE, SIZE, |x, y| {
            let plank = (y / 8) % 2;
            let base = if plank == 0 {
                Vec3::new(133.0, 94.0, 58.0)
            } else {
                Vec3::new(115.0, 80.0, 48.0)
            };
            let grain = hash2(x as f32 * 0.6, (y / 8) as f32) * 18.0;
            base + Vec3::splat(grain) - Vec3::splat(9.0)
        })
    }

    /// Piedra de mamposteria gris con manchas.
    pub fn stone() -> Texture {
        Texture::from_fn(SIZE, SIZE, |x, y| {
            let speckle = hash2(x as f32, y as f32) * 40.0;
            Vec3::new(120.0, 118.0, 115.0) + Vec3::splat(speckle) - Vec3::splat(20.0)
        })
    }

    /// Nieve blanca con un dejo azulado, no un blanco plano.
    pub fn snow() -> Texture {
        Texture::from_fn(SIZE, SIZE, |x, y| {
            let sparkle = hash2(x as f32 * 2.0, y as f32 * 2.0) * 15.0;
            Vec3::new(225.0, 232.0, 240.0) + Vec3::splat(sparkle)
        })
    }

    /// Hielo celeste palido con lineas finas simulando grietas.
    pub fn ice() -> Texture {
        Texture::from_fn(SIZE, SIZE, |x, y| {
            let crack = ((x as i32 - y as i32) % 17 == 0) as u8 as f32 * 30.0;
            Vec3::new(180.0, 215.0, 230.0) - Vec3::splat(crack)
        })
    }

    /// Vidrio casi transparente con un ligero tono escarchado.
    pub fn glass() -> Texture {
        Texture::from_fn(SIZE, SIZE, |x, y| {
            let frost = hash2(x as f32, y as f32) * 10.0;
            Vec3::new(225.0, 235.0, 240.0) + Vec3::splat(frost)
        })
    }

    /// Follaje verde oscuro para los pinos.
    pub fn leaves() -> Texture {
        Texture::from_fn(SIZE, SIZE, |x, y| {
            let shade = hash2(x as f32, y as f32) * 25.0;
            Vec3::new(35.0, 70.0, 40.0) + Vec3::splat(shade) - Vec3::splat(12.0)
        })
    }
}
