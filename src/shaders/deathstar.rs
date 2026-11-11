//! Estrella de la Muerte: esfera metálica gris con paneles dispuestos en filas por latitud (cada fila con su cantidad de
//! paneles, tono por hash y juntas más claras), ventanas diminutas encendidas, zanja ecuatorial oscura con luces y el plato
//! del superláser (anillos concéntricos y centro oscuro) en el hemisferio norte. Todo se calcula con la posición local,
//! así que gira con el cuerpo.

use super::*;
use crate::noise::{hash_cell, value3};

const FILAS: f32 = 15.0;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();
    let v = (u.camera_pos - f.world_pos).normalize();

    let lat = d.y.asin(); // -pi/2..pi/2
    let lon = d.z.atan2(d.x); // -pi..pi
    let tau = std::f32::consts::TAU;

    // Paneles: fila según la latitud; cada fila divide la longitud en un número distinto de columnas.
    let fila_f = (lat / std::f32::consts::PI + 0.5) * FILAS;
    let fila = fila_f.floor() as i32;
    let columnas = 18.0 + (hash_cell(fila, 3, 1) * 14.0).floor(); // más paneles que filas: paneles apaisados
    let col_f = (lon / tau + 0.5) * columnas * (1.0 + (lat.cos() - 1.0) * 0.0); // columnas fijas por fila
    let (fu, fv) = (col_f - col_f.floor(), fila_f - fila_f.floor());
    let junta = 1.0 - smoothstep(0.03, 0.07, fu.min(1.0 - fu).min(fv.min(1.0 - fv) * 1.6));
    let h = hash_cell(col_f.floor() as i32, fila, 5);
    let tono = 0.70 + 0.45 * h + 0.10 * value3(d * 60.0);
    let mut c = vec3(0.50, 0.48, 0.55) * tono; // gris con tinte violáceo
    c = c.lerp(vec3(0.74, 0.72, 0.68), junta * 0.8);

    // Ventanas diminutas encendidas (puntos cálidos) dentro de algunos paneles.
    let (wu, wv) = (fu * 7.0, fv * 4.0);
    let (wi, wj) = (wu.floor() as i32, wv.floor() as i32);
    let punto = (wu.fract() - 0.5).abs() < 0.22 && (wv.fract() - 0.5).abs() < 0.28;
    let luz = if punto && hash_cell(col_f.floor() as i32 * 9 + wi, fila * 5 + wj, 9) > 0.86 { 1.0 } else { 0.0 };

    // Zanja ecuatorial: banda oscura con una hilera de luces.
    let zanja = smoothstep(0.022, 0.012, d.y.abs());
    c = c * (1.0 - 0.8 * zanja);
    let luces_zanja = zanja * if (lon * 90.0).sin() > 0.7 { 1.0 } else { 0.0 };

    // Plato del superláser: círculo en el hemisferio norte con anillos y centro oscuro.
    let eje = vec3(-0.50, 0.42, 0.76).normalize();
    let a = d.dot(eje).clamp(-1.0, 1.0).acos() / 0.34; // 0 en el centro, 1 en el borde del plato
    if a < 1.0 {
        let anillo = 0.5 + 0.5 * (a * 34.0).sin();
        let mut p = vec3(0.60, 0.58, 0.47) * (0.72 + 0.28 * anillo) * (0.6 + 0.4 * a); // más oscuro hacia el centro
        p = p * smoothstep(0.10, 0.18, a) + vec3(0.07, 0.07, 0.06) * (1.0 - smoothstep(0.10, 0.18, a)); // lente central
        let borde = smoothstep(0.90, 0.97, a);
        c = p.lerp(vec3(0.78, 0.76, 0.66), borde);
    }

    let difuso = lambert(n, l, 0.05);
    let brillo = blinn(n, l, v, 25.0) * 0.07;
    c * difuso + Vec3::splat(brillo) + vec3(1.0, 0.92, 0.65) * (luz + luces_zanja) * 0.9 * (1.0 - 0.5 * difuso)
}
