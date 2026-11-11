//! Endor: luna tipo Tierra. Océanos azules (la costa usa la MISMA altura que deforma la malla, por eso el mar es plano),
//! bosques verdes, manchas de tierra, capa de nubes en remolinos que se desplaza con el tiempo y borde atmosférico azul.

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
        especular = blinn(n, l, v, 90.0) * 0.8;
        vec3(0.03, 0.17, 0.36).lerp(vec3(0.10, 0.36, 0.55), value3(d * 14.0) * 0.8) // océano profundo -> aguas claras
    } else {
        let b = 0.5 + 0.5 * fbm(d * 5.0 + vec3(1.0, 8.0, 3.0), 3);
        let follaje = 0.8 + 0.4 * value3(d * 80.0);
        let bosque = vec3(0.07, 0.30, 0.12) * follaje;
        let claro = vec3(0.30, 0.45, 0.16) * follaje;
        let tierra = vec3(0.55, 0.48, 0.28);
        let mut t = bosque.lerp(claro, smoothstep(0.52, 0.70, b));
        t = t.lerp(tierra, smoothstep(0.62, 0.85, 0.5 + 0.5 * fbm(d * 7.0 + vec3(5.0, 2.0, 8.0), 3)) * 0.75);
        t.lerp(vec3(0.62, 0.58, 0.36), smoothstep(cfg.nivel_mar + 0.04, cfg.nivel_mar, h)) // orillas
    };
    let mut col = c * lambert(n, l, 0.04) + Vec3::splat(especular);

    // Remolinos de nubes: blancos, iluminados por el Sol.
    let nube = nubes(d, u.time, config::CLOUD_COVER);
    col = col.lerp(Vec3::splat(lambert(n, l, 0.06)) * 1.05, nube * 0.9);
    col + atmosfera(n, v, l, vec3(0.35, 0.60, 1.0), 1.1)
}
