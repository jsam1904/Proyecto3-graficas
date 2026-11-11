//! Pipeline: espacio modelo -> mundo -> vista -> clip -> NDC -> pantalla, y contadores.
//!
//! Dos etapas por cuadro:
//!  1) `submit_mesh` (hilo principal): etapa de vértices, recorte near, culling de caras traseras -> cola de triángulos.
//!  2) `render`: la pantalla se divide en bandas de filas y varios hilos rasterizan TODOS los triángulos de la cola,
//!     cada uno solo en sus filas. El cuello de botella es el shader por píxel, y así escala con los núcleos.

use crate::config;
use crate::framebuffer::Framebuffer;
use crate::math::*;
use crate::mesh::Mesh;
use crate::raster::{self, ClipVertex, ScreenTri};
use crate::shaders::{Shader, Uniforms};
use std::sync::Mutex;

/// Contadores que mostrará el HUD.
#[derive(Default, Clone, Copy)]
pub struct Stats {
    pub tris_sent: u32,
    /// Descartados: caras traseras o totalmente detrás del near.
    pub tris_culled: u32,
    /// Triángulos que cruzaban el plano near y fueron recortados.
    pub tris_clipped: u32,
    pub tris_raster: u32,
    /// Objetos (Sol, planetas, nave e instancias de superficie): total existente, dibujados,
    /// descartados por frustum/horizonte y omitidos por LOD.
    pub objs_total: u32,
    pub objs_drawn: u32,
    pub objs_culled: u32,
    pub objs_lod_omit: u32,
}

pub struct DrawOpts { pub cull_back: bool, pub wireframe: bool }

/// Shader + datos constantes con los que se dibuja un objeto.
pub struct Material<'a> { pub shader: Shader, pub uniforms: Uniforms<'a> }

pub struct FrameQueue<'a> {
    w: usize,
    h: usize,
    materials: Vec<Material<'a>>,
    tris: Vec<ScreenTri>,
    /// Líneas 2D (wireframe y ejes), se dibujan al final sin profundidad.
    lines: Vec<(f32, f32, f32, f32, u32)>,
    /// Buffer reutilizable de vértices transformados (evita reservar memoria en cada objeto).
    scratch: Vec<ClipVertex>,
}

impl<'a> FrameQueue<'a> {
    pub fn new(w: usize, h: usize) -> Self { Self { w, h, materials: Vec::new(), tris: Vec::new(), lines: Vec::new(), scratch: Vec::new() } }

    pub fn add_material(&mut self, shader: Shader, uniforms: Uniforms<'a>) -> usize {
        self.materials.push(Material { shader, uniforms });
        self.materials.len() - 1
    }

    /// Etapa de geometría de una malla: modelo -> mundo -> clip, recorte near, culling, y encola.
    pub fn submit_mesh(&mut self, mesh: &Mesh, model: &Mat4, view_proj: &Mat4, material: usize, opts: &DrawOpts, stats: &mut Stats) {
        // 1) Etapa de vértices: modelo -> mundo -> clip.
        let mut verts = std::mem::take(&mut self.scratch);
        verts.clear();
        verts.extend(mesh.vertices.iter().map(|v| {
            let world = model.transform_point(v.pos);
            ClipVertex {
                clip: view_proj.mul_vec4(world.extend(1.0)),
                world,
                local: v.pos,
                normal: model.transform_dir(v.normal).normalize(), // válido para rotación + escala uniforme
                tangent: model.transform_dir(v.tangent).normalize(),
                uv: v.uv,
            }
        }));

        // 2) Triángulos: recorte near -> (abanico) -> culling de caras traseras -> cola.
        for t in &mesh.tris {
            stats.tris_sent += 1;
            let tri = [verts[t[0] as usize], verts[t[1] as usize], verts[t[2] as usize]];
            let dentro = tri.iter().filter(|v| v.clip.z + v.clip.w >= 0.0).count();
            if dentro == 0 { stats.tris_culled += 1; continue; } // totalmente detrás del plano near

            // Polígono resultante: el triángulo original (3) o el recortado (3 o 4 vértices).
            let (poly, n) = if dentro == 3 { ([tri[0], tri[1], tri[2], tri[2]], 3) } else { stats.tris_clipped += 1; raster::clip_near(tri) };
            if n < 3 { stats.tris_culled += 1; continue; }

            if opts.wireframe {
                for i in 0..n {
                    let (a, b) = (poly[i].clip, poly[(i + 1) % n].clip);
                    let (sa, sb) = (raster::to_screen(a, self.w, self.h), raster::to_screen(b, self.w, self.h));
                    self.lines.push((sa.0, sa.1, sb.0, sb.1, 0xFFFFFF));
                }
                stats.tris_raster += 1;
                continue;
            }
            // Un cuadrilátero se parte en dos triángulos (abanico). El orden de vértices conserva el sentido de giro.
            let mut dibujado = false;
            for k in 1..n - 1 {
                if let Some(st) = raster::setup_triangle([&poly[0], &poly[k], &poly[k + 1]], self.w, self.h, opts.cull_back, material) {
                    self.tris.push(st);
                    dibujado = true;
                }
            }
            if dibujado { stats.tris_raster += 1; } else { stats.tris_culled += 1; }
        }
        self.scratch = verts;
    }

    /// Rasteriza la cola en el framebuffer, repartiendo bandas de filas entre hilos.
    pub fn render(&self, fb: &mut Framebuffer) {
        let hilos = if config::RENDER_THREADS == 0 { std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) } else { config::RENDER_THREADS };
        let bandas = Mutex::new(fb.split_bands(config::BAND_ROWS));
        let trabajo = || loop {
            // Cada hilo toma la siguiente banda libre (reparto dinámico: las bandas con planeta cuestan más).
            let Some(mut banda) = bandas.lock().unwrap().pop() else { break };
            for tri in &self.tris {
                let m = &self.materials[tri.material];
                raster::fill_triangle(&mut banda, tri, &m.uniforms, m.shader);
            }
        };
        if hilos <= 1 {
            trabajo();
        } else {
            std::thread::scope(|sc| { for _ in 0..hilos { sc.spawn(&trabajo); } });
        }
        for &(x0, y0, x1, y1, c) in &self.lines { raster::draw_line(fb, x0, y0, x1, y1, c); }
    }
}
