//! Escena: el Sol (origen) y los planetas, con posición y orientación en función del TIEMPO.

use crate::config::{self, PlanetCfg};
use crate::math::*;

/// Cuerpo celeste en un instante: centro, velocidad, radio base y "GM" (para la gravedad).
pub struct Body { pub pos: Vec3, pub vel: Vec3, pub radio: f32, pub gm: f32 }

/// Posición en la órbita circular (plano XZ): ángulo = fase + t · velocidad.
pub fn planet_pos(p: &PlanetCfg, t: f32) -> Vec3 {
    let a = p.fase + t * p.vel_orbita;
    vec3(a.cos() * p.orbita, 0.0, a.sin() * p.orbita)
}


/// Transformación rígida del planeta (sin escala): giro propio -> inclinación axial -> traslación orbital.
/// Los objetos de superficie son hijos de esta transformación: giran y orbitan con el planeta.
pub fn planet_rigid(p: &PlanetCfg, t: f32) -> Mat4 {
    Mat4::translation(planet_pos(p, t)) * Mat4::rotation_z(p.inclinacion_deg.to_radians()) * Mat4::rotation_y(t * p.vel_rotacion)
}

/// Modelo -> mundo: escala (radio) -> giro propio -> inclinación axial -> traslación orbital.
pub fn planet_model(p: &PlanetCfg, t: f32) -> Mat4 { planet_rigid(p, t) * Mat4::scale_uniform(p.radio) }

pub fn sun_model() -> Mat4 { Mat4::scale_uniform(config::SUN_RADIUS) }

/// Todos los cuerpos (Sol primero) en el instante t.
pub fn bodies(t: f32) -> Vec<Body> {
    let mut v = vec![Body { pos: Vec3::ZERO, vel: Vec3::ZERO, radio: config::SUN_RADIUS, gm: config::G * config::SUN_MASS }];
    for p in config::PLANETAS.iter() {
        let a = p.fase + t * p.vel_orbita;
        let vel = vec3(-a.sin(), 0.0, a.cos()) * (p.orbita * p.vel_orbita);
        v.push(Body { pos: planet_pos(p, t), vel, radio: p.radio, gm: config::G * p.masa });
    }
    v
}

/// Aceleración gravitatoria en `p`: suma de g = GM/(r²+ε) hacia el centro de cada cuerpo.
pub fn gravity(p: Vec3, bodies: &[Body]) -> Vec3 {
    let mut a = Vec3::ZERO;
    for b in bodies {
        let d = b.pos - p;
        let r2 = d.dot(d);
        a += d.normalize() * (b.gm / (r2 + config::GRAVITY_EPS));
    }
    a
}
