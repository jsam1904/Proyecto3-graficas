//! Tatooine: desierto color durazno con grandes manchas oscuras púrpura-marrón (el relieve de "mares" visto desde el
//! espacio), nubes blancas arremolinadas y un borde atmosférico azul. Dunas suaves + MAPA DE NORMALES procedural.

use super::*;
use crate::config;
use crate::noise::{fbm, ridged, value3};

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let cfg = &config::PLANETAS[u.planet];
    let d = f.local_pos.normalize();
    let n_geo = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let v = (u.camera_pos - f.world_pos).normalize();

    // Manchas grandes: fBm con domain-warp (bordes irregulares) + umbral.
    let w = vec3(fbm(d * 2.0 + vec3(3.0, 1.0, 7.0), 2), fbm(d * 2.0 + vec3(8.0, 4.0, 2.0), 2), fbm(d * 2.0 + vec3(1.0, 9.0, 5.0), 2));
    let m = 0.5 + 0.5 * fbm(d * 2.3 + w * 0.9, 4);
    let mancha = smoothstep(0.50, 0.58, m);
    // Arena durazno con variación suave; las manchas son púrpura-marrón con vetas más claras.
    let arena = vec3(0.90, 0.72, 0.52).lerp(vec3(0.80, 0.62, 0.44), smoothstep(0.3, 0.7, 0.5 + 0.5 * fbm(d * 5.0 + vec3(2.0, 2.0, 8.0), 3)));
    let oscuro = vec3(0.42, 0.28, 0.31) * (0.8 + 0.4 * value3(d * 38.0)) * (0.9 + 0.2 * ridged(d * 9.0, 2));
    let mut c = arena.lerp(oscuro, mancha * 0.92);
    c = c * (0.94 + 0.10 * ridged(d * vec3(10.0, 5.0, 10.0), 2)) * (0.92 + 0.16 * altura_vertice(f, cfg.amplitud)); // dunas suaves

    let n = match u.normal_map {
        Some(mapa) => aplicar_normal_map(f, mapa, config::NORMAL_MAP_TILING, config::NORMAL_MAP_STRENGTH),
        None => n_geo,
    };
    let luz = lambert(n, l, 0.04);
    let mut col = c * luz;
    // Nubes blancas escasas.
    col = col.lerp(Vec3::splat(lambert(n_geo, l, 0.05)) * 0.95, nubes(d, u.time, 0.38) * 0.85);
    col + atmosfera(n_geo, v, l, vec3(0.45, 0.65, 1.0), 1.3)
}
