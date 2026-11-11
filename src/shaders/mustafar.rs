//! Mustafar: volcánico y ANIMADO. Basalto oscuro con grietas de lava (bordes de celdas de Worley)
//! que fluyen y pulsan en el tiempo. La lava es emisiva: brilla incluso en el lado nocturno.

use super::*;
use crate::config;
use crate::noise::{fbm, value3, worley3};
use crate::planet::altura_norm;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let cfg = &config::PLANETAS[u.planet];
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let t = u.time;

    // Roca: basalto muy oscuro con variación.
    let roca = vec3(0.09, 0.075, 0.07) * (0.5 + 1.1 * value3(d * 28.0)) * (0.7 + 0.6 * fbm(d * 9.0, 3).abs());
    let difuso = lambert(n, l, 0.04);

    // Grietas: Worley con la coordenada desplazada lentamente (flujo) por un ruido que cambia con el tiempo.
    let flujo = vec3(0.0, t * 0.03, 0.0) + vec3(fbm(d * 4.0 + vec3(0.0, t * 0.05, 0.0), 2), 0.0, 0.0) * 0.35;
    let (f1, f2) = worley3(d * 6.5 + flujo);
    let vena = 1.0 - smoothstep(0.0, 0.07, f2 - f1);
    // Lagos de magma en los valles (altura baja del relieve).
    let lago = smoothstep(0.16, 0.06, altura_norm(cfg, d));
    let lava = (vena + lago).min(1.0);

    // Pulso: el brillo oscila en el tiempo, con fase distinta por región.
    let pulso = 0.65 + 0.35 * (t * config::LAVA_PULSE_SPEED + 6.0 * value3(d * 4.0)).sin();
    let brillo = lava * pulso * config::LAVA_INTENSITY;
    let color_lava = vec3(0.85, 0.12, 0.02).lerp(vec3(1.0, 0.8, 0.25), (brillo * 0.9).min(1.0));

    // Emisivo: se suma sin multiplicar por la luz.
    roca * difuso * (1.0 - lava) + color_lava * brillo * 1.5
}
