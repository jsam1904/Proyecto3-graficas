//! Descarte de objetos antes de rasterizar: frustum (esfera envolvente vs 6 planos) y oclusión por esfera
//! (horizonte de un planeta, o un planeta detrás del Sol).

use crate::math::*;

pub struct Frustum { planos: [Vec4; 6] }

impl Frustum {
    /// Extrae los 6 planos de la matriz vista*proyección (método de Gribb-Hartmann).
    /// En espacio clip un punto está dentro si -w <= x,y,z <= w; cada desigualdad es "fila3 ± fila_i >= 0",
    /// que es un plano ax+by+cz+d >= 0 en coordenadas del MUNDO.
    pub fn new(vp: &Mat4) -> Frustum {
        let m = &vp.m;
        let fila = |i: usize| vec4(m[i][0], m[i][1], m[i][2], m[i][3]);
        let (r0, r1, r2, r3) = (fila(0), fila(1), fila(2), fila(3));
        let suma = |a: Vec4, b: Vec4, s: f32| vec4(a.x + s * b.x, a.y + s * b.y, a.z + s * b.z, a.w + s * b.w);
        let mut planos = [suma(r3, r0, 1.0), suma(r3, r0, -1.0), suma(r3, r1, 1.0), suma(r3, r1, -1.0), suma(r3, r2, 1.0), suma(r3, r2, -1.0)];
        for p in planos.iter_mut() { // normalizar para que la distancia esté en unidades del mundo
            let l = p.xyz().length();
            *p = vec4(p.x / l, p.y / l, p.z / l, p.w / l);
        }
        Frustum { planos }
    }

    /// false solo si la esfera está completamente fuera de algún plano.
    pub fn sphere_visible(&self, c: Vec3, r: f32) -> bool {
        self.planos.iter().all(|p| p.x * c.x + p.y * c.y + p.z * c.z + p.w >= -r)
    }
}

/// ¿El segmento ojo -> `objetivo` atraviesa la esfera (centro, radio) antes de llegar? Si sí, el objetivo está tapado.
/// Si el ojo está dentro de la esfera devuelve false (no se puede afirmar nada).
pub fn tapado_por_esfera(ojo: Vec3, objetivo: Vec3, centro: Vec3, radio: f32) -> bool {
    if radio <= 0.0 { return false; }
    let d = objetivo - ojo;
    let f = ojo - centro;
    let (a, b, c) = (d.dot(d), 2.0 * f.dot(d), f.dot(f) - radio * radio);
    if c < 0.0 || a < 1e-12 { return false; }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 { return false; }
    let t1 = (-b - disc.sqrt()) / (2.0 * a); // primer cruce con la esfera
    t1 > 0.0 && t1 < 1.0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frustum_y_oclusion() {
        let proj = Mat4::perspective(60f32.to_radians(), 16.0 / 9.0, 0.1, 5000.0);
        let vp = proj * Mat4::look_at(Vec3::ZERO, vec3(0.0, 0.0, -1.0), Vec3::UP);
        let f = Frustum::new(&vp);
        assert!(f.sphere_visible(vec3(0.0, 0.0, -50.0), 1.0));      // delante
        assert!(!f.sphere_visible(vec3(0.0, 0.0, 50.0), 1.0));      // detrás
        assert!(!f.sphere_visible(vec3(500.0, 0.0, -50.0), 1.0));   // muy a la derecha
        assert!(f.sphere_visible(vec3(500.0, 0.0, -50.0), 600.0));  // pero una esfera enorme sí lo toca
        assert!(!f.sphere_visible(vec3(0.0, 0.0, -6000.0), 1.0));   // más allá del far
        // Oclusión: una esfera de radio 10 en z=-50 tapa un punto en z=-100, no uno a un lado ni uno delante.
        assert!(tapado_por_esfera(Vec3::ZERO, vec3(0.0, 0.0, -100.0), vec3(0.0, 0.0, -50.0), 10.0));
        assert!(!tapado_por_esfera(Vec3::ZERO, vec3(40.0, 0.0, -100.0), vec3(0.0, 0.0, -50.0), 10.0));
        assert!(!tapado_por_esfera(Vec3::ZERO, vec3(0.0, 0.0, -20.0), vec3(0.0, 0.0, -50.0), 10.0));
    }
}
