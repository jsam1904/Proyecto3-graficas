//! Planeta: relieve (función de altura única), mallas por nivel de detalle y consultas
//! de superficie (altitud, colisión). La MISMA función `forma` deforma la malla, ubica los
//! objetos de superficie y sirve para colisionar, así que nada flota ni se hunde.

use crate::config::{self, PlanetCfg, Terreno};
use crate::math::*;
use crate::mesh::Mesh;
use crate::noise;

pub struct Planet {
    pub cfg: &'static PlanetCfg,
    /// Mallas deformadas, de menos a más detalle (nivel 0 = sphere.obj, luego subdivisión 1, 2, 3).
    pub lods: Vec<Mesh>,
}

/// Niveles de detalle de una esfera unitaria: 0 = base, n = n subdivisiones.
pub fn build_lods(base: &Mesh) -> Vec<Mesh> {
    let mut v = vec![base.clone()];
    for _ in 1..config::LOD_LEVELS { let m = v.last().unwrap().subdivide_sphere(); v.push(m); }
    v
}

impl Planet {
    pub fn new(cfg: &'static PlanetCfg, base_sphere: &Mesh) -> Planet {
        let mut lods = build_lods(base_sphere);
        for m in lods.iter_mut() { m.deform_radial(|d| forma(cfg, d)); }
        Planet { cfg, lods }
    }

    /// Posición del centro en el instante t (órbita circular en XZ).
    pub fn pos(&self, t: f32) -> Vec3 { crate::scene::planet_pos(self.cfg, t) }

    /// Velocidad orbital del centro (derivada de la posición).
    pub fn vel(&self, t: f32) -> Vec3 {
        let a = self.cfg.fase + t * self.cfg.vel_orbita;
        vec3(-a.sin(), 0.0, a.cos()) * (self.cfg.orbita * self.cfg.vel_orbita)
    }

    /// Dirección del mundo -> dirección en el espacio del objeto (deshace giro e inclinación).
    pub fn world_to_local(&self, t: f32, d: Vec3) -> Vec3 {
        let inv = Mat4::rotation_y(-t * self.cfg.vel_rotacion) * Mat4::rotation_z(-self.cfg.inclinacion_deg.to_radians());
        inv.transform_dir(d)
    }

    /// Radio de la superficie en la dirección local `d`.
    pub fn surface_radius_local(&self, d: Vec3) -> f32 { self.cfg.radio * forma(self.cfg, d) }

    /// Radio de la superficie bajo un punto del mundo.
    pub fn surface_radius_at(&self, t: f32, p: Vec3) -> f32 {
        let d = (p - self.pos(t)).normalize();
        self.surface_radius_local(self.world_to_local(t, d))
    }

    /// Altitud de un punto sobre la superficie (puede ser negativa si está dentro).
    pub fn altitude(&self, t: f32, p: Vec3) -> f32 { (p - self.pos(t)).length() - self.surface_radius_at(t, p) }

    /// Punto de la superficie (espacio rígido del planeta, unidades del mundo) en la dirección local `d`.
    pub fn surface_point(&self, d: Vec3) -> Vec3 { d * self.surface_radius_local(d) }

    /// Normal de la superficie en la dirección local `d`, por diferencias finitas de la MISMA función de altura:
    /// se toman dos puntos vecinos sobre el terreno y la normal es el producto cruz de las dos aristas.
    /// `paso` (unidades del mundo) debe ser del orden del tamaño del objeto que se apoya ahí.
    pub fn surface_normal(&self, d: Vec3, paso: f32) -> Vec3 {
        let eps = paso / self.cfg.radio;
        let t1 = d.cross(if d.y.abs() < 0.9 { Vec3::UP } else { vec3(1.0, 0.0, 0.0) }).normalize();
        let t2 = d.cross(t1);
        let p0 = self.surface_point(d);
        let pa = self.surface_point((d + t1 * eps).normalize());
        let pb = self.surface_point((d + t2 * eps).normalize());
        let n = (pa - p0).cross(pb - p0).normalize();
        if n.dot(d) < 0.0 { -n } else { n }
    }
}

/// Nivel de LOD para una distancia expresada en radios.
pub fn lod_level(dist_en_radios: f32) -> usize {
    config::LOD_DIST.iter().filter(|&&d| dist_en_radios < d).count()
}

/// Altura normalizada del relieve en la dirección unitaria `d` (espacio del objeto), en [0,1].
/// Es la única definición del terreno: la usan la malla, la colisión, los objetos de superficie y los shaders.
pub fn altura_norm(cfg: &PlanetCfg, d: Vec3) -> f32 {
    let p = d * cfg.frecuencia + Vec3::from(cfg.semilla);
    let n = match cfg.terreno {
        // Dunas: crestas (ridged) mezcladas con ondulación suave.
        Terreno::Dunas => 0.7 * noise::ridged(p, 3) + 0.3 * (0.5 + 0.5 * noise::fbm(p * 0.5, 3)),
        // Montañas: fBm elevado a potencia para picos aislados y valles planos.
        Terreno::Montanas => (0.5 + 0.5 * noise::fbm(p, 5)).powf(2.0),
        Terreno::Colinas => 0.5 + 0.5 * noise::fbm(p, 3),
    };
    n.max(cfg.nivel_mar) // por debajo del nivel del mar el terreno queda plano (océano)
}

/// Multiplicador del radio en la dirección `d`: 1 + amplitud · altura_norm(d).
pub fn forma(cfg: &PlanetCfg, d: Vec3) -> f32 { 1.0 + cfg.amplitud * altura_norm(cfg, d) }
