//! Hash de enteros y (en fases siguientes) ruido procedural escrito a mano.

/// Hash de 32 bits (mezcla tipo "lowbias32"): buena dispersión, barato y determinista.
pub fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// Hash a float en [0,1).
pub fn hash01(x: u32) -> f32 { (hash_u32(x) >> 8) as f32 / 16_777_216.0 }

// ---------------------------------------------------------------------------------------------
// Ruido 3D. Se usa con la posición en el espacio del objeto (o la dirección unitaria del planeta),
// así no hay costuras como pasaría con UV.
// ---------------------------------------------------------------------------------------------
use crate::math::*;

/// Hash de una celda entera 3D.
pub fn hash3(x: i32, y: i32, z: i32) -> u32 {
    hash_u32((x as u32).wrapping_mul(73_856_093) ^ (y as u32).wrapping_mul(19_349_663) ^ (z as u32).wrapping_mul(83_492_791))
}

/// Curva de suavizado quíntica de Perlin: 6t^5 - 15t^4 + 10t^3 (derivada nula en los extremos).
fn fade(t: f32) -> f32 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

/// Value noise: valores aleatorios en los vértices de la cuadrícula, interpolados. Resultado en [0,1].
pub fn value3(p: Vec3) -> f32 {
    let (ix, iy, iz) = (p.x.floor(), p.y.floor(), p.z.floor());
    let (fx, fy, fz) = (fade(p.x - ix), fade(p.y - iy), fade(p.z - iz));
    let (ix, iy, iz) = (ix as i32, iy as i32, iz as i32);
    let v = |dx: i32, dy: i32, dz: i32| (hash3(ix + dx, iy + dy, iz + dz) >> 8) as f32 / 16_777_216.0;
    let x00 = lerp(v(0, 0, 0), v(1, 0, 0), fx);
    let x10 = lerp(v(0, 1, 0), v(1, 1, 0), fx);
    let x01 = lerp(v(0, 0, 1), v(1, 0, 1), fx);
    let x11 = lerp(v(0, 1, 1), v(1, 1, 1), fx);
    lerp(lerp(x00, x10, fy), lerp(x01, x11, fy), fz)
}

/// Producto punto entre un gradiente pseudoaleatorio (uno de 12 vectores tipo arista de cubo) y el desplazamiento.
fn grad(h: u32, x: f32, y: f32, z: f32) -> f32 {
    let h = h & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 { y } else if h == 12 || h == 14 { x } else { z };
    (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
}

/// Perlin gradient noise (versión mejorada). Resultado aproximado en [-1,1].
pub fn perlin3(p: Vec3) -> f32 {
    let (fx0, fy0, fz0) = (p.x.floor(), p.y.floor(), p.z.floor());
    let (x, y, z) = (p.x - fx0, p.y - fy0, p.z - fz0);
    let (ix, iy, iz) = (fx0 as i32, fy0 as i32, fz0 as i32);
    let (u, v, w) = (fade(x), fade(y), fade(z));
    let g = |dx: i32, dy: i32, dz: i32| grad(hash3(ix + dx, iy + dy, iz + dz), x - dx as f32, y - dy as f32, z - dz as f32);
    let x00 = lerp(g(0, 0, 0), g(1, 0, 0), u);
    let x10 = lerp(g(0, 1, 0), g(1, 1, 0), u);
    let x01 = lerp(g(0, 0, 1), g(1, 0, 1), u);
    let x11 = lerp(g(0, 1, 1), g(1, 1, 1), u);
    lerp(lerp(x00, x10, v), lerp(x01, x11, v), w)
}

/// fBm: suma de octavas de Perlin, cada una al doble de frecuencia y la mitad de amplitud. En ~[-1,1].
pub fn fbm(p: Vec3, octavas: u32) -> f32 {
    let (mut suma, mut amp, mut norma, mut q) = (0.0, 1.0, 0.0, p);
    for _ in 0..octavas {
        suma += amp * perlin3(q);
        norma += amp;
        amp *= 0.5;
        q = q * 2.0;
    }
    suma / norma
}

/// Ridged noise: 1-|perlin| forma crestas afiladas (dunas, montañas). En ~[0,1].
pub fn ridged(p: Vec3, octavas: u32) -> f32 {
    let (mut suma, mut amp, mut norma, mut q) = (0.0, 1.0, 0.0, p);
    for _ in 0..octavas {
        let r = 1.0 - perlin3(q).abs();
        suma += amp * r * r; // al cuadrado: crestas más marcadas
        norma += amp;
        amp *= 0.5;
        q = q * 2.0;
    }
    suma / norma
}

/// Hash de celda entera -> float en [0,1).
pub fn hash_cell(a: i32, b: i32, c: i32) -> f32 { (hash3(a, b, c) >> 8) as f32 / 16_777_216.0 }

/// Ruido celular de Worley: distancias al punto-característica más cercano (f1) y al segundo (f2).
/// Cada celda entera tiene un punto aleatorio; se revisan las 27 celdas vecinas.
/// f2-f1 vale ~0 sobre los bordes entre celdas: sirve para grietas y venas de lava.
pub fn worley3(p: Vec3) -> (f32, f32) {
    let (cx, cy, cz) = (p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32);
    let (mut f1, mut f2) = (f32::MAX, f32::MAX);
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (x, y, z) = (cx + dx, cy + dy, cz + dz);
                let h = hash3(x, y, z);
                // De un solo hash sacamos 3 desplazamientos de 10 bits.
                let o = vec3((h & 1023) as f32, ((h >> 10) & 1023) as f32, ((h >> 20) & 1023) as f32) * (1.0 / 1023.0);
                let q = vec3(x as f32, y as f32, z as f32) + o - p;
                let d = q.dot(q);
                if d < f1 { f2 = f1; f1 = d; } else if d < f2 { f2 = d; }
            }
        }
    }
    (f1.sqrt(), f2.sqrt())
}

/// Value noise 2D periódico (la cuadrícula "da la vuelta" cada `periodo` celdas): para texturas que repiten sin costura.
pub fn value2_periodic(x: f32, y: f32, px: i32, py: i32) -> f32 {
    let (ix, iy) = (x.floor(), y.floor());
    let (fx, fy) = (fade(x - ix), fade(y - iy));
    let (ix, iy) = (ix as i32, iy as i32);
    let v = |dx: i32, dy: i32| hash_cell((ix + dx).rem_euclid(px), (iy + dy).rem_euclid(py), 7);
    lerp(lerp(v(0, 0), v(1, 0), fx), lerp(v(0, 1), v(1, 1), fx), fy)
}
