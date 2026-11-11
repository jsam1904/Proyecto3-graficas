//! Texturas procedurales (sin imágenes externas): mapa de normales generado desde un campo de alturas.

use crate::math::*;
use crate::noise;

pub struct Texture { pub w: usize, pub h: usize, data: Vec<Vec3> }

impl Texture {
    /// Muestreo bilinear con repetición (wrap): u,v fuera de [0,1) vuelven a empezar.
    pub fn sample(&self, u: f32, v: f32) -> Vec3 {
        let (x, y) = (u * self.w as f32 - 0.5, v * self.h as f32 - 0.5);
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let at = |i: i32, j: i32| self.data[(j.rem_euclid(self.h as i32) as usize) * self.w + i.rem_euclid(self.w as i32) as usize];
        let (i, j) = (x0 as i32, y0 as i32);
        at(i, j).lerp(at(i + 1, j), fx).lerp(at(i, j + 1).lerp(at(i + 1, j + 1), fx), fy)
    }

    /// Mapa de normales (espacio tangente, componentes en -1..1) de ondulaciones tipo arena/duna.
    /// 1) altura = suma de octavas de value noise periódico, alargada en una dirección (rizos);
    /// 2) normal = (-dh/dx, -dh/dy, 1) normalizada, con diferencias centrales (con wrap).
    pub fn ripple_normal_map(size: usize, strength: f32) -> Texture {
        let n = size as i32;
        let altura = |x: i32, y: i32| -> f32 {
            let (s, t) = (x.rem_euclid(n) as f32 / size as f32, y.rem_euclid(n) as f32 / size as f32);
            let mut h = 0.0;
            let mut amp = 1.0;
            for o in 0..4 {
                let p = 4 << o; // el periodo se duplica con cada octava: la suma sigue repitiendo en [0,1)
                h += amp * noise::value2_periodic(s * p as f32 * 2.0, t * p as f32 * 0.5, p * 2, (p / 2).max(1));
                amp *= 0.5;
            }
            h
        };
        let mut data = Vec::with_capacity(size * size);
        for y in 0..n {
            for x in 0..n {
                let dx = altura(x + 1, y) - altura(x - 1, y);
                let dy = altura(x, y + 1) - altura(x, y - 1);
                data.push(vec3(-dx * strength, -dy * strength, 1.0).normalize());
            }
        }
        Texture { w: size, h: size, data }
    }
}
