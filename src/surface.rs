//! Objetos sobre la superficie de los planetas: carga de modelos, distribución procedural reproducible
//! (misma semilla => mismas posiciones) y matrices de colocación.

use crate::config::{self, SurfaceCfg};
use crate::math::*;
use crate::mesh::Mesh;
use crate::noise::{fbm, hash01, hash_u32};
use crate::obj;
use crate::planet::{altura_norm, Planet};

/// Una instancia: matriz en el espacio rígido del planeta (hija de su transformación) y esfera envolvente
/// en ese mismo espacio (para el descarte por frustum de la Fase 8).
pub struct Instance {
    pub local: Mat4,
    pub centro: Vec3,
    pub radio: f32,
    /// Altura en unidades del mundo (sirve para el LOD).
    pub altura: f32,
}

/// Un tipo de modelo con todas sus instancias en un planeta.
pub struct SurfaceSet {
    pub cfg: &'static SurfaceCfg,
    pub mesh: Mesh,
    /// Versión simplificada (LOD intermedio).
    pub mesh_simple: Mesh,
    pub instancias: Vec<Instance>,
}

/// Número pseudoaleatorio en [0,1) determinista a partir de (semilla, índice, canal).
fn rng(semilla: u32, i: u32, canal: u32) -> f32 { hash01(hash_u32(semilla ^ hash_u32(i.wrapping_mul(0x9E37_79B9) ^ canal))) }

/// Carga el modelo (normalizado a base y=0, altura 1) y genera sus instancias sobre `planet`.
pub fn crear(cfg: &'static SurfaceCfg, planet: &Planet, semilla: u64) -> SurfaceSet {
    let ruta = format!("{}/{}", config::SURFACE_DIR, cfg.archivo);
    let mut mesh = obj::load(&ruta).unwrap_or_else(|e| panic!("no se pudo cargar {ruta}: {e}"));
    mesh.normalize_base_height();
    let instancias = distribuir(cfg, &mesh, planet, semilla);
    let mesh_simple = mesh.simplify_clustering(config::SIMPLIFY_CELL);
    SurfaceSet { cfg, mesh, mesh_simple, instancias }
}

/// Distribución procedural:
///  1. Candidatos: puntos de espiral de Fibonacci (cubren la esfera casi uniformemente) con una sacudida aleatoria.
///  2. Se recorren en orden pseudoaleatorio (si no, al llegar al máximo solo quedaría un hemisferio lleno).
///  3. Aceptación/rechazo con fBm: donde el ruido es alto la probabilidad sube => agrupaciones (bosques, barrios).
///  4. Reglas: solo en tierra (no océano) y separación mínima proporcional a la altura de las instancias.
fn distribuir(cfg: &SurfaceCfg, mesh: &Mesh, planet: &Planet, semilla: u64) -> Vec<Instance> {
    let s = (semilla as u32) ^ ((semilla >> 32) as u32);
    let pc = planet.cfg;
    let n_cand = cfg.cantidad * 14;
    let paso = (4.0 * std::f32::consts::PI / n_cand as f32).sqrt(); // separación angular entre candidatos
    let oro = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    let desfase = vec3(rng(s, 0, 101), rng(s, 0, 102), rng(s, 0, 103)) * 50.0; // el ruido cambia con la semilla

    let mut orden: Vec<u32> = (0..n_cand as u32).collect();
    orden.sort_by_key(|&i| hash_u32(s ^ hash_u32(i ^ 0xABCD)));

    let mut aceptadas: Vec<(Vec3, f32)> = Vec::new(); // (dirección local, altura)
    let mut out = Vec::new();
    for &i in &orden {
        if out.len() >= cfg.cantidad { break; }
        // Punto de Fibonacci i + sacudida.
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / n_cand as f32;
        let r = (1.0 - y * y).sqrt();
        let a = oro * i as f32;
        let sac = vec3(rng(s, i, 1) - 0.5, rng(s, i, 2) - 0.5, rng(s, i, 3) - 0.5) * (paso * 0.9);
        let d = (vec3(r * a.cos(), y, r * a.sin()) + sac).normalize();

        // Solo en tierra firme (por encima del nivel del mar del planeta).
        if pc.nivel_mar > 0.0 && altura_norm(pc, d) < pc.nivel_mar + 0.03 { continue; }
        // Aceptación con ruido: agrupa las instancias.
        let g = 0.5 + 0.5 * fbm(d * cfg.frecuencia + desfase, 3);
        if rng(s, i, 4) > smoothstep(cfg.umbral, cfg.umbral + 0.18, g) { continue; }
        // Altura y separación mínima respecto a las ya aceptadas.
        let h = cfg.altura[0] + (cfg.altura[1] - cfg.altura[0]) * rng(s, i, 5);
        let libre = aceptadas.iter().all(|&(d2, h2)| (d - d2).length() * pc.radio >= cfg.separacion * 0.5 * (h + h2));
        if !libre { continue; }
        aceptadas.push((d, h));

        // Colocación: sobre el terreno (misma función de altura que la malla), "arriba" = normal local, giro aleatorio.
        let pos = planet.surface_point(d) - d * cfg.hundir;
        let arriba = planet.surface_normal(d, h * 0.3);
        let ref_ = if arriba.y.abs() < 0.9 { Vec3::UP } else { vec3(1.0, 0.0, 0.0) };
        let x = arriba.cross(ref_).normalize().rotate_about(arriba, rng(s, i, 6) * std::f32::consts::TAU);
        let z = x.cross(arriba); // (x, arriba, z) es una base derecha
        let local = Mat4::from_axes(x * h, arriba * h, z * h, pos); // escala uniforme = altura
        let centro = local.transform_point(mesh.center);
        out.push(Instance { local, centro, radio: mesh.radius * h, altura: h });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene;

    fn base_esfera() -> Mesh {
        let mut s = obj::load(config::SPHERE_OBJ).unwrap();
        s.normalize_to_unit_sphere();
        s
    }

    /// La base de cada instancia, transformada como hija del planeta, debe quedar a altitud -hundir
    /// sobre la superficie en cualquier instante: ni flota ni se hunde mientras el planeta gira y orbita.
    #[test]
    fn base_pegada_a_la_superficie_mientras_gira() {
        crate::placeholders::asegurar(config::SURFACE_DIR);
        let esfera = base_esfera();
        for cfg in config::SUPERFICIE.iter() {
            let planeta = Planet::new(&config::PLANETAS[cfg.planeta], &esfera);
            let set = crear(cfg, &planeta, 7);
            assert!(!set.instancias.is_empty(), "{} sin instancias", cfg.archivo);
            for t in [0.0, 7.3, 123.4] {
                let rigido = scene::planet_rigid(planeta.cfg, t);
                for inst in set.instancias.iter().take(60) {
                    let base = (rigido * inst.local).transform_point(Vec3::ZERO);
                    let alt = planeta.altitude(t, base);
                    assert!((alt + cfg.hundir).abs() < 0.05, "{}: altitud de la base {alt} != -{}", cfg.archivo, cfg.hundir);
                }
            }
        }
    }

    /// Mismo planeta + misma semilla => mismas posiciones; otra semilla => distintas.
    #[test]
    fn semilla_reproducible() {
        crate::placeholders::asegurar(config::SURFACE_DIR);
        let esfera = base_esfera();
        let cfg = &config::SUPERFICIE[2];
        let planeta = Planet::new(&config::PLANETAS[cfg.planeta], &esfera);
        let pos = |s| crear(cfg, &planeta, s).instancias.iter().map(|i| (i.centro.x, i.centro.y, i.centro.z)).collect::<Vec<_>>();
        assert_eq!(pos(5), pos(5));
        assert_ne!(pos(5), pos(6));
    }

    /// El "arriba" de la instancia coincide con la normal del terreno y no hay árboles en el océano.
    #[test]
    fn orientacion_y_tierra_firme() {
        crate::placeholders::asegurar(config::SURFACE_DIR);
        let esfera = base_esfera();
        let cfg = &config::SUPERFICIE[2]; // árboles de Endor (tiene océano)
        let planeta = Planet::new(&config::PLANETAS[cfg.planeta], &esfera);
        let set = crear(cfg, &planeta, 3);
        for inst in &set.instancias {
            let d = inst.local.transform_point(Vec3::ZERO).normalize();
            assert!(altura_norm(planeta.cfg, d) >= planeta.cfg.nivel_mar, "árbol en el océano");
            let arriba = inst.local.transform_dir(Vec3::UP).normalize();
            assert!(arriba.dot(d) > 0.8, "árbol demasiado inclinado respecto a la vertical local"); // sigue la normal, que se desvía poco de la radial
        }
    }
}
