//! Nave: estado, física suave (aceleración, amortiguación, giro con velocidad angular suavizada)
//! y su matriz de modelo. Todo escalado por dt, así que se comporta igual en cualquier equipo.

use crate::config;
use crate::math::*;
use crate::mesh::Mesh;
use crate::platform::{Input, Key};
use crate::scene::{self, Body};

/// Matriz que centra la malla, corrige su orientación (SHIP_FORWARD_FIX) y la escala a SHIP_LENGTH.
/// Se calcula desde el bounding box al cargar: sirve para cualquier modelo de nave.
pub fn base_matrix(mesh: &Mesh) -> Mat4 {
    let rot = Mat4::from_euler_deg(config::SHIP_FORWARD_FIX);
    let d = mesh.bmax - mesh.bmin;
    let largo = d.x.max(d.y).max(d.z);
    let s = config::SHIP_LENGTH / largo * config::SHIP_SCALE;
    Mat4::scale_uniform(s) * rot * Mat4::translation(-mesh.center)
}

/// Marco de referencia del modo local: velocidad del planeta y vertical local (radial, unitaria).
pub struct LocalFrame { pub vel: Vec3, pub up: Vec3, /** distancia de la nave al centro del planeta */ pub dist: f32 }

pub struct Ship {
    pub pos: Vec3,
    /// Velocidad de la nave RELATIVA al marco de referencia (el mundo en modo espacial, el planeta en modo local).
    pub vel: Vec3,
    /// Velocidad del marco (0 en modo espacial; la del planeta en modo local). La nave se mueve a vel + frame_vel.
    pub frame_vel: Vec3,
    /// Base ortonormal local: adelante, arriba, derecha (el modelo mira a -Z, con Y arriba).
    pub fwd: Vec3,
    pub up: Vec3,
    pub right: Vec3,
    /// Velocidad angular actual (suavizada): x = cabeceo, y = guiñada, z = alabeo (rad/s).
    ang: Vec3,
    pub turbo: bool,
    /// Modo local: distancia al centro del planeta que se mantiene (retención de altitud) y si el cuadro anterior ya era local.
    local_r: f32,
    was_local: bool,
    /// Aceleración gravitatoria del último cuadro (para el HUD).
    pub grav: Vec3,
}

impl Ship {
    /// Nave en `pos` mirando hacia `target`, con el arriba del mundo como referencia.
    pub fn new(pos: Vec3, target: Vec3) -> Ship {
        let fwd = (target - pos).normalize();
        // Si miramos recto hacia arriba/abajo, "arriba del mundo" es degenerado: usamos -Z como referencia.
        let mut right = fwd.cross(Vec3::UP);
        if right.length() < 1e-3 { right = fwd.cross(vec3(0.0, 0.0, -1.0)); }
        let right = right.normalize();
        Ship { pos, vel: Vec3::ZERO, frame_vel: Vec3::ZERO, fwd, up: right.cross(fwd), right, ang: Vec3::ZERO, turbo: false, local_r: 0.0, was_local: false, grav: Vec3::ZERO }
    }

    pub fn speed(&self) -> f32 { self.vel.length() }

    pub fn update(&mut self, inp: &Input, dt: f32, bodies: &[Body], local: Option<&LocalFrame>) {
        self.frame_vel = local.map(|l| l.vel).unwrap_or(Vec3::ZERO);
        // ---- Giro: velocidad angular deseada según teclas, suavizada hacia ese valor.
        let eje = |pos: Key, neg: Key| inp.down(pos) as i32 as f32 - inp.down(neg) as i32 as f32;
        let signo_cabeceo = if config::INVERT_PITCH { -1.0 } else { 1.0 };
        let objetivo = vec3(
            eje(Key::Up, Key::Down) * signo_cabeceo * config::SHIP_PITCH_RATE,
            eje(Key::A, Key::D) * config::SHIP_YAW_RATE,
            eje(Key::E, Key::Q) * config::SHIP_ROLL_RATE,
        );
        self.ang = self.ang.lerp(objetivo, smooth(config::SHIP_ANG_SMOOTH, dt));
        // Rotaciones alrededor de los ejes locales (regla de la mano derecha), una tras otra:
        // guiñada (eje arriba) -> cabeceo (eje derecha) -> alabeo (eje adelante).
        let (yaw, pitch, roll) = (self.ang.y * dt, self.ang.x * dt, self.ang.z * dt);
        let eje_up = self.up;
        self.right = self.right.rotate_about(eje_up, yaw);
        self.fwd = self.fwd.rotate_about(eje_up, yaw);
        let eje_right = self.right;
        self.fwd = self.fwd.rotate_about(eje_right, pitch);
        self.up = self.up.rotate_about(eje_right, pitch);
        let eje_fwd = self.fwd;
        self.right = self.right.rotate_about(eje_fwd, roll);
        self.up = self.up.rotate_about(eje_fwd, roll);
        // Modo local: el "arriba" de la nave se alinea suavemente con la vertical local (giro de arco corto).
        // Si el jugador cabecea, el equilibrio entre su giro y esta corrección limita la inclinación y vuelve sola.
        if let Some(l) = local {
            let eje = self.up.cross(l.up);
            let ang = eje.length().atan2(self.up.dot(l.up)) * smooth(config::LOCAL_ALIGN_K, dt);
            if eje.length() > 1e-5 {
                let e = eje.normalize();
                self.fwd = self.fwd.rotate_about(e, ang);
                self.right = self.right.rotate_about(e, ang);
                self.up = self.up.rotate_about(e, ang);
            }
        }
        // Reortonormalizar: los errores numéricos se acumulan si no.
        self.fwd = self.fwd.normalize();
        self.right = self.fwd.cross(self.up).normalize();
        self.up = self.right.cross(self.fwd);

        // ---- Empuje y frenado.
        self.turbo = inp.down(Key::Shift);
        let antes = self.speed();
        let empuje = eje(Key::W, Key::S);
        let accel = if self.turbo { config::SHIP_ACCEL_TURBO } else { config::SHIP_ACCEL };
        // En modo local la gravedad se atenúa: la nave puede rodear el planeta sin caer.
        let escala_g = if local.is_some() { config::LOCAL_GRAVITY_SCALE } else { 1.0 };
        self.grav = scene::gravity(self.pos, bodies) * escala_g;
        // Integración semi-implícita: primero se actualiza la velocidad con la aceleración (empuje + gravedad)...
        self.vel += (self.fwd * (empuje * accel) + self.grav) * dt;
        // Amortiguación exponencial: sin empuje frena sola; con S frena fuerte.
        let drag = if inp.down(Key::S) { config::SHIP_DRAG_BRAKE } else if empuje > 0.0 { config::SHIP_DRAG_THRUST } else { config::SHIP_DRAG_COAST };
        self.vel = self.vel * (-drag * dt).exp();
        // Tope de velocidad: nunca se supera con el empuje; si veníamos más rápido (soltaste el turbo)
        // el exceso decae suavemente hacia el tope en vez de cortarse de golpe.
        // Modo local: retención de altitud. Sin teclas, la nave mantiene su distancia al centro del planeta
        // (la velocidad radial converge hacia la que corrige el error) y así rodea el planeta en vez de alejarse en línea recta.
        // Con Espacio/Ctrl sube o baja a velocidad fija y la nueva distancia pasa a ser la que se retiene.
        if let Some(l) = local {
            let sube = eje(Key::Space, Key::Ctrl);
            if !self.was_local || sube != 0.0 { self.local_r = l.dist; }
            let objetivo_vr = if sube != 0.0 { sube * config::LOCAL_CLIMB_SPEED }
                else { clamp((self.local_r - l.dist) * config::LOCAL_ALT_K, -config::LOCAL_CLIMB_SPEED, config::LOCAL_CLIMB_SPEED) };
            let vr = self.vel.dot(l.up);
            self.vel += l.up * ((objetivo_vr - vr) * smooth(config::LOCAL_VERT_K, dt));
        }
        self.was_local = local.is_some();
        let tope = match (local.is_some(), self.turbo) {
            (false, false) => config::SHIP_CRUISE_SPEED,
            (false, true) => config::SHIP_TURBO_SPEED,
            (true, false) => config::LOCAL_SPEED,
            (true, true) => config::LOCAL_TURBO_SPEED,
        };
        let permitido = tope + (antes - tope).max(0.0) * (-3.0 * dt).exp();
        let v = self.speed();
        if v > permitido { self.vel = self.vel * (permitido / v); }

        // ...y luego la posición con la velocidad ya nueva.
        self.pos += (self.vel + self.frame_vel) * dt;
    }

    /// Matriz modelo -> mundo de la nave.
    pub fn model(&self, base: &Mat4) -> Mat4 { Mat4::from_axes(self.right, self.up, -self.fwd, self.pos) * *base }
}

impl Ship {
    /// Colisión contra una superficie: si la nave (esfera de radio SHIP_COLLISION_RADIUS) quedó dentro,
    /// se empuja hacia afuera sobre la normal y se amortigua la velocidad normal (relativa al cuerpo).
    /// `centro`/`vel_cuerpo`: centro y velocidad del cuerpo; `radio_sup`: radio de la superficie bajo la nave.
    pub fn collide_sphere(&mut self, centro: Vec3, vel_cuerpo: Vec3, radio_sup: f32) {
        let d = self.pos - centro;
        let n = d.normalize();
        let limite = radio_sup + config::SHIP_COLLISION_RADIUS;
        if d.length() >= limite { return; }
        self.pos = centro + n * limite;
        let v_rel = self.vel + self.frame_vel - vel_cuerpo; // velocidad respecto al cuerpo (vale en ambos modos)
        let vn = v_rel.dot(n);
        if vn < 0.0 {
            // Quita la componente que entra en el suelo; deja un pequeño rebote.
            self.vel = self.vel - n * (vn * (1.0 + config::COLLISION_BOUNCE));
        }
    }
}
