//! HUD: fuente bitmap 5x7 propia (sin texto de raylib) dibujada sobre el framebuffer.

use crate::framebuffer::{rgb, Framebuffer};
use crate::pipeline::Stats;

/// Cada glifo son 7 filas de 5 bits (bit 4 = columna izquierda).
fn glyph(c: char) -> [u8; 7] {
    match c.to_ascii_uppercase() {
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1E, 0x01, 0x01, 0x0E, 0x01, 0x01, 0x1E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '.' => [0, 0, 0, 0, 0, 0x0C, 0x0C],
        ':' => [0, 0x0C, 0x0C, 0, 0x0C, 0x0C, 0],
        ',' => [0, 0, 0, 0, 0x0C, 0x04, 0x08],
        '-' => [0, 0, 0, 0x1F, 0, 0, 0],
        '+' => [0, 0x04, 0x04, 0x1F, 0x04, 0x04, 0],
        '=' => [0, 0, 0x1F, 0, 0x1F, 0, 0],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '%' => [0x18, 0x19, 0x02, 0x04, 0x08, 0x13, 0x03],
        '(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        ')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        '[' => [0x0E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x0E],
        ']' => [0x0E, 0x02, 0x02, 0x02, 0x02, 0x02, 0x0E],
        '<' => [0x02, 0x04, 0x08, 0x10, 0x08, 0x04, 0x02],
        '>' => [0x08, 0x04, 0x02, 0x01, 0x02, 0x04, 0x08],
        '_' => [0, 0, 0, 0, 0, 0, 0x1F],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0, 0x04],
        '?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0, 0x04],
        '\'' => [0x04, 0x04, 0x08, 0, 0, 0, 0],
        '^' => [0x04, 0x0A, 0x11, 0, 0, 0, 0],
        '*' => [0, 0x0A, 0x04, 0x1F, 0x04, 0x0A, 0],
        _ => [0; 7], // espacio y desconocidos
    }
}

/// Dibuja `text` en (x,y) con escala entera `s`. Sombra negra de 1 px de escala para legibilidad.
pub fn draw_text(fb: &mut Framebuffer, x: i32, y: i32, text: &str, s: i32, color: u32) {
    for (dx, dy, c) in [(s, s, 0u32), (0, 0, color)] {
        let mut cx = x;
        for ch in text.chars() {
            let g = glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for col in 0..5 {
                    if bits & (0x10 >> col) != 0 {
                        for py in 0..s {
                            for px in 0..s {
                                fb.put_pixel(cx + col * s + px + dx, y + row as i32 * s + py + dy, c);
                            }
                        }
                    }
                }
            }
            cx += 6 * s; // 5 de ancho + 1 de separación
        }
    }
}

/// Medidor de FPS suavizado (promedio sobre ventanas de 0.5 s).
pub struct FpsCounter {
    acc: f32,
    frames: u32,
    pub fps: f32,
    pub ms: f32,
}

impl FpsCounter {
    pub fn new() -> Self {
        Self { acc: 0.0, frames: 0, fps: 0.0, ms: 0.0 }
    }
    pub fn tick(&mut self, dt: f32) {
        self.acc += dt;
        self.frames += 1;
        if self.acc >= 0.5 {
            self.fps = self.frames as f32 / self.acc;
            self.ms = 1000.0 * self.acc / self.frames as f32;
            self.acc = 0.0;
            self.frames = 0;
        }
    }
}

/// Oscurece un rectángulo (fondo del panel del HUD) para que el texto se lea sobre cualquier escena.
pub fn dim_rect(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32) {
    for j in y.max(0)..(y + h).min(fb.h as i32) {
        for i in x.max(0)..(x + w).min(fb.w as i32) {
            let c = &mut fb.color[j as usize * fb.w + i as usize];
            *c = (*c >> 1) & 0x7F7F7F; // 50 % de brillo
        }
    }
}

/// Datos que muestra el HUD.
pub struct HudData<'a> {
    pub fps: &'a FpsCounter,
    pub stats: Stats,
    pub speed: f32,
    pub altitude: f32,
    pub cuerpo: &'a str,
    /// Nombre del planeta si está en modo local.
    pub local: Option<&'a str>,
    pub camara: &'a str,
    pub time_scale: f32,
    pub pausa: bool,
    pub seed: u64,
    pub grav: f32,
    pub lods: String,
    pub cull: bool,
    pub lod: bool,
    pub wire: bool,
    /// Salto en curso: (nombre del destino, progreso 0..1).
    pub warp: Option<(&'a str, f32)>,
    /// Mensaje temporal (p. ej. "sin destino").
    pub aviso: Option<&'a str>,
}

pub fn draw_hud(fb: &mut Framebuffer, d: &HudData) {
    let (b, c) = (rgb(255, 255, 255), rgb(190, 215, 255));
    let st = &d.stats;
    dim_rect(fb, 0, 0, 600, 138);
    draw_text(fb, 8, 6, &format!("FPS: {:.0}  MS: {:.1}", d.fps.fps, d.fps.ms), 2, b);
    draw_text(fb, 8, 24, &format!("TRIS ENV: {}  CULL: {}  CLIP: {}  RAST: {}", st.tris_sent, st.tris_culled, st.tris_clipped, st.tris_raster), 2, c);
    draw_text(fb, 8, 42, &format!("OBJ: {}/{}  FRUSTUM: {}  LOD-OMIT: {}", st.objs_drawn, st.objs_total, st.objs_culled, st.objs_lod_omit), 2, c);
    draw_text(fb, 8, 60, &format!("VEL: {:.0} U/S  ALT: {:.0} ({})  G: {:.1}", d.speed, d.altitude, d.cuerpo, d.grav), 2, b);
    let pausa = if d.pausa { " [PAUSA]" } else { "" };
    let modo = if d.local.is_some() { "LOCAL" } else { "ESPACIAL" };
    draw_text(fb, 8, 78, &format!("MODO: {}  CAM: {}  T: X{}{}  SEED: {}", modo, d.camara, d.time_scale, pausa, d.seed), 2, c);
    draw_text(fb, 8, 96, &format!("LOD: {}", d.lods), 2, c);
    let onoff = |v: bool| if v { "ON" } else { "OFF" };
    draw_text(fb, 8, 116, &format!("F2 CULLING: {}   F3 LOD: {}   F4 WIRE: {}", onoff(d.cull), onoff(d.lod), onoff(d.wire)), 2, rgb(255, 225, 140));

    if let Some(n) = d.local {
        let txt = format!("MODO LOCAL: {}", n);
        let x = (fb.w as i32 - txt.len() as i32 * 12) / 2;
        dim_rect(fb, x - 10, 6, txt.len() as i32 * 12 + 20, 22);
        draw_text(fb, x, 10, &txt, 2, rgb(120, 255, 160));
    }
    if let Some((n, p)) = d.warp {
        let txt = format!("WARP >> {}  {:.0}%", n, p * 100.0);
        let x = (fb.w as i32 - txt.len() as i32 * 12) / 2;
        let y = fb.h as i32 - 80;
        dim_rect(fb, x - 10, y - 4, txt.len() as i32 * 12 + 20, 22);
        draw_text(fb, x, y, &txt, 2, rgb(150, 200, 255));
    } else if let Some(m) = d.aviso {
        let x = (fb.w as i32 - m.len() as i32 * 12) / 2;
        let y = fb.h as i32 - 80;
        dim_rect(fb, x - 10, y - 4, m.len() as i32 * 12 + 20, 22);
        draw_text(fb, x, y, m, 2, rgb(255, 225, 140));
    }
    let y = fb.h as i32 - 30;
    dim_rect(fb, 0, y - 4, fb.w as i32, 34);
    let g = rgb(185, 200, 225);
    draw_text(fb, 8, y, "W/S EMPUJE  A/D GUINADA  Q/E ALABEO  UP/DOWN CABECEO  SHIFT TURBO  ESPACIO/CTRL SUBIR/BAJAR (MODO LOCAL)", 1, g);
    draw_text(fb, 8, y + 10, "G WARP AL PLANETA APUNTADO  C CAMARA  P PAUSA  [ ] TIEMPO  IZQ/DER O CLIC DER+MOUSE: MIRAR  F1 HUD  F2 CULLING  F3 LOD  F4 WIRE  ESC SALIR", 1, g);
    if d.local.is_none() && d.altitude < 120.0 && d.cuerpo != "SOL" {
        draw_text(fb, 8, y - 12, "ACERCATE A UN PLANETA PARA ACTIVAR EL MODO LOCAL", 1, rgb(255, 225, 140));
    }
}
