//! Matemáticas propias: Vec2/3/4 y Mat4 (filas, se multiplica M * v con v columna).
//! Sistema de coordenadas: mano derecha, Y arriba, la cámara mira hacia -Z.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec2 { pub x: f32, pub y: f32 }

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec3 { pub x: f32, pub y: f32, pub z: f32 }

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Vec4 { pub x: f32, pub y: f32, pub z: f32, pub w: f32 }

pub const fn vec2(x: f32, y: f32) -> Vec2 { Vec2 { x, y } }
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 { Vec3 { x, y, z } }
pub const fn vec4(x: f32, y: f32, z: f32, w: f32) -> Vec4 { Vec4 { x, y, z, w } }

impl Vec2 {
    pub fn lerp(self, o: Vec2, t: f32) -> Vec2 { vec2(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t) }
}
impl Add for Vec2 { type Output = Vec2; fn add(self, o: Vec2) -> Vec2 { vec2(self.x + o.x, self.y + o.y) } }
impl Mul<f32> for Vec2 { type Output = Vec2; fn mul(self, s: f32) -> Vec2 { vec2(self.x * s, self.y * s) } }

impl Vec3 {
    pub const ZERO: Vec3 = vec3(0.0, 0.0, 0.0);
    pub const UP: Vec3 = vec3(0.0, 1.0, 0.0);
    pub fn splat(v: f32) -> Vec3 { vec3(v, v, v) }
    pub fn dot(self, o: Vec3) -> f32 { self.x * o.x + self.y * o.y + self.z * o.z }
    pub fn cross(self, o: Vec3) -> Vec3 {
        vec3(self.y * o.z - self.z * o.y, self.z * o.x - self.x * o.z, self.x * o.y - self.y * o.x)
    }
    pub fn length(self) -> f32 { self.dot(self).sqrt() }
    /// Normaliza; si el vector es casi nulo devuelve cero (evita NaN).
    pub fn normalize(self) -> Vec3 {
        let l = self.length();
        if l > 1e-12 { self * (1.0 / l) } else { Vec3::ZERO }
    }
    pub fn lerp(self, o: Vec3, t: f32) -> Vec3 { self + (o - self) * t }
    pub fn min(self, o: Vec3) -> Vec3 { vec3(self.x.min(o.x), self.y.min(o.y), self.z.min(o.z)) }
    pub fn max(self, o: Vec3) -> Vec3 { vec3(self.x.max(o.x), self.y.max(o.y), self.z.max(o.z)) }
    /// Rotación de Rodrigues: gira `self` un ángulo (rad) alrededor de `axis` (unitario), regla de la mano derecha.
    pub fn rotate_about(self, axis: Vec3, ang: f32) -> Vec3 {
        let (s, c) = ang.sin_cos();
        self * c + axis.cross(self) * s + axis * (axis.dot(self) * (1.0 - c))
    }
    pub fn extend(self, w: f32) -> Vec4 { vec4(self.x, self.y, self.z, w) }
}
impl Add for Vec3 { type Output = Vec3; fn add(self, o: Vec3) -> Vec3 { vec3(self.x + o.x, self.y + o.y, self.z + o.z) } }
impl Sub for Vec3 { type Output = Vec3; fn sub(self, o: Vec3) -> Vec3 { vec3(self.x - o.x, self.y - o.y, self.z - o.z) } }
impl Mul<f32> for Vec3 { type Output = Vec3; fn mul(self, s: f32) -> Vec3 { vec3(self.x * s, self.y * s, self.z * s) } }
impl Mul<Vec3> for Vec3 { type Output = Vec3; fn mul(self, o: Vec3) -> Vec3 { vec3(self.x * o.x, self.y * o.y, self.z * o.z) } }
impl Div<f32> for Vec3 { type Output = Vec3; fn div(self, s: f32) -> Vec3 { self * (1.0 / s) } }
impl Neg for Vec3 { type Output = Vec3; fn neg(self) -> Vec3 { vec3(-self.x, -self.y, -self.z) } }
impl AddAssign for Vec3 { fn add_assign(&mut self, o: Vec3) { *self = *self + o; } }
impl SubAssign for Vec3 { fn sub_assign(&mut self, o: Vec3) { *self = *self - o; } }

impl From<[f32; 3]> for Vec3 { fn from(a: [f32; 3]) -> Vec3 { vec3(a[0], a[1], a[2]) } }

impl Vec4 {
    pub fn lerp(self, o: Vec4, t: f32) -> Vec4 {
        vec4(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t, self.z + (o.z - self.z) * t, self.w + (o.w - self.w) * t)
    }
    pub fn xyz(self) -> Vec3 { vec3(self.x, self.y, self.z) }
}

/// Matriz 4x4: `m[fila][columna]`.
#[derive(Clone, Copy, Debug)]
pub struct Mat4 { pub m: [[f32; 4]; 4] }

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 { m: [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]] };

    pub fn translation(t: Vec3) -> Mat4 {
        let mut r = Mat4::IDENTITY;
        r.m[0][3] = t.x; r.m[1][3] = t.y; r.m[2][3] = t.z;
        r
    }
    pub fn scale(s: Vec3) -> Mat4 {
        let mut r = Mat4::IDENTITY;
        r.m[0][0] = s.x; r.m[1][1] = s.y; r.m[2][2] = s.z;
        r
    }
    pub fn scale_uniform(s: f32) -> Mat4 { Mat4::scale(Vec3::splat(s)) }
    pub fn rotation_x(a: f32) -> Mat4 {
        let (s, c) = a.sin_cos();
        let mut r = Mat4::IDENTITY;
        r.m[1][1] = c; r.m[1][2] = -s; r.m[2][1] = s; r.m[2][2] = c;
        r
    }
    pub fn rotation_y(a: f32) -> Mat4 {
        let (s, c) = a.sin_cos();
        let mut r = Mat4::IDENTITY;
        r.m[0][0] = c; r.m[0][2] = s; r.m[2][0] = -s; r.m[2][2] = c;
        r
    }
    pub fn rotation_z(a: f32) -> Mat4 {
        let (s, c) = a.sin_cos();
        let mut r = Mat4::IDENTITY;
        r.m[0][0] = c; r.m[0][1] = -s; r.m[1][0] = s; r.m[1][1] = c;
        r
    }
    /// Matriz cuyas columnas son los ejes X,Y,Z de un sistema local y su origen (modelo -> mundo).
    pub fn from_axes(x: Vec3, y: Vec3, z: Vec3, origin: Vec3) -> Mat4 {
        Mat4 { m: [
            [x.x, y.x, z.x, origin.x],
            [x.y, y.y, z.y, origin.y],
            [x.z, y.z, z.z, origin.z],
            [0.0, 0.0, 0.0, 1.0],
        ] }
    }
    /// Rotación con ángulos en grados: primero X, luego Y, luego Z.
    pub fn from_euler_deg(d: [f32; 3]) -> Mat4 {
        let r = std::f32::consts::PI / 180.0;
        Mat4::rotation_z(d[2] * r) * Mat4::rotation_y(d[1] * r) * Mat4::rotation_x(d[0] * r)
    }

    /// Matriz de vista: lleva el mundo al espacio de la cámara (mira hacia -Z).
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
        let f = (target - eye).normalize();
        let r = f.cross(up).normalize();
        let u = r.cross(f);
        Mat4 { m: [
            [r.x, r.y, r.z, -r.dot(eye)],
            [u.x, u.y, u.z, -u.dot(eye)],
            [-f.x, -f.y, -f.z, f.dot(eye)],
            [0.0, 0.0, 0.0, 1.0],
        ] }
    }

    /// Proyección en perspectiva. Deja w_clip = -z_vista (la distancia a la cámara).
    pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
        let f = 1.0 / (fov_y * 0.5).tan();
        Mat4 { m: [
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) / (near - far), 2.0 * far * near / (near - far)],
            [0.0, 0.0, -1.0, 0.0],
        ] }
    }

    pub fn mul_vec4(&self, v: Vec4) -> Vec4 {
        let m = &self.m;
        vec4(
            m[0][0] * v.x + m[0][1] * v.y + m[0][2] * v.z + m[0][3] * v.w,
            m[1][0] * v.x + m[1][1] * v.y + m[1][2] * v.z + m[1][3] * v.w,
            m[2][0] * v.x + m[2][1] * v.y + m[2][2] * v.z + m[2][3] * v.w,
            m[3][0] * v.x + m[3][1] * v.y + m[3][2] * v.z + m[3][3] * v.w,
        )
    }
    /// Transforma un punto (w=1).
    pub fn transform_point(&self, p: Vec3) -> Vec3 { self.mul_vec4(p.extend(1.0)).xyz() }
    /// Transforma una dirección (w=0, ignora la traslación).
    pub fn transform_dir(&self, d: Vec3) -> Vec3 { self.mul_vec4(d.extend(0.0)).xyz() }
}

impl Mul for Mat4 {
    type Output = Mat4;
    fn mul(self, o: Mat4) -> Mat4 {
        let mut r = [[0.0f32; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                r[i][j] = (0..4).map(|k| self.m[i][k] * o.m[k][j]).sum();
            }
        }
        Mat4 { m: r }
    }
}

pub fn clamp(x: f32, a: f32, b: f32) -> f32 { x.max(a).min(b) }

/// Factor de suavizado independiente de los FPS: 1 - e^(-k·dt). Se usa como t en lerp.
pub fn smooth(k: f32, dt: f32) -> f32 { 1.0 - (-k * dt).exp() }

/// Interpolación suave 0..1 entre e0 y e1 (3t²-2t³). Funciona también con e0 > e1 (curva invertida).
pub fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    let t = clamp((x - e0) / (e1 - e0), 0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
