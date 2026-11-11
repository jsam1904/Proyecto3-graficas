//! Mustafar: planeta de corteza roja. Rojo carmesí con grandes manchas oscuras (basalto), ceniza clara hacia el polo
//! norte, venas y destellos de lava amarilla que pulsan en el tiempo (ANIMADO) y brillo propio: se ve rojo incluso en el
//! lado nocturno. Borde rojizo de bruma volcánica.

use super::*;
use crate::config;
use crate::noise::{fbm, value3};

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let cfg = &config::PLANETAS[u.planet];
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let v = (u.camera_pos - f.world_pos).normalize();
    let t = u.time;

    // Corteza: rojo con manchas oscuras de bordes irregulares (domain-warp).
    let w = vec3(fbm(d * 2.5 + vec3(2.0, 5.0, 1.0), 2), fbm(d * 2.5 + vec3(8.0, 1.0, 6.0), 2), fbm(d * 2.5 + vec3(4.0, 7.0, 3.0), 2));
    let m = 0.5 + 0.5 * fbm(d * 3.0 + w * 1.0, 5);
    let mut c = vec3(0.66, 0.10, 0.045).lerp(vec3(0.12, 0.028, 0.028), smoothstep(0.44, 0.58, m));
    c = c * (0.75 + 0.5 * value3(d * 34.0));
    // Ceniza clara hacia el polo norte.
    let ceniza = smoothstep(0.30, 0.62, d.y + 0.35 * fbm(d * 3.0 + vec3(6.0, 2.0, 9.0), 3));
    c = c.lerp(vec3(0.92, 0.78, 0.72), ceniza * 0.9);

    // Lava: venas finas (|fBm| ≈ 0, fluyen con el tiempo), lagos en los valles y destellos que pulsan.
    let flujo = vec3(0.0, t * 0.02, t * 0.012);
    let vena = 1.0 - smoothstep(0.0, 0.018, fbm(d * 6.0 + w * 0.6 + flujo, 4).abs());
    let lago = smoothstep(0.14, 0.05, altura_vertice(f, cfg.amplitud));
    let pulso = 0.65 + 0.35 * (t * config::LAVA_PULSE_SPEED + 6.0 * value3(d * 4.0)).sin();
    let lava = ((vena * (1.0 - 0.7 * ceniza) + lago).min(1.0)) * pulso * config::LAVA_INTENSITY;
    let color_lava = vec3(1.0, 0.30, 0.04).lerp(vec3(1.0, 0.72, 0.18), (lava * 0.9).min(1.0));

    // Brillo propio: la corteza se ve rojiza también de noche (35 %); la lava es totalmente emisiva.
    c * (0.35 + 0.65 * lambert(n, l, 0.0)) + color_lava * lava * 1.2 + atmosfera(n, v, l, vec3(1.0, 0.30, 0.10), 1.0)
}
