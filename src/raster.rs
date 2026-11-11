//! Rasterización de triángulos por coordenadas baricéntricas con z-buffer de 1/w.

use crate::framebuffer::{Framebuffer, Target};
use crate::math::*;
use crate::shaders::{to_u32, FragmentInput, Shader, Uniforms};

/// Vértice después del vertex stage: posición en clip + atributos a interpolar.
#[derive(Clone, Copy)]
pub struct ClipVertex {
    pub clip: Vec4,
    pub world: Vec3,
    pub local: Vec3,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub uv: Vec2,
}

impl ClipVertex {
    /// Interpolación lineal (la usará el clipping en la Fase 3).
    pub fn lerp(&self, o: &ClipVertex, t: f32) -> ClipVertex {
        ClipVertex {
            clip: self.clip.lerp(o.clip, t),
            world: self.world.lerp(o.world, t),
            local: self.local.lerp(o.local, t),
            normal: self.normal.lerp(o.normal, t),
            tangent: self.tangent.lerp(o.tangent, t),
            uv: self.uv.lerp(o.uv, t),
        }
    }
}

/// Distancia con signo al plano near en espacio clip: >= 0 dentro (z >= -w).
fn near_dist(c: Vec4) -> f32 { c.z + c.w }

/// Recorte Sutherland–Hodgman de un triángulo contra el plano near, en espacio CLIP
/// (antes de dividir por w; después de dividir, los vértices detrás de la cámara se "voltean" y no hay arreglo).
/// Por cada arista se conserva el vértice si está dentro y se añade el punto de corte si la arista cruza el plano.
/// Devuelve hasta 4 vértices (un triángulo recortado puede quedar como cuadrilátero) y cuántos hay.
/// Los atributos se interpolan linealmente en clip: es correcto porque aún no se dividió por w.
pub fn clip_near(tri: [ClipVertex; 3]) -> ([ClipVertex; 4], usize) {
    let mut out = [tri[0]; 4];
    let mut n = 0;
    for i in 0..3 {
        let (a, b) = (&tri[i], &tri[(i + 1) % 3]);
        let (da, db) = (near_dist(a.clip), near_dist(b.clip));
        if da >= 0.0 { out[n] = *a; n += 1; }
        if (da >= 0.0) != (db >= 0.0) {
            out[n] = a.lerp(b, da / (da - db)); // t donde la distancia vale 0
            n += 1;
        }
    }
    (out, n)
}

/// Clip -> NDC (dividir por w) -> pantalla (Y invertida: arriba = 0).
pub fn to_screen(c: Vec4, w: usize, h: usize) -> (f32, f32, f32) {
    let inv_w = 1.0 / c.w;
    ((c.x * inv_w * 0.5 + 0.5) * w as f32, (1.0 - (c.y * inv_w * 0.5 + 0.5)) * h as f32, inv_w)
}

/// Función de borde: positiva a un lado de la arista a->b y negativa al otro.
fn edge(ax: f32, ay: f32, bx: f32, by: f32, px: f32, py: f32) -> f32 {
    (bx - ax) * (py - ay) - (by - ay) * (px - ax)
}

/// Triángulo ya listo para rasterizar: vértices (con atributos), posiciones en pantalla (x, y, 1/w),
/// 1/área y qué material (shader + uniforms) lo dibuja.
pub struct ScreenTri {
    pub v: [ClipVertex; 3],
    pub s: [(f32, f32, f32); 3],
    pub inv_area: f32,
    pub material: usize,
    pub min_y: f32,
    pub max_y: f32,
}

/// Prepara un triángulo: pasa a pantalla y descarta caras traseras (o degeneradas).
/// Cara frontal = antihoraria en NDC = área negativa en pantalla (Y hacia abajo).
pub fn setup_triangle(v: [&ClipVertex; 3], w: usize, h: usize, cull_back: bool, material: usize) -> Option<ScreenTri> {
    let s = [to_screen(v[0].clip, w, h), to_screen(v[1].clip, w, h), to_screen(v[2].clip, w, h)];
    let area = edge(s[0].0, s[0].1, s[1].0, s[1].1, s[2].0, s[2].1);
    if area == 0.0 || (cull_back && area > 0.0) { return None; }
    let min_y = s[0].1.min(s[1].1).min(s[2].1);
    let max_y = s[0].1.max(s[1].1).max(s[2].1);
    Some(ScreenTri { v: [*v[0], *v[1], *v[2]], s, inv_area: 1.0 / area, material, min_y, max_y })
}

/// Rasteriza el triángulo dentro de una banda de filas: caja envolvente, baricéntricas, z-buffer y shader.
pub fn fill_triangle(t: &mut Target, tri: &ScreenTri, u: &Uniforms, shader: Shader) {
    let s = &tri.s;
    let v = &tri.v;
    // Fuera de la banda: nada que hacer.
    if tri.max_y < t.y0 as f32 || tri.min_y >= t.y1 as f32 { return; }
    let w = t.w;
    let min_x = s.iter().map(|p| p.0).fold(f32::MAX, f32::min).floor().max(0.0) as usize;
    let max_x = s.iter().map(|p| p.0).fold(f32::MIN, f32::max).ceil().min(w as f32 - 1.0).max(0.0) as usize;
    let min_y = tri.min_y.floor().max(t.y0 as f32) as usize;
    let max_y = tri.max_y.ceil().min(t.y1 as f32 - 1.0) as usize;

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            // Coordenadas baricéntricas en pantalla.
            let b0 = edge(s[1].0, s[1].1, s[2].0, s[2].1, px, py) * tri.inv_area;
            let b1 = edge(s[2].0, s[2].1, s[0].0, s[0].1, px, py) * tri.inv_area;
            let b2 = edge(s[0].0, s[0].1, s[1].0, s[1].1, px, py) * tri.inv_area;
            if b0 < 0.0 || b1 < 0.0 || b2 < 0.0 { continue; }

            // 1/w es lineal en pantalla: se interpola directo y sirve de profundidad.
            let inv_w = b0 * s[0].2 + b1 * s[1].2 + b2 * s[2].2;
            let idx = (y - t.y0) * w + x;
            if inv_w <= t.depth[idx] { continue; } // más lejos (o igual) que lo ya dibujado
            t.depth[idx] = inv_w;

            // Corrección de perspectiva: peso_i = b_i * (1/w_i) / (1/w).
            let p0 = b0 * s[0].2 / inv_w;
            let p1 = b1 * s[1].2 / inv_w;
            let p2 = b2 * s[2].2 / inv_w;
            let mix3 = |a: Vec3, b: Vec3, c: Vec3| a * p0 + b * p1 + c * p2;
            let frag = FragmentInput {
                world_pos: mix3(v[0].world, v[1].world, v[2].world),
                local_pos: mix3(v[0].local, v[1].local, v[2].local),
                normal: mix3(v[0].normal, v[1].normal, v[2].normal),
                tangent: mix3(v[0].tangent, v[1].tangent, v[2].tangent),
                uv: v[0].uv * p0 + v[1].uv * p1 + v[2].uv * p2,
            };
            t.color[idx] = to_u32(shader(&frag, u));
        }
    }
}

/// Línea 2D (Bresenham) sin prueba de profundidad. Se usa para wireframe y ejes de depuración.
pub fn draw_line(fb: &mut Framebuffer, x0: f32, y0: f32, x1: f32, y1: f32, color: u32) {
    let (mut x0, mut y0, x1, y1) = (x0.round() as i32, y0.round() as i32, x1.round() as i32, y1.round() as i32);
    // Protege contra coordenadas enormes (vértices casi en el plano de la cámara).
    if x0.abs() > 20000 || y0.abs() > 20000 || x1.abs() > 20000 || y1.abs() > 20000 { return; }
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    loop {
        fb.put_pixel(x0, y0, color);
        if x0 == x1 && y0 == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}
