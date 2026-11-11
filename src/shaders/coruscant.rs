//! Coruscant: planeta-ciudad visto desde el espacio. Suelo casi negro con "nodos urbanos" anaranjados: anillos concéntricos
//! alrededor de los puntos característicos de un ruido de Worley (distancia f1) unidos por líneas rectas (los bordes de
//! las celdas, donde f2-f1 ≈ 0). Nubes grisáceas arremolinadas y borde azul. Los nodos brillan siempre y más de noche.

use super::*;
use crate::config;
use crate::noise::{value3, worley3};

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let v = (u.camera_pos - f.world_pos).normalize();

    let suelo = vec3(0.075, 0.052, 0.042) * (0.7 + 0.7 * value3(d * 28.0)) * (0.85 + 0.3 * value3(d * 7.0));
    let (f1, f2) = worley3(d * config::CITY_GRID);

    // Nodo: radio variable por región; anillos concéntricos finos, núcleo brillante y polvo rojizo cerca.
    let rmax = 0.22 + 0.34 * value3(d * 2.3 + vec3(5.0, 1.0, 3.0));
    let dentro = smoothstep(rmax, rmax * 0.75, f1);
    let anillos = smoothstep(0.09, 0.0, ((f1 * 9.0).fract() - 0.5).abs()) * dentro; // anillos finos
    let nucleo = smoothstep(0.045, 0.0, f1);
    // Líneas rectas entre nodos (bordes de Voronoi), discontinuas.
    let corte = if value3(d * 26.0) > 0.38 { 1.0 } else { 0.0 };
    let linea = (1.0 - smoothstep(0.0, 0.014, f2 - f1)) * corte;
    let polvo = smoothstep(0.62, 0.86, value3(d * 70.0)) * dentro;

    let naranja = vec3(1.0, 0.56, 0.22);
    let noche = smoothstep(0.1, -0.2, n.dot(l));
    let luz = naranja * ((anillos * 1.3 + nucleo * 1.6 + linea * 1.0) * (0.8 + 0.6 * noche)) * config::CITY_LIGHT_INTENSITY
        + vec3(0.55, 0.12, 0.05) * polvo * 0.5;

    let dia = suelo * lambert(n, l, 0.06);
    let mut col = dia + luz;
    // Nubes grisáceo-lila, arremolinadas.
    col = col.lerp(vec3(0.72, 0.72, 0.82) * lambert(n, l, 0.04), nubes(d, u.time, 0.20) * 0.75);
    col + atmosfera(n, v, l, vec3(0.55, 0.70, 1.0), 1.2)
}
