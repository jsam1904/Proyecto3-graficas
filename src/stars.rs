//! Fondo de estrellas procedural: direcciones fijas generadas por hash, proyectadas como si
//! estuvieran en el infinito (solo importa la rotación de la cámara) y con parpadeo leve.

use crate::camera::Camera;
use crate::config;
use crate::framebuffer::Framebuffer;
use crate::math::*;
use crate::noise::hash01;
use crate::shaders::to_u32;

struct Star { dir: Vec3, brillo: f32, fase: f32, tinte: Vec3 }

pub struct Stars { stars: Vec<Star> }

impl Stars {
    pub fn new() -> Stars {
        let stars = (0..config::STAR_COUNT as u32).map(|i| {
            let h = |k: u32| hash01(config::STAR_SEED.wrapping_add(i.wrapping_mul(8)).wrapping_add(k));
            // Dirección uniforme en la esfera: z uniforme en [-1,1] y ángulo uniforme.
            let z = 2.0 * h(0) - 1.0;
            let a = std::f32::consts::TAU * h(1);
            let r = (1.0 - z * z).sqrt();
            let calido = h(4);
            Star {
                dir: vec3(r * a.cos(), z, r * a.sin()),
                brillo: 0.25 + 0.75 * h(2).powi(3), // pocas brillantes, muchas tenues
                fase: h(3) * std::f32::consts::TAU,
                tinte: vec3(0.8 + 0.2 * calido, 0.85 + 0.1 * (1.0 - calido).min(calido), 1.0 - 0.2 * calido),
            }
        }).collect();
        Stars { stars }
    }

    /// Dibuja solo donde no hay geometría (profundidad = 0). Se llama después de rasterizar.
    pub fn draw(&self, fb: &mut Framebuffer, cam: &Camera, t: f32) {
        let aspect = fb.w as f32 / fb.h as f32;
        let (view, proj) = (cam.view(), cam.projection(aspect));
        for (i, s) in self.stars.iter().enumerate() {
            // Solo rotación: dirección en vista, tratada como punto a distancia unitaria.
            let c = proj.mul_vec4(view.transform_dir(s.dir).extend(1.0));
            if c.w <= 0.0 { continue; }
            let (sx, sy, _) = crate::raster::to_screen(c, fb.w, fb.h);
            if sx < 0.0 || sy < 0.0 || sx >= fb.w as f32 || sy >= fb.h as f32 { continue; }
            let parpadeo = 0.85 + 0.15 * (t * (1.0 + (i % 5) as f32) + s.fase).sin();
            let col = to_u32(s.tinte * (s.brillo * parpadeo));
            let tam = if s.brillo > 0.8 { 2 } else { 1 }; // las más brillantes ocupan 2x2
            for dy in 0..tam {
                for dx in 0..tam {
                    let (x, y) = (sx as usize + dx, sy as usize + dy);
                    if x < fb.w && y < fb.h && fb.depth[y * fb.w + x] == 0.0 { fb.color[y * fb.w + x] = col; }
                }
            }
        }
    }
}
