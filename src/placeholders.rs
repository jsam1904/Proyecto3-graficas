//! Generador de modelos PLACEHOLDER de superficie (formato OBJ con v/vn/f, caras con normal plana).
//! Se escriben en assets/surface/ SOLO si el archivo no existe: para usar tus modelos de Blender basta con
//! reemplazar esos archivos (mismo nombre). Convención: base en y=0 y "arriba" = +Y (el cargador igual los
//! recentra y escala a altura 1, así que cualquier tamaño sirve).

use crate::math::*;
use std::f32::consts::TAU;
use std::fmt::Write;

/// Acumula caras y las escribe como OBJ.
struct ObjWriter { v: String, n: String, f: String, nv: usize, nn: usize }

impl ObjWriter {
    fn new() -> Self { Self { v: String::new(), n: String::new(), f: String::new(), nv: 0, nn: 0 } }

    /// Polígono convexo con vértices dados; `centro` es un punto interior de la pieza: sirve para orientar la
    /// cara hacia afuera (si la normal apunta hacia el centro, se invierte el orden de los vértices).
    fn cara(&mut self, mut p: Vec<Vec3>, centro: Vec3) {
        let mut n = (p[1] - p[0]).cross(p[2] - p[0]);
        let cen = p.iter().fold(Vec3::ZERO, |a, &b| a + b) / p.len() as f32;
        if n.dot(cen - centro) < 0.0 { p.reverse(); n = -n; }
        let n = n.normalize();
        writeln!(self.n, "vn {:.5} {:.5} {:.5}", n.x, n.y, n.z).unwrap();
        self.nn += 1;
        write!(self.f, "f").unwrap();
        for q in &p {
            writeln!(self.v, "v {:.5} {:.5} {:.5}", q.x, q.y, q.z).unwrap();
            self.nv += 1;
            write!(self.f, " {}//{}", self.nv, self.nn).unwrap();
        }
        writeln!(self.f).unwrap();
    }

    /// Prisma/cono truncado de `lados` lados entre y0 (radio r0) e y1 (radio r1) centrado en (cx, cz). r1=0 => cono.
    fn prisma(&mut self, cx: f32, cz: f32, lados: usize, r0: f32, r1: f32, y0: f32, y1: f32) {
        let centro = vec3(cx, (y0 + y1) * 0.5, cz);
        let pt = |i: usize, r: f32, y: f32| { let a = TAU * i as f32 / lados as f32; vec3(cx + r * a.cos(), y, cz + r * a.sin()) };
        for i in 0..lados {
            if r1 > 0.0 {
                self.cara(vec![pt(i, r0, y0), pt(i + 1, r0, y0), pt(i + 1, r1, y1), pt(i, r1, y1)], centro);
            } else {
                self.cara(vec![pt(i, r0, y0), pt(i + 1, r0, y0), vec3(cx, y1, cz)], centro);
            }
        }
        if r1 > 0.0 { self.cara((0..lados).map(|i| pt(i, r1, y1)).collect(), centro); } // tapa superior
    }

    /// Caja alineada a los ejes centrada en (cx, cz), de ancho w (X), fondo d (Z), entre y0 e y1.
    fn caja(&mut self, cx: f32, cz: f32, w: f32, d: f32, y0: f32, y1: f32) {
        let (x0, x1, z0, z1) = (cx - w / 2.0, cx + w / 2.0, cz - d / 2.0, cz + d / 2.0);
        let c = vec3(cx, (y0 + y1) * 0.5, cz);
        let p = |x, y, z| vec3(x, y, z);
        for q in [
            vec![p(x0, y0, z0), p(x1, y0, z0), p(x1, y1, z0), p(x0, y1, z0)],
            vec![p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)],
            vec![p(x0, y0, z0), p(x0, y0, z1), p(x0, y1, z1), p(x0, y1, z0)],
            vec![p(x1, y0, z0), p(x1, y0, z1), p(x1, y1, z1), p(x1, y1, z0)],
            vec![p(x0, y1, z0), p(x1, y1, z0), p(x1, y1, z1), p(x0, y1, z1)],
        ] { self.cara(q, c); }
    }

    fn escribir(&self, path: &str, nombre: &str) -> std::io::Result<()> {
        let txt = format!("# PLACEHOLDER generado por src/placeholders.rs: {nombre}\n# Reemplazar por el modelo propio (mismo nombre de archivo). Base en y=0, +Y arriba.\no {nombre}\n{}{}{}", self.v, self.n, self.f);
        std::fs::write(path, txt)
    }
}

fn arbol() -> ObjWriter {
    let mut w = ObjWriter::new();
    w.prisma(0.0, 0.0, 8, 0.13, 0.08, 0.0, 0.55); // tronco
    w.prisma(0.0, 0.0, 8, 0.55, 0.0, 0.35, 0.78); // copa en tres conos apilados
    w.prisma(0.0, 0.0, 8, 0.42, 0.0, 0.58, 0.93);
    w.prisma(0.0, 0.0, 8, 0.28, 0.0, 0.78, 1.1);
    w
}

fn vaporizador() -> ObjWriter {
    let mut w = ObjWriter::new();
    w.prisma(0.0, 0.0, 10, 0.34, 0.30, 0.0, 0.14); // base
    w.prisma(0.0, 0.0, 8, 0.11, 0.09, 0.14, 0.86); // tubo
    w.caja(0.0, 0.0, 0.44, 0.07, 0.62, 0.74); // aletas condensadoras
    w.caja(0.0, 0.0, 0.07, 0.44, 0.50, 0.62);
    w.prisma(0.0, 0.0, 8, 0.15, 0.0, 0.86, 1.0); // remate
    w
}

fn pico_hielo() -> ObjWriter {
    let mut w = ObjWriter::new();
    w.prisma(0.0, 0.0, 5, 0.26, 0.0, 0.0, 1.0);
    w.prisma(0.30, 0.05, 5, 0.18, 0.0, 0.0, 0.62);
    w.prisma(-0.24, 0.17, 5, 0.15, 0.0, 0.0, 0.48);
    w
}

fn rascacielos() -> ObjWriter {
    let mut w = ObjWriter::new();
    w.caja(0.0, 0.0, 0.6, 0.6, 0.0, 0.68);
    w.caja(0.0, 0.0, 0.44, 0.44, 0.68, 0.88);
    w.caja(0.0, 0.0, 0.22, 0.22, 0.88, 1.0);
    w.prisma(0.0, 0.0, 4, 0.02, 0.01, 1.0, 1.2); // antena
    w
}

/// Crea los archivos que falten. Devuelve cuántos generó.
pub fn asegurar(dir: &str) -> usize {
    std::fs::create_dir_all(dir).ok();
    let mut n = 0;
    for (nombre, g) in [("arbol", arbol as fn() -> ObjWriter), ("vaporizador", vaporizador), ("pico_hielo", pico_hielo), ("rascacielos", rascacielos)] {
        let path = format!("{dir}/{nombre}.obj");
        if !std::path::Path::new(&path).exists() {
            g().escribir(&path, nombre).expect("no se pudo escribir el placeholder");
            n += 1;
        }
    }
    n
}
