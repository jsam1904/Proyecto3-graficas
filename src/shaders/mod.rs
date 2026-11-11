//! Shaders de fragmentos: funciones puras fn(&FragmentInput, &Uniforms) -> Color.
//! Un archivo por shader. Cada planeta tiene el suyo (ver `para_planeta`).

use crate::config::ShaderKind;
use crate::math::*;
use crate::texture::Texture;

pub mod coruscant;
pub mod deathstar;
pub mod endor;
pub mod flat;
pub mod hoth;
pub mod mustafar;
pub mod objeto;
pub mod sun;
pub mod tatooine;

/// Color lineal RGB en 0..1.
pub type Color = Vec3;

/// Lo que recibe el shader por cada píxel (atributos ya interpolados con corrección de perspectiva).
pub struct FragmentInput {
    pub world_pos: Vec3,
    /// Posición en espacio del objeto (para ruido 3D sin costuras de UV). Gira junto con el planeta.
    pub local_pos: Vec3,
    pub normal: Vec3,
    /// Tangente (mundo) para el mapeo de normales.
    pub tangent: Vec3,
    pub uv: Vec2,
}

/// Datos constantes durante el dibujo de un objeto.
pub struct Uniforms<'a> {
    pub time: f32,
    pub light_pos: Vec3,
    pub camera_pos: Vec3,
    pub base_color: Color,
    /// Índice del planeta en config::PLANETAS (para consultar su relieve).
    pub planet: usize,
    pub normal_map: Option<&'a Texture>,
    /// Color secundario y parámetro libre (usados por el shader de objetos de superficie).
    pub color2: Color,
    pub param: f32,
}

pub type Shader = fn(&FragmentInput, &Uniforms) -> Color;

pub fn para_planeta(k: ShaderKind) -> Shader {
    match k {
        ShaderKind::Tatooine => tatooine::shade,
        ShaderKind::Hoth => hoth::shade,
        ShaderKind::Endor => endor::shade,
        ShaderKind::Mustafar => mustafar::shade,
        ShaderKind::Coruscant => coruscant::shade,
        ShaderKind::DeathStar => deathstar::shade,
    }
}

/// Color 0..1 -> 0x00RRGGBB.
pub fn to_u32(c: Color) -> u32 {
    let q = |v: f32| (clamp(v, 0.0, 1.0) * 255.0 + 0.5) as u32;
    (q(c.x) << 16) | (q(c.y) << 8) | q(c.z)
}

// ---- Ayudas comunes de iluminación ----

/// Difuso de Lambert con ambiente mínimo.
pub fn lambert(n: Vec3, l: Vec3, ambiente: f32) -> f32 { ambiente + (1.0 - ambiente) * n.dot(l).max(0.0) }

/// Especular de Blinn-Phong.
pub fn blinn(n: Vec3, l: Vec3, v: Vec3, brillo: f32) -> f32 { n.dot((l + v).normalize()).max(0.0).powf(brillo) }

/// Perturba la normal con un mapa de normales (espacio tangente) usando la base (T, B, N) del fragmento.
pub fn aplicar_normal_map(f: &FragmentInput, mapa: &Texture, escala_uv: f32, fuerza: f32) -> Vec3 {
    let n = f.normal.normalize();
    let t = (f.tangent - n * n.dot(f.tangent)).normalize();
    let b = n.cross(t);
    let m = mapa.sample(f.uv.x * escala_uv, f.uv.y * escala_uv); // componentes en -1..1
    // fuerza escala solo la parte inclinada (x,y); z se mantiene para no invertir la normal.
    (t * (m.x * fuerza) + b * (m.y * fuerza) + n * m.z).normalize()
}

/// Brillo atmosférico en el borde del disco (dispersión de la luz): más intenso de canto y en el lado iluminado.
pub fn atmosfera(n: Vec3, v: Vec3, l: Vec3, color: Color, fuerza: f32) -> Color {
    let borde = (1.0 - n.dot(v).max(0.0)).powi(3);
    let dia = 0.2 + 0.8 * smoothstep(-0.2, 0.5, n.dot(l));
    color * (borde * dia * fuerza)
}

/// Altura normalizada (0..1) del relieve en este fragmento, leída de la propia malla: |local| = 1 + amplitud·altura.
/// Es gratis (sin ruido por píxel) y coincide con la geometría. Para decisiones finas (costas) se usa planet::altura_norm.
pub fn altura_vertice(f: &FragmentInput, amplitud: f32) -> f32 { clamp((f.local_pos.length() - 1.0) / amplitud.max(1e-4), 0.0, 1.0) }

/// Nubes arremolinadas (0..1): fBm estirado cuya coordenada se deforma con otro fBm (domain warp) y deriva con el tiempo.
/// El warp es lo que crea los remolinos y filamentos en vez de manchas redondas.
pub fn nubes(d: Vec3, t: f32, cobertura: f32) -> f32 {
    let deriva = t * crate::config::CLOUD_SPEED;
    let w = vec3(
        crate::noise::fbm(d * 2.4 + vec3(1.0, deriva, 4.0), 2),
        crate::noise::fbm(d * 2.4 + vec3(7.0, 2.0, -deriva), 2),
        crate::noise::fbm(d * 2.4 + vec3(deriva, 9.0, 3.0), 2),
    );
    let n = 0.5 + 0.5 * crate::noise::fbm(d * vec3(3.2, 5.5, 3.2) + w * 1.7, 4);
    smoothstep(1.0 - cobertura, 1.0 - cobertura + 0.22, n)
}
