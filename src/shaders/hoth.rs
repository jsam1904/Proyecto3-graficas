//! Hoth: hielo y nieve. Grietas glaciares con ruido de Worley (f2-f1 ≈ 0 en los bordes de celda),
//! parches de hielo azul, destellos especulares (brillos que centellean) y borde azul tipo Fresnel.

use super::*;
use crate::config;
use crate::noise::{fbm, value3, worley3};

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let v = (u.camera_pos - f.world_pos).normalize();

    // Nieve con parches de hielo azulado.
    let hielo = smoothstep(0.1, 0.6, 0.5 + 0.5 * fbm(d * 4.0 + vec3(3.0, 1.0, 7.0), 3));
    let mut c = vec3(0.93, 0.96, 1.0).lerp(vec3(0.55, 0.75, 0.95), hielo * 0.8);
    // Grietas: bordes de celdas de Worley.
    let (f1, f2) = worley3(d * 9.0);
    let grieta = 1.0 - smoothstep(0.0, 0.07, f2 - f1);
    c = c.lerp(vec3(0.18, 0.32, 0.55), grieta * 0.85);

    let difuso = lambert(n, l, 0.07);
    // Destellos: Blinn-Phong modulado por un ruido de alta frecuencia (cristales sueltos).
    let cristal = smoothstep(0.65, 1.0, value3(d * 130.0));
    let brillo = blinn(n, l, v, 50.0) * (0.25 + 1.5 * cristal) * (1.0 - 0.8 * grieta) * config::ICE_GLINT;
    // Fresnel: más intenso al mirar de canto; se atenúa en el lado oscuro.
    let rim = (1.0 - n.dot(v).max(0.0)).powi(3) * config::ICE_RIM * (0.25 + 0.75 * n.dot(l).max(0.0));
    c * difuso + Vec3::splat(brillo) + vec3(0.35, 0.65, 1.0) * rim
}
