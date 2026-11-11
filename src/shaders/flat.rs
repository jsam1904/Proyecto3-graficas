//! Shader básico: Lambert con luz puntual + ambiente mínimo (para comparar y para la nave).

use super::*;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    u.base_color * lambert(n, l, 0.12)
}
