//! Tatooine: desierto. Bandas de arena deformadas por fBm, crestas de dunas más claras (misma altura que la malla),
//! grano fino y MAPA DE NORMALES procedural (rizos de arena) con la base tangente (T,B,N).

use super::*;
use crate::config;
use crate::noise::{fbm, value3};
use crate::planet::altura_norm;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let d = f.local_pos.normalize();
    // Bandas latitudinales de dos tonos de arena, con la frontera ondulada por ruido.
    let banda = ((d.y * 9.0 + fbm(d * 3.0, 3) * 2.2).sin()) * 0.5 + 0.5;
    let mut c = vec3(0.88, 0.67, 0.38).lerp(vec3(0.70, 0.45, 0.24), smoothstep(0.3, 0.7, banda));
    // Las crestas de las dunas (altura alta) son más claras; los valles, más oscuros.
    let h = altura_norm(&config::PLANETAS[u.planet], d);
    c = c * (0.78 + 0.5 * h);
    c = c * (0.94 + 0.12 * value3(d * 90.0)); // grano

    // Normal perturbada por el mapa de normales.
    let n = match u.normal_map {
        Some(m) => aplicar_normal_map(f, m, config::NORMAL_MAP_TILING, config::NORMAL_MAP_STRENGTH),
        None => f.normal.normalize(),
    };
    let l = (u.light_pos - f.world_pos).normalize();
    c * lambert(n, l, 0.05)
}
