//! Cámara y cámara de seguimiento (tercera / primera persona).

use crate::config;
use crate::math::*;
use crate::ship::Ship;

#[derive(Clone, Copy)]
pub struct Camera { pub pos: Vec3, pub dir: Vec3, pub up: Vec3, /** campo de visión vertical en grados */ pub fov_deg: f32 }

impl Camera {
    /// yaw=0 mira hacia -Z; yaw positivo gira a la izquierda; pitch positivo mira arriba. (Radianes.)
    pub fn from_yaw_pitch(pos: Vec3, yaw: f32, pitch: f32) -> Camera {
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        Camera { pos, dir: vec3(-sy * cp, sp, -cy * cp), up: Vec3::UP, fov_deg: config::FOV_Y_DEG }
    }
    pub fn view(&self) -> Mat4 { Mat4::look_at(self.pos, self.pos + self.dir, self.up) }
    pub fn projection(&self, aspect: f32) -> Mat4 { Mat4::perspective(self.fov_deg.to_radians(), aspect, config::NEAR, config::FAR) }
}

pub struct FollowCam {
    pub cam: Camera,
    pub first_person: bool,
    /// Offset suavizado respecto a la nave (en mundo). Suavizar el offset (y no la posición)
    /// evita que la cámara se rezague a alta velocidad: solo "arrastra" al girar.
    offset: Vec3,
    look_yaw: f32,
    look_pitch: f32,
    /// Mezcla suave 0..1 del sesgo de cámara del modo local (evita saltos al entrar/salir).
    local_k: f32,
}

impl FollowCam {
    pub fn new(ship: &Ship) -> FollowCam {
        let mut c = FollowCam { cam: Camera { pos: Vec3::ZERO, dir: ship.fwd, up: ship.up, fov_deg: config::FOV_Y_DEG }, first_person: false, offset: Vec3::ZERO, look_yaw: 0.0, look_pitch: 0.0, local_k: 0.0 };
        c.offset = -ship.fwd * config::CAM_DIST + ship.up * config::CAM_HEIGHT;
        c.update(ship, 0.0, 0.0, 0.0, false, false);
        c
    }

    /// `look_dyaw/look_dpitch`: giro de mirada en radianes de este cuadro; `looking`: si el jugador está mirando (no recentrar).
    pub fn update(&mut self, ship: &Ship, dt: f32, look_dyaw: f32, look_dpitch: f32, looking: bool, local: bool) {
        self.look_yaw += look_dyaw;
        self.look_pitch = clamp(self.look_pitch + look_dpitch, -1.4, 1.4);
        if !looking {
            let k = (-config::LOOK_RECENTER * dt).exp();
            self.look_yaw *= k;
            self.look_pitch *= k;
        }
        // Dirección de mirada = dirección de la nave girada por el offset de mirada.
        let right_l = ship.right.rotate_about(ship.up, self.look_yaw);
        let dir_l = ship.fwd.rotate_about(ship.up, self.look_yaw).rotate_about(right_l, self.look_pitch);

        // En modo local el campo de visión se amplía suavemente (más planeta en cuadro, sensación de cercanía).
        self.local_k += (if local { 1.0 } else { 0.0 } - self.local_k) * smooth(config::FOV_SMOOTH, dt);
        let fov_obj = if local { config::FOV_LOCAL_DEG } else { config::FOV_Y_DEG };
        self.cam.fov_deg += (fov_obj - self.cam.fov_deg) * smooth(config::FOV_SMOOTH, dt);

        if self.first_person {
            self.cam.pos = ship.pos + ship.fwd * 0.4 + ship.up * 0.25;
            self.cam.dir = dir_l;
            self.cam.up = ship.up;
            self.offset = self.cam.pos - ship.pos; // para que al volver a 3ª persona no salte
            return;
        }
        let frac = clamp(ship.vel.length() / config::SHIP_TURBO_SPEED, 0.0, 1.0);
        let dist = config::CAM_DIST * (1.0 + config::CAM_SPEED_PULLBACK * frac);
        let deseado = -dir_l * dist + ship.up * config::CAM_HEIGHT;
        self.offset = self.offset.lerp(deseado, smooth(config::CAM_FOLLOW_K, dt));
        self.cam.pos = ship.pos + self.offset;
        // En modo local la nave va alineada con la vertical local: se baja el punto de mira para que el planeta quede en cuadro.
        let abajo = -ship.up * (config::CAM_LOCAL_DOWN * self.local_k);
        let objetivo = ship.pos + dir_l * config::CAM_LOOK_AHEAD + abajo;
        self.cam.dir = (objetivo - self.cam.pos).normalize();
        self.cam.up = self.cam.up.lerp(ship.up, smooth(config::CAM_UP_K, dt)).normalize();
    }
}
