//! Estado de la aplicación: actualización (física, tiempo) y render de un cuadro.

use crate::camera::{Camera, FollowCam};
use crate::culling::{tapado_por_esfera, Frustum};
use crate::config;
use crate::framebuffer::{rgb, Framebuffer};
use crate::hud::{self, HudData};
use crate::math::*;
use crate::mesh::Mesh;
use crate::obj;
use crate::pipeline::{DrawOpts, FrameQueue, Stats};
use crate::planet::{self, Planet};
use crate::platform::{Input, Key};
use crate::scene;
use crate::shaders::{self, Shader, Uniforms};
use crate::ship::{self, LocalFrame, Ship};
use crate::stars::Stars;
use crate::surface::{self, SurfaceSet};
use crate::texture::Texture;

impl SurfaceSet { fn cfg_planeta_nombre(&self) -> &'static str { config::PLANETAS[self.cfg.planeta].nombre } }

pub struct App {
    /// Niveles de detalle de la esfera lisa (para el Sol).
    sun_lods: Vec<Mesh>,
    planets: Vec<Planet>,
    /// Objetos de superficie (un conjunto por tipo de modelo).
    surface: Vec<SurfaceSet>,
    ship_mesh: Mesh,
    ship_base: Mat4,
    pub ship: Ship,
    pub cam: FollowCam,
    stars: Stars,
    normal_map: Texture,
    /// Cámara fija opcional (--cam) que reemplaza a la de seguimiento.
    pub cam_override: Option<Camera>,
    /// Tiempo de simulación (escalado); gobierna órbitas y rotaciones.
    pub t: f32,
    scale_idx: usize,
    pub paused: bool,
    pub wire: bool,
    pub cull: bool,
    pub hud: bool,
    /// true = todos los planetas con el shader plano (para comparar rendimiento).
    pub flat_shading: bool,
    pub lod: bool,
    pub seed: u64,
    /// Planeta de referencia si estamos en modo local (None = modo espacial).
    pub local: Option<usize>,
}

impl App {
    pub fn new(seed: u64) -> App {
        let ship_mesh = obj::load(config::SHIP_OBJ).expect("no se pudo cargar la nave");
        let ship_base = ship::base_matrix(&ship_mesh);
        let mut sphere = obj::load(config::SPHERE_OBJ).expect("no se pudo cargar la esfera");
        sphere.normalize_to_unit_sphere(); // sphere.obj es un cubo subdividido: se proyecta a radio 1
        let planets: Vec<Planet> = config::PLANETAS.iter().map(|c| Planet::new(c, &sphere)).collect();
        let sun_lods = planet::build_lods(&sphere);
        crate::placeholders::asegurar(config::SURFACE_DIR);
        let surface: Vec<SurfaceSet> = config::SUPERFICIE.iter().map(|c| surface::crear(c, &planets[c.planeta], seed)).collect();
        for s in &surface { println!("superficie: {} -> {} instancias en {} (semilla {seed})", s.cfg.archivo, s.instancias.len(), s.cfg_planeta_nombre()); }
        let p0 = scene::planet_pos(&config::PLANETAS[config::SHIP_START_LOOK_PLANET], 0.0);
        let ship = Ship::new(Vec3::from(config::SHIP_START_POS), p0);
        let cam = FollowCam::new(&ship);
        App { sun_lods, planets, surface, ship_mesh, ship_base, ship, cam, stars: Stars::new(), normal_map: Texture::ripple_normal_map(config::NORMAL_MAP_SIZE, 3.0), cam_override: None, t: 0.0, scale_idx: config::TIME_SCALE_DEFAULT,
              paused: false, wire: false, cull: true, hud: true, flat_shading: false, lod: true, seed, local: None }
    }

    /// Coloca la nave en `pos` mirando al planeta indicado (pruebas y arranque).
    pub fn place_ship(&mut self, pos: Vec3, mirar_planeta: usize) {
        self.ship = Ship::new(pos, self.planets[mirar_planeta].pos(self.t));
        self.local = None;
        self.cam = FollowCam::new(&self.ship);
    }

    /// Cuerpo más cercano (por altitud sobre su superficie) y esa altitud.
    pub fn nearest_altitude(&self) -> (&'static str, f32) {
        let sol_alt = self.ship.pos.length() - config::SUN_RADIUS;
        self.planets.iter().map(|p| (p.cfg.nombre, p.altitude(self.t, self.ship.pos)))
            .fold(("SOL", sol_alt), |a, b| if b.1 < a.1 { b } else { a })
    }

    pub fn time_scale(&self) -> f32 { config::TIME_SCALES[self.scale_idx] }

    /// Avanza la simulación `dt` segundos reales.
    pub fn update(&mut self, inp: &Input, dt: f32) {
        if inp.pressed(Key::P) { self.paused = !self.paused; }
        if inp.pressed(Key::LBracket) && self.scale_idx > 0 { self.scale_idx -= 1; }
        if inp.pressed(Key::RBracket) && self.scale_idx + 1 < config::TIME_SCALES.len() { self.scale_idx += 1; }
        if inp.pressed(Key::F1) { self.hud = !self.hud; }
        if inp.pressed(Key::F2) { self.cull = !self.cull; }
        if inp.pressed(Key::F3) { self.lod = !self.lod; }
        if inp.pressed(Key::F4) { self.wire = !self.wire; }
        if inp.pressed(Key::C) { self.cam.first_person = !self.cam.first_person; }

        // Con pausa todo se congela (dt = 0). La escala de tiempo solo afecta a los cuerpos celestes.
        let dt = if self.paused { 0.0 } else { dt };
        self.t += dt * self.time_scale();
        let cuerpos = scene::bodies(self.t);
        self.actualizar_modo();
        let marco = self.local.map(|i| LocalFrame { vel: self.planets[i].vel(self.t), up: (self.ship.pos - self.planets[i].pos(self.t)).normalize(), dist: (self.ship.pos - self.planets[i].pos(self.t)).length() });
        self.ship.update(inp, dt, &cuerpos, marco.as_ref());
        self.collide_ship();

        // Mirada: flechas izq/der (continuo) o mouse con botón derecho.
        let mut dyaw = (inp.down(Key::Left) as i32 - inp.down(Key::Right) as i32) as f32 * config::LOOK_KEY_RATE * dt;
        let mut dpitch = 0.0;
        if inp.mouse_right {
            dyaw -= inp.mouse_dx * config::LOOK_MOUSE_SENS;
            dpitch -= inp.mouse_dy * config::LOOK_MOUSE_SENS;
        }
        let looking = dyaw != 0.0 || dpitch != 0.0;
        self.cam.update(&self.ship, dt, dyaw, dpitch, looking, self.local.is_some());
        self.keep_camera_outside();
    }

    /// Entra/sale del modo local según la distancia (con histéresis) y cambia el marco de referencia de la velocidad:
    /// al entrar, la velocidad pasa a medirse relativa al planeta (vel -= vel_planeta); al salir, vuelve al mundo.
    /// No hay transición de escena: solo cambia cómo se interpreta la velocidad y los controles.
    fn actualizar_modo(&mut self) {
        let t = self.t;
        let razon = |p: &Planet| (self.ship.pos - p.pos(t)).length() / p.cfg.radio;
        match self.local {
            None => {
                let cercano = self.planets.iter().enumerate().map(|(i, p)| (i, razon(p)))
                    .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                if let Some((i, r)) = cercano {
                    if r < config::LOCAL_ENTER {
                        self.ship.vel -= self.planets[i].vel(t);
                        self.local = Some(i);
                    }
                }
            }
            Some(i) => {
                if razon(&self.planets[i]) > config::LOCAL_EXIT {
                    self.ship.vel += self.ship.frame_vel;
                    self.ship.frame_vel = Vec3::ZERO;
                    self.local = None;
                }
            }
        }
    }

    /// Colisión de la nave con el Sol (esfera lisa) y con cada planeta (superficie deformada).
    fn collide_ship(&mut self) {
        let sol = &scene::bodies(self.t)[0];
        self.ship.collide_sphere(sol.pos, sol.vel, sol.radio);
        for p in &self.planets {
            let r = p.surface_radius_at(self.t, self.ship.pos);
            self.ship.collide_sphere(p.pos(self.t), p.vel(self.t), r);
        }
    }

    /// Evita que la cámara de seguimiento quede dentro de un cuerpo.
    fn keep_camera_outside(&mut self) {
        let m = config::CAM_SURFACE_MARGIN;
        let mut pos = self.cam.cam.pos;
        let d = pos.length();
        if d < config::SUN_RADIUS + m { pos = pos.normalize() * (config::SUN_RADIUS + m); }
        for p in &self.planets {
            let c = p.pos(self.t);
            let r = p.surface_radius_at(self.t, pos) + m;
            if (pos - c).length() < r { pos = c + (pos - c).normalize() * r; }
        }
        self.cam.cam.pos = pos;
    }

    /// Nivel de LOD a usar (F3 apagado = siempre el máximo, para comparar rendimiento).
    fn lod_of(&self, dist_en_radios: f32) -> usize { if self.lod { planet::lod_level(dist_en_radios) } else { config::LOD_LEVELS - 1 } }

    pub fn camera(&self) -> Camera { self.cam_override.unwrap_or(self.cam.cam) }

    /// Dibuja un cuadro completo (escena + estrellas + HUD).
    pub fn render(&self, fb: &mut Framebuffer, fps: &hud::FpsCounter) -> Stats {
        fb.clear(rgb(2, 2, 8));
        let mut stats = Stats::default();
        let cam = self.camera();
        let vp = cam.projection(fb.w as f32 / fb.h as f32) * cam.view();
        let frustum = Frustum::new(&vp);
        let opts = DrawOpts { cull_back: self.cull, wireframe: self.wire };
        let uniforms = |planet: usize, color: [f32; 3]| Uniforms {
            time: self.t, light_pos: Vec3::ZERO, camera_pos: cam.pos, base_color: Vec3::from(color), planet, normal_map: Some(&self.normal_map), color2: Vec3::ZERO, param: 0.0,
        };
        let mut q = FrameQueue::new(fb.w, fb.h);

        // ---- Sol (emisivo, en el origen): frustum + LOD por distancia relativa a su radio.
        stats.objs_total += 1;
        let lod_sol = self.lod_of(cam.pos.length() / config::SUN_RADIUS);
        if !self.cull || frustum.sphere_visible(Vec3::ZERO, config::SUN_RADIUS) {
            let m_sol = q.add_material(shaders::sun::shade, uniforms(0, [1.0, 1.0, 1.0]));
            q.submit_mesh(&self.sun_lods[lod_sol], &scene::sun_model(), &vp, m_sol, &opts, &mut stats);
            stats.objs_drawn += 1;
        } else { stats.objs_culled += 1; }

        // ---- Planetas: la luz es el Sol en el origen; cada uno con su shader.
        let mut lods = format!("SOL {}", lod_sol);
        for (i, p) in self.planets.iter().enumerate() {
            stats.objs_total += 1;
            let centro = p.pos(self.t);
            lods += &format!("  {} {}", &p.cfg.nombre[..3], self.lod_of((cam.pos - centro).length() / p.cfg.radio));
            let r_env = p.cfg.radio * (1.0 + p.cfg.amplitud); // esfera envolvente (incluye el relieve)
            if self.cull {
                // Fuera del frustum, o completamente tapado por el Sol (el Sol, de radio mayor, lo oculta entero).
                let tapado = tapado_por_esfera(cam.pos, centro, Vec3::ZERO, config::SUN_RADIUS - r_env);
                if !frustum.sphere_visible(centro, r_env) || tapado { stats.objs_culled += 1; continue; }
            }
            let sh: Shader = if self.flat_shading { shaders::flat::shade } else { shaders::para_planeta(p.cfg.shader) };
            let lod = self.lod_of((cam.pos - centro).length() / p.cfg.radio);
            let m = q.add_material(sh, uniforms(i, [0.7, 0.7, 0.7]));
            q.submit_mesh(&p.lods[lod], &scene::planet_model(p.cfg, self.t), &vp, m, &opts, &mut stats);
            stats.objs_drawn += 1;
        }

        // ---- Objetos de superficie: hijos de la transformación rígida del planeta (giran y orbitan con él).
        for set in &self.surface {
            let p = &self.planets[set.cfg.planeta];
            stats.objs_total += set.instancias.len() as u32;
            let centro_p = p.pos(self.t);
            if (cam.pos - centro_p).length() > config::SURFACE_DRAW_DIST * p.cfg.radio { continue; } // planeta lejano: ninguna instancia se ve
            let mut u = uniforms(set.cfg.planeta, set.cfg.color_alto);
            u.color2 = Vec3::from(set.cfg.color_bajo);
            u.param = set.cfg.corte;
            let m = q.add_material(shaders::objeto::shade, u);
            let rigido = scene::planet_rigid(p.cfg, self.t);
            for inst in &set.instancias {
                let c = rigido.transform_point(inst.centro);
                // 1) Frustum: esfera envolvente de la instancia vs 6 planos. 2) Horizonte: tapada por el propio planeta.
                if self.cull && (!frustum.sphere_visible(c, inst.radio)
                    || tapado_por_esfera(cam.pos, c, centro_p, (p.cfg.radio - inst.radio).max(0.0))) {
                    stats.objs_culled += 1;
                    continue;
                }
                // 3) LOD por distancia relativa a la altura del objeto: completo / simple / omitido.
                let d = (c - cam.pos).length() / inst.altura;
                let malla = if !self.lod || d < config::LOD_OBJ_DIST[0] { &set.mesh }
                    else if d < config::LOD_OBJ_DIST[1] { &set.mesh_simple }
                    else { stats.objs_lod_omit += 1; continue };
                q.submit_mesh(malla, &(rigido * inst.local), &vp, m, &opts, &mut stats);
                stats.objs_drawn += 1;
            }
        }

        // ---- Nave (en primera persona no se dibuja: la cámara está dentro).
        stats.objs_total += 1;
        if !self.cam.first_person || self.cam_override.is_some() {
            let m = q.add_material(shaders::flat::shade, uniforms(0, [0.75, 0.78, 0.85]));
            q.submit_mesh(&self.ship_mesh, &self.ship.model(&self.ship_base), &vp, m, &opts, &mut stats);
            stats.objs_drawn += 1;
        }
        q.render(fb); // rasterización multihilo por bandas

        // ---- Fondo: estrellas y halo del Sol (solo en píxeles vacíos).
        self.stars.draw(fb, &cam, self.t);
        halo_solar(fb, &cam, &vp);
        for p in &self.planets {
            let a = p.cfg.atmosfera;
            if a == [0.0, 0.0, 0.0] { continue; }
            halo_atmosfera(fb, &cam, &vp, p.pos(self.t), p.cfg.radio * (1.0 + p.cfg.amplitud), Vec3::from(a));
        }

        if self.hud {
            let (nombre, alt) = self.nearest_altitude();
            hud::draw_hud(fb, &HudData {
                fps, stats, speed: self.ship.speed(), altitude: alt, cuerpo: nombre,
                local: self.local.map(|i| self.planets[i].cfg.nombre),
                camara: if self.cam.first_person { "1RA" } else { "3RA" }, time_scale: self.time_scale(), pausa: self.paused, seed: self.seed,
                grav: self.ship.grav.length(), lods, cull: self.cull, lod: self.lod, wire: self.wire,
            });
        }
        stats
    }
}

/// Resplandor del Sol: suma un degradado naranja alrededor del disco, solo sobre píxeles sin geometría.
fn halo_solar(fb: &mut Framebuffer, cam: &Camera, vp: &Mat4) {
    let c = vp.mul_vec4(vec4(0.0, 0.0, 0.0, 1.0));
    if c.w <= config::NEAR || cam.pos.length() < config::SUN_RADIUS * 1.05 { return; }
    let (sx, sy, _) = crate::raster::to_screen(c, fb.w, fb.h);
    // Radio del disco en píxeles: R / distancia * (alto/2) / tan(fov/2).
    let r_px = config::SUN_RADIUS / c.w * (fb.h as f32 * 0.5) / (cam.fov_deg.to_radians() * 0.5).tan();
    let alcance = r_px * 3.5;
    let (x0, x1) = (((sx - alcance).max(0.0)) as usize, ((sx + alcance).min(fb.w as f32 - 1.0)) as usize);
    let (y0, y1) = (((sy - alcance).max(0.0)) as usize, ((sy + alcance).min(fb.h as f32 - 1.0)) as usize);
    if sx + alcance < 0.0 || sy + alcance < 0.0 || sx - alcance > fb.w as f32 || sy - alcance > fb.h as f32 { return; }
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = y * fb.w + x;
            if fb.depth[i] != 0.0 { continue; }
            let d = ((x as f32 - sx).powi(2) + (y as f32 - sy).powi(2)).sqrt() / r_px;
            if d < 1.0 { continue; }
            let g = (-(d - 1.0) * 1.8).exp() * 0.55; // cae exponencialmente con la distancia al borde del disco
            let p = fb.color[i];
            let (r, gg, b) = (((p >> 16) & 255) as f32 / 255.0, ((p >> 8) & 255) as f32 / 255.0, (p & 255) as f32 / 255.0);
            fb.color[i] = crate::shaders::to_u32(vec3(r + g, gg + g * 0.55, b + g * 0.2));
        }
    }
}

/// Halo atmosférico: anillo fino y brillante justo fuera del disco del planeta, más intenso hacia el lado que mira al Sol.
/// Es un efecto de post-proceso en 2D sobre píxeles sin geometría (profundidad = 0).
fn halo_atmosfera(fb: &mut Framebuffer, cam: &Camera, vp: &Mat4, centro: Vec3, radio: f32, color: Vec3) {
    let c = vp.mul_vec4(centro.extend(1.0));
    if c.w <= config::NEAR { return; }
    let (sx, sy, _) = crate::raster::to_screen(c, fb.w, fb.h);
    let r_px = radio / c.w * (fb.h as f32 * 0.5) / (cam.fov_deg.to_radians() * 0.5).tan();
    if r_px < 3.0 || r_px > fb.h as f32 { return; } // muy lejos o demasiado cerca (el borde ya lo da el shader)
    let grosor = (r_px * 0.14).max(2.5);
    // Dirección en pantalla hacia el Sol (para iluminar solo el limbo que lo mira).
    let cs = vp.mul_vec4(vec4(0.0, 0.0, 0.0, 1.0));
    let (ux, uy) = if cs.w > config::NEAR {
        let (a, b, _) = crate::raster::to_screen(cs, fb.w, fb.h);
        (a - sx, b - sy)
    } else { (sx - fb.w as f32 * 0.5, sy - fb.h as f32 * 0.5) };
    let l = (ux * ux + uy * uy).sqrt().max(1e-3);
    let (ux, uy) = (ux / l, uy / l);
    let alcance = r_px + grosor;
    if sx + alcance < 0.0 || sy + alcance < 0.0 || sx - alcance > fb.w as f32 || sy - alcance > fb.h as f32 { return; }
    let (x0, x1) = (((sx - alcance).max(0.0)) as usize, ((sx + alcance).min(fb.w as f32 - 1.0)) as usize);
    let (y0, y1) = (((sy - alcance).max(0.0)) as usize, ((sy + alcance).min(fb.h as f32 - 1.0)) as usize);
    for y in y0..=y1 {
        for x in x0..=x1 {
            let i = y * fb.w + x;
            if fb.depth[i] != 0.0 { continue; }
            let (dx, dy) = (x as f32 - sx, y as f32 - sy);
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < r_px * 0.97 || dist > alcance { continue; }
            let t = ((dist - r_px) / grosor).max(0.0);
            let lado = ((dx * ux + dy * uy) / dist.max(1e-3)) * 0.5 + 0.5; // 1 = mira al Sol
            let g = (1.0 - t).powf(2.2) * lado.powi(2) * 0.9;
            let p = fb.color[i];
            let (r, gg, b) = (((p >> 16) & 255) as f32 / 255.0, ((p >> 8) & 255) as f32 / 255.0, (p & 255) as f32 / 255.0);
            fb.color[i] = crate::shaders::to_u32(vec3(r + color.x * g, gg + color.y * g, b + color.z * g));
        }
    }
}
