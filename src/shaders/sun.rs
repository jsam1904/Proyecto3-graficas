//! Sol: emisivo (no recibe luz). Granulación animada = fBm con domain-warp que evoluciona en el tiempo,
//! manchas oscuras donde el ruido es bajo y oscurecimiento de limbo hacia el borde del disco.

use super::*;
use crate::config;
use crate::noise::fbm;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let d = f.local_pos.normalize();
    let t = u.time * config::SUN_ANIM_SPEED;
    let p = d * 3.0;
    // Domain warp: antes de evaluar el ruido principal, se desplaza la coordenada con otro ruido.
    // Como el desplazamiento cambia con el tiempo, los granos "hierven" en vez de solo deslizarse.
    let q = vec3(fbm(p + vec3(0.0, 0.0, t), 2), fbm(p + vec3(5.2, 1.3, -t), 2), fbm(p + vec3(-t, 9.1, 2.7), 2));
    let g = 0.5 + 0.5 * fbm(p * 1.7 + q * 1.6 + vec3(0.0, t * 0.7, 0.0), 3);
    let mut c = vec3(0.95, 0.30, 0.05).lerp(vec3(1.0, 0.92, 0.55), smoothstep(0.25, 0.8, g));
    c = c * (1.0 - 0.65 * smoothstep(0.32, 0.2, g)); // manchas solares

    // Limbo: mu = coseno entre la normal y la visual (1 al centro, 0 al borde).
    let v = (u.camera_pos - f.world_pos).normalize();
    let mu = f.normal.normalize().dot(v).max(0.0);
    let limbo = 0.4 + 0.6 * mu.sqrt();
    // Más rojizo hacia el borde (la luz sale de capas más frías): el verde y el azul caen más rápido que el rojo.
    c * vec3(limbo, limbo.powf(1.3), limbo.powf(1.7)) * 1.15
}
