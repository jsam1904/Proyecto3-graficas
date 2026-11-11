//! Objetos de superficie (árboles, vaporizadores, picos, rascacielos): Lambert con dos colores según la
//! altura relativa del modelo (y en [0,1] tras normalizar) y un oscurecimiento hacia la base (oclusión falsa).

use super::*;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let y = f.local_pos.y; // el modelo está normalizado: base en 0, tope en 1
    let color = u.color2.lerp(u.base_color, smoothstep(u.param - 0.02, u.param + 0.02, y));
    let ao = 0.55 + 0.45 * smoothstep(0.0, 0.5, y);
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    color * ao * lambert(n, l, 0.07)
}
