use std::io::{self, Write};

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    buffer: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }

    pub fn buffer(&self) -> &[u32] {
        &self.buffer
    }

    pub fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.buffer
    }

    /// Vuelca el framebuffer a un BMP de 24 bits, para poder guardar
    /// capturas del diorama sin depender de ninguna libreria de imagenes.
    pub fn save_bmp(&self, path: &str) -> io::Result<()> {
        let row_size = (self.width * 3 + 3) & !3;
        let pixel_data_size = row_size * self.height;
        let file_size = 54 + pixel_data_size;

        let mut file = std::fs::File::create(path)?;
        let mut header = Vec::with_capacity(54);
        header.extend_from_slice(b"BM");
        header.extend_from_slice(&(file_size as u32).to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&54u32.to_le_bytes());
        header.extend_from_slice(&40u32.to_le_bytes());
        header.extend_from_slice(&(self.width as u32).to_le_bytes());
        header.extend_from_slice(&(self.height as u32).to_le_bytes());
        header.extend_from_slice(&1u16.to_le_bytes());
        header.extend_from_slice(&24u16.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
        header.extend_from_slice(&2835u32.to_le_bytes());
        header.extend_from_slice(&2835u32.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        file.write_all(&header)?;

        let padding = vec![0u8; row_size - self.width * 3];
        for row in (0..self.height).rev() {
            for col in 0..self.width {
                let pixel = self.buffer[row * self.width + col];
                let bytes = [
                    (pixel & 0xff) as u8,
                    ((pixel >> 8) & 0xff) as u8,
                    ((pixel >> 16) & 0xff) as u8,
                ];
                file.write_all(&bytes)?;
            }
            file.write_all(&padding)?;
        }

        Ok(())
    }
}

/// Empaqueta tres canales de color de 8 bits en el formato que usa `minifb`.
pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}
