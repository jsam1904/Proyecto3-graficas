//! Endor: luna boscosa. Biomas por fBm (océano / playa / claros / bosque denso) usando la MISMA altura que
//! deforma la malla (por eso el océano es plano), más una capa de nubes que se desplaza con el tiempo.

use super::*;
use crate::config;
use crate::noise::{fbm, value3};
use crate::planet::altura_norm;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let cfg = &config::PLANETAS[u.planet];
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let v = (u.camera_pos - f.world_pos).normalize();
    let h = altura_norm(cfg, d);

    let mut especular = 0.0;
    let c = if h <= cfg.nivel_mar + 0.002 {
        // Océano: azul más oscuro hacia el centro de la zona plana + brillo especular.
        especular = blinn(n, l, v, 90.0) * 0.8;
        vec3(0.05, 0.18, 0.35).lerp(vec3(0.08, 0.3, 0.45), value3(d * 20.0))
    } else {
        let b = 0.5 + 0.5 * fbm(d * 6.0 + vec3(1.0, 8.0, 3.0), 3); // bioma
        let veg = 0.85 + 0.3 * value3(d * 70.0); // variación fina del follaje
        let bosque = vec3(0.05, 0.24, 0.08) * veg;
        let claro = vec3(0.30, 0.45, 0.14) * veg;
        let playa = vec3(0.66, 0.58, 0.36);
        let tierra = claro.lerp(bosque, smoothstep(0.42, 0.58, b));
        tierra.lerp(playa, smoothstep(cfg.nivel_mar + 0.06, cfg.nivel_mar, h))
    };
    let mut col = c * lambert(n, l, 0.05) + Vec3::splat(especular);

    // Nubes: fBm que se desplaza con el tiempo; blanco iluminado por el Sol.
    let w = vec3(u.time * config::CLOUD_SPEED, 0.0, 0.0);
    let nube = smoothstep(1.0 - config::CLOUD_COVER, 1.0 - config::CLOUD_COVER + 0.2, 0.5 + 0.5 * fbm(d * 4.0 + w + vec3(9.0, 2.0, 5.0), 4));
    col = col.lerp(Vec3::splat(1.0) * lambert(n, l, 0.06), nube * 0.9);
    col
}
