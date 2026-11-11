//! Hoth: planeta helado. Hielo blanco-azulado con sombreado suave, grietas y vetas de agua azul intenso (líneas donde
//! un fBm deformado vale 0: sinuosas y ramificadas), estrías finas y roca oscura en las cumbres; terminador muy oscuro
//! y borde azul. Destellos de cristales (Blinn-Phong con ruido).

use super::*;
use crate::config;
use crate::noise::{fbm, value3};

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let cfg = &config::PLANETAS[u.planet];
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let v = (u.camera_pos - f.world_pos).normalize();

    // Hielo: blanco con tonos azul-grisáceos de gran escala.
    let tono = 0.5 + 0.5 * fbm(d * 3.0 + vec3(3.0, 1.0, 7.0), 4);
    let mut c = vec3(0.90, 0.95, 0.98).lerp(vec3(0.66, 0.78, 0.90), smoothstep(0.35, 0.8, tono) * 0.75);
    // Vetas de agua/grietas: concentradas en una región (máscara) y dibujadas por las curvas |fBm| ≈ 0 con domain-warp.
    let w = vec3(fbm(d * 3.5 + vec3(2.0, 6.0, 1.0), 2), fbm(d * 3.5 + vec3(9.0, 3.0, 4.0), 2), fbm(d * 3.5 + vec3(5.0, 1.0, 8.0), 2));
    let region = smoothstep(0.30, 0.62, 0.5 + 0.5 * fbm(d * 1.7 + vec3(4.0, 8.0, 2.0), 3));
    let vena = 1.0 - smoothstep(0.0, 0.07, fbm(d * 5.0 + w * 0.9, 4).abs());
    let fina = 1.0 - smoothstep(0.0, 0.04, fbm(d * 13.0 + w * 0.5, 3).abs());
    c = c.lerp(vec3(0.16, 0.50, 0.90), (vena * region * 0.85 + fina * region * 0.35).min(0.9));
    // Estrías finas alargadas y roca en las cumbres.
    c = c * (0.94 + 0.08 * value3(d * vec3(55.0, 10.0, 55.0)));
    let roca = smoothstep(0.55, 0.80, altura_vertice(f, cfg.amplitud));
    c = c.lerp(vec3(0.36, 0.40, 0.48), roca * 0.6);

    let ambiente = vec3(0.025, 0.045, 0.10); // la sombra es azul oscura, no negra
    let difuso = lambert(n, l, 0.0);
    let cristal = smoothstep(0.65, 1.0, value3(d * 130.0));
    let brillo = blinn(n, l, v, 50.0) * (0.15 + 1.2 * cristal) * (1.0 - 0.7 * roca) * config::ICE_GLINT;
    c * difuso + ambiente + Vec3::splat(brillo) + atmosfera(n, v, l, vec3(0.55, 0.75, 1.0), 0.9 * config::ICE_RIM)
}
