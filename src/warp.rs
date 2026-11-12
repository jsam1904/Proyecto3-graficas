//! Warp (salto de hiperespacio): estado del salto y efectos visuales (líneas de velocidad, viñeta azul, destello).
//! El salto lleva a la nave hasta una distancia segura de un cuerpo (0 = Sol, i+1 = planeta i) con una curva suave
//! (arranque lento, crucero rápido, frenado) y el destino sigue su órbita mientras dura el viaje.

use crate::config;
use crate::framebuffer::Framebuffer;
use crate::math::*;
use crate::noise::hash01;

pub struct Warp {
    /// Cuerpo destino: 0 = Sol, i+1 = planeta i.
    pub target: usize,
    pub elapsed: f32,
    /// Posición de partida y dirección (unitaria) desde el destino hacia la partida: la nave llega por ese lado.
    pub start: Vec3,
    pub dir0: Vec3,
}

impl Warp {
    pub fn progress(&self) -> f32 { clamp(self.elapsed / config::WARP_DURATION, 0.0, 1.0) }
    /// Intensidad 0..1 del efecto: sube al arrancar, se sostiene y cae al frenar.
    pub fn level(&self) -> f32 {
        let p = self.progress();
        smoothstep(0.0, 0.15, p) * (1.0 - smoothstep(0.8, 1.0, p))
    }
    /// Avance a lo largo del trayecto (suavizado de arranque y frenada).
    pub fn eased(&self) -> f32 { let p = self.progress(); p * p * (3.0 - 2.0 * p) }
}

/// Dibuja una línea de píxeles solo donde no hay geometría; el brillo va de `b0` (inicio) a `b1` (final).
fn streak(fb: &mut Framebuffer, a: (f32, f32), b: (f32, f32), col: Vec3, b0: f32, b1: f32) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let n = dx.abs().max(dy.abs()).ceil().min(600.0) as i32;
    if n < 1 { return; }
    for i in 0..=n {
        let f = i as f32 / n as f32;
        let (x, y) = ((a.0 + dx * f) as i32, (a.1 + dy * f) as i32);
        if x < 0 || y < 0 || x as usize >= fb.w || y as usize >= fb.h { continue; }
        let idx = y as usize * fb.w + x as usize;
        if fb.depth[idx] != 0.0 { continue; }
        let k = b0 + (b1 - b0) * f;
        fb.color[idx] = crate::shaders::to_u32(col * k);
    }
}

/// Líneas de velocidad sintéticas que emanan del centro (túnel de hiperespacio), animadas con `phase`.
pub fn draw_tunnel(fb: &mut Framebuffer, level: f32, phase: f32) {
    if level < 0.02 { return; }
    let (cx, cy) = (fb.w as f32 * 0.5, fb.h as f32 * 0.5);
    let rmax = (cx * cx + cy * cy).sqrt();
    for i in 0..config::WARP_STREAKS as u32 {
        let h = |k: u32| hash01(777u32.wrapping_add(i.wrapping_mul(5)).wrapping_add(k));
        let ang = std::f32::consts::TAU * h(0);
        let vel = 0.6 + 1.4 * h(1);
        // Radio normalizado que avanza hacia el borde y se reinicia (cada línea con su velocidad y fase).
        let r = (h(2) + phase * vel * (0.6 + level)).fract();
        let r0 = 0.04 + r * r * 0.96;
        let largo = (0.02 + 0.5 * r) * level * 1.6;
        let (s, c) = ang.sin_cos();
        let a = (cx + c * r0 * rmax, cy + s * r0 * rmax);
        let b = (cx + c * (r0 + largo) * rmax, cy + s * (r0 + largo) * rmax);
        let tinte = vec3(0.65 + 0.35 * h(3), 0.8 + 0.2 * h(3), 1.0);
        streak(fb, a, b, tinte, 0.15, (0.5 + 0.5 * r).min(1.0));
    }
}

/// Estiramiento radial de un punto de pantalla (estrella) según la intensidad.
pub fn stretch(fb: &Framebuffer, p: (f32, f32), level: f32) -> (f32, f32) {
    let (cx, cy) = (fb.w as f32 * 0.5, fb.h as f32 * 0.5);
    let k = 1.0 + level * 2.2;
    (cx + (p.0 - cx) * k, cy + (p.1 - cy) * k)
}

pub fn draw_streak_for_star(fb: &mut Framebuffer, p: (f32, f32), level: f32, col: Vec3, brillo: f32) {
    let e = stretch(fb, p, level);
    streak(fb, p, e, col, 0.2 * brillo, brillo.max(0.5));
}

/// Postproceso: viñeta azul-blanca en los bordes mientras se salta y destello blanco al llegar.
pub fn post_fx(fb: &mut Framebuffer, level: f32, flash: f32) {
    if level < 0.01 && flash < 0.01 { return; }
    let (w, h) = (fb.w, fb.h);
    let (cx, cy) = (w as f32 * 0.5, h as f32 * 0.5);
    let inv = 1.0 / (cx * cx + cy * cy);
    for y in 0..h {
        let dy = y as f32 - cy;
        for x in 0..w {
            let dx = x as f32 - cx;
            let v = (dx * dx + dy * dy) * inv; // 0 centro .. 1 esquina
            let c = fb.color[y * w + x];
            let mut r = ((c >> 16) & 255) as f32;
            let mut g = ((c >> 8) & 255) as f32;
            let mut b = (c & 255) as f32;
            let glow = level * (0.10 + 0.55 * v * v);
            r += 80.0 * glow; g += 150.0 * glow; b += 255.0 * glow;
            r += (255.0 - r) * flash; g += (255.0 - g) * flash; b += (255.0 - b) * flash;
            fb.color[y * w + x] = ((r.min(255.0) as u32) << 16) | ((g.min(255.0) as u32) << 8) | b.min(255.0) as u32;
        }
    }
}
