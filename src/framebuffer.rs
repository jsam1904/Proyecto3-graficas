//! Framebuffer de color (0x00RRGGBB) y buffer de profundidad (1/w).

use std::io::Write;

/// Vista mutable de una banda de filas [y0, y1) del framebuffer: permite que varios hilos rasterizen
/// bandas distintas a la vez sin compartir memoria (cada banda es un trozo disjunto de los Vec).
pub struct Target<'a> {
    pub w: usize,
    pub y0: usize,
    pub y1: usize,
    pub color: &'a mut [u32],
    pub depth: &'a mut [f32],
}

pub struct Framebuffer {
    pub w: usize,
    pub h: usize,
    pub color: Vec<u32>,
    /// Guarda 1/w por píxel: más grande = más cerca; 0 = vacío (infinito).
    pub depth: Vec<f32>,
}

/// Empaqueta r,g,b (0..255) en un u32.
#[inline]
pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

impl Framebuffer {
    pub fn new(w: usize, h: usize) -> Self {
        Self { w, h, color: vec![0; w * h], depth: vec![0.0; w * h] }
    }

    /// Divide el framebuffer en bandas de `filas` filas (la última puede ser menor).
    pub fn split_bands(&mut self, filas: usize) -> Vec<Target<'_>> {
        let w = self.w;
        self.color.chunks_mut(w * filas).zip(self.depth.chunks_mut(w * filas)).enumerate()
            .map(|(i, (c, d))| Target { w, y0: i * filas, y1: i * filas + c.len() / w, color: c, depth: d })
            .collect()
    }

    /// Limpia color y profundidad al inicio de cada cuadro.
    pub fn clear(&mut self, c: u32) {
        self.color.fill(c);
        self.depth.fill(0.0);
    }

    /// Escribe un píxel sin prueba de profundidad (HUD, líneas 2D).
    #[inline]
    pub fn put_pixel(&mut self, x: i32, y: i32, c: u32) {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.color[y as usize * self.w + x as usize] = c;
        }
    }

    /// Convierte a bytes RGBA para subirlos como textura.
    pub fn to_rgba(&self, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(self.w * self.h * 4);
        for &p in &self.color {
            out.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8, 255]);
        }
    }

    /// Guarda el framebuffer como PPM binario (P6).
    pub fn save_ppm(&self, path: &str) -> std::io::Result<()> {
        let mut f = std::io::BufWriter::new(std::fs::File::create(path)?);
        write!(f, "P6\n{} {}\n255\n", self.w, self.h)?;
        for &p in &self.color {
            f.write_all(&[(p >> 16) as u8, (p >> 8) as u8, p as u8])?;
        }
        Ok(())
    }
}
