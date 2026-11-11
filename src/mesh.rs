//! Malla triangular indexada y utilidades (normales, límites).

use crate::math::*;

#[derive(Clone, Copy, Debug)]
pub struct Vertex { pub pos: Vec3, pub normal: Vec3, pub uv: Vec2, pub tangent: Vec3 }

#[derive(Clone, Debug)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub tris: Vec<[u32; 3]>,
    /// Caja envolvente y esfera envolvente (calculadas al cargar, sin números mágicos).
    pub bmin: Vec3,
    pub bmax: Vec3,
    pub center: Vec3,
    pub radius: f32,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, tris: Vec<[u32; 3]>) -> Mesh {
        let mut m = Mesh { vertices, tris, bmin: Vec3::ZERO, bmax: Vec3::ZERO, center: Vec3::ZERO, radius: 0.0 };
        m.update_bounds();
        m
    }

    /// Simplificación por agrupamiento de vértices (vertex clustering): se superpone una cuadrícula de celdas de
    /// tamaño `celda`; todos los vértices de una celda se funden en uno (promedio de posiciones); los triángulos que
    /// quedan con dos vértices iguales (degenerados) o repetidos se eliminan. Sirve como LOD "simple" de cualquier modelo.
    pub fn simplify_clustering(&self, celda: f32) -> Mesh {
        let clave = |p: Vec3| ((p.x / celda).floor() as i32, (p.y / celda).floor() as i32, (p.z / celda).floor() as i32);
        let mut celdas: std::collections::HashMap<(i32, i32, i32), u32> = std::collections::HashMap::new();
        let mut sumas: Vec<(Vec3, f32, Vec2)> = Vec::new();
        let mapa: Vec<u32> = self.vertices.iter().map(|v| {
            let id = *celdas.entry(clave(v.pos)).or_insert_with(|| { sumas.push((Vec3::ZERO, 0.0, Vec2::default())); (sumas.len() - 1) as u32 });
            let s = &mut sumas[id as usize];
            s.0 += v.pos; s.1 += 1.0; s.2 = s.2 + v.uv;
            id
        }).collect();
        let vertices: Vec<Vertex> = sumas.iter().map(|(p, n, uv)| Vertex { pos: *p / *n, normal: Vec3::ZERO, uv: *uv * (1.0 / *n), tangent: Vec3::ZERO }).collect();
        let mut vistos = std::collections::HashSet::new();
        let mut tris = Vec::new();
        for t in &self.tris {
            let tri = [mapa[t[0] as usize], mapa[t[1] as usize], mapa[t[2] as usize]];
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] { continue; }
            let mut k = tri; k.sort();
            if vistos.insert(k) { tris.push(tri); } // sin triángulos duplicados
        }
        let mut m = Mesh::new(vertices, tris);
        m.recompute_normals();
        m
    }

    /// Normaliza un modelo de objeto: base centrada en el origen (y mínimo = 0) y altura total = 1.
    /// Así cualquier modelo (placeholder o propio) se escala igual con "altura objetivo en unidades del mundo".
    pub fn normalize_base_height(&mut self) {
        let h = (self.bmax.y - self.bmin.y).max(1e-6);
        let base = vec3(self.center.x, self.bmin.y, self.center.z);
        for v in self.vertices.iter_mut() { v.pos = (v.pos - base) / h; }
        self.update_bounds();
    }

    /// Recalcula caja y esfera envolvente (centro = centro de la caja).
    pub fn update_bounds(&mut self) {
        if self.vertices.is_empty() { return; }
        let mut lo = self.vertices[0].pos;
        let mut hi = lo;
        for v in &self.vertices { lo = lo.min(v.pos); hi = hi.max(v.pos); }
        self.bmin = lo;
        self.bmax = hi;
        self.center = (lo + hi) * 0.5;
        self.radius = self.vertices.iter().map(|v| (v.pos - self.center).length()).fold(0.0, f32::max);
    }

    /// Normales suaves: suma de normales de cara ponderadas por área (el producto cruz ya la pondera).
    /// Los vértices con la misma posición (duplicados por costuras de UV) se "sueldan" para que compartan normal.
    pub fn recompute_normals(&mut self) {
        let clave = |p: Vec3| ((p.x * 1e4).round() as i64, (p.y * 1e4).round() as i64, (p.z * 1e4).round() as i64);
        let mut acc: std::collections::HashMap<(i64, i64, i64), Vec3> = std::collections::HashMap::new();
        for t in &self.tris {
            let [a, b, c] = [t[0] as usize, t[1] as usize, t[2] as usize];
            let n = (self.vertices[b].pos - self.vertices[a].pos).cross(self.vertices[c].pos - self.vertices[a].pos);
            for i in [a, b, c] { *acc.entry(clave(self.vertices[i].pos)).or_insert(Vec3::ZERO) += n; }
        }
        for v in self.vertices.iter_mut() { v.normal = acc[&clave(v.pos)].normalize(); }
    }

    /// Tangentes por vértice: dirección en la que crece U sobre la superficie, derivada de las posiciones y UV de cada
    /// triángulo (T = (e1·dv2 - e2·dv1)/det) acumulada en los vértices y ortogonalizada contra la normal (Gram-Schmidt).
    /// Junto con la normal forma la base (T, B = N×T, N) del espacio tangente para el mapeo de normales.
    pub fn compute_tangents(&mut self) {
        let mut acc = vec![Vec3::ZERO; self.vertices.len()];
        for t in &self.tris {
            let [a, b, c] = [t[0] as usize, t[1] as usize, t[2] as usize];
            let (va, vb, vc) = (self.vertices[a], self.vertices[b], self.vertices[c]);
            let (e1, e2) = (vb.pos - va.pos, vc.pos - va.pos);
            let (du1, dv1, du2, dv2) = (vb.uv.x - va.uv.x, vb.uv.y - va.uv.y, vc.uv.x - va.uv.x, vc.uv.y - va.uv.y);
            let det = du1 * dv2 - du2 * dv1;
            if det.abs() < 1e-12 { continue; }
            let tan = (e1 * dv2 - e2 * dv1) * (1.0 / det);
            acc[a] += tan; acc[b] += tan; acc[c] += tan;
        }
        for (v, t) in self.vertices.iter_mut().zip(acc) {
            let mut o = t - v.normal * v.normal.dot(t);
            if o.length() < 1e-9 { // sin dato de UV: cualquier perpendicular a la normal
                o = v.normal.cross(if v.normal.x.abs() < 0.9 { vec3(1.0, 0.0, 0.0) } else { Vec3::UP });
            }
            v.tangent = o.normalize();
        }
    }

    /// Subdivisión: cada triángulo se parte en 4 con vértices en los puntos medios de sus aristas
    /// (compartidos entre triángulos vecinos). Los puntos nuevos se proyectan a la esfera unitaria.
    /// Pensada para mallas esféricas de radio 1.
    pub fn subdivide_sphere(&self) -> Mesh {
        let mut vertices = self.vertices.clone();
        let mut medios: std::collections::HashMap<(u32, u32), u32> = std::collections::HashMap::new();
        let mut tris = Vec::with_capacity(self.tris.len() * 4);
        for t in &self.tris {
            let mut m = [0u32; 3]; // m[i] = punto medio de la arista t[i] -> t[i+1]
            for i in 0..3 {
                let (a, b) = (t[i], t[(i + 1) % 3]);
                let key = (a.min(b), a.max(b));
                m[i] = *medios.entry(key).or_insert_with(|| {
                    let (va, vb) = (self.vertices[a as usize], self.vertices[b as usize]);
                    let pos = ((va.pos + vb.pos) * 0.5).normalize();
                    vertices.push(Vertex { pos, normal: pos, uv: (va.uv + vb.uv) * 0.5, tangent: Vec3::ZERO });
                    (vertices.len() - 1) as u32
                });
            }
            tris.push([t[0], m[0], m[2]]);
            tris.push([m[0], t[1], m[1]]);
            tris.push([m[2], m[1], t[2]]);
            tris.push([m[0], m[1], m[2]]);
        }
        Mesh::new(vertices, tris)
    }

    /// Deforma una malla esférica unitaria: cada vértice pasa a dir * f(dir). Recalcula normales y límites.
    pub fn deform_radial(&mut self, f: impl Fn(Vec3) -> f32) {
        for v in self.vertices.iter_mut() {
            let dir = v.pos.normalize();
            v.pos = dir * f(dir);
        }
        self.recompute_normals();
        self.compute_tangents();
        self.update_bounds();
    }

    /// Proyecta cada vértice a la esfera unitaria; normal = posición. (Para sphere.obj, que es un cubo subdividido.)
    pub fn normalize_to_unit_sphere(&mut self) {
        for v in &mut self.vertices {
            v.pos = v.pos.normalize();
            v.normal = v.pos;
        }
        self.update_bounds();
    }
}
