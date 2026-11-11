//! Coruscant: planeta-ciudad. Cuadrícula de manzanas (proyección tipo cubo sobre la esfera) con hash por celda
//! que decide la altura/tono de cada bloque. De día se ven bloques grises y calles; en el lado nocturno
//! (terminador día/noche según N·L) se encienden ventanas y calles.

use super::*;
use crate::config;
use crate::noise::hash_cell;

pub fn shade(f: &FragmentInput, u: &Uniforms) -> Color {
    let d = f.local_pos.normalize();
    let n = f.normal.normalize();
    let l = (u.light_pos - f.world_pos).normalize();

    // Proyección cúbica: la cara dominante da dos ejes (a,b) para la cuadrícula; `cara` distingue celdas entre caras.
    let (ax, ay, az) = (d.x.abs(), d.y.abs(), d.z.abs());
    let (a, b, cara) = if ax >= ay && ax >= az { (d.y / ax, d.z / ax, if d.x > 0.0 { 0 } else { 1 }) }
        else if ay >= az { (d.x / ay, d.z / ay, if d.y > 0.0 { 2 } else { 3 }) }
        else { (d.x / az, d.y / az, if d.z > 0.0 { 4 } else { 5 }) };
    let (gu, gv) = (a * config::CITY_GRID, b * config::CITY_GRID);
    let (ci, cj) = (gu.floor() as i32, gv.floor() as i32);
    let (fu, fv) = (gu - gu.floor(), gv - gv.floor());
    let h = hash_cell(ci, cj, cara); // "altura" del bloque (0..1)

    // Calles: el borde de cada celda.
    let borde = fu.min(1.0 - fu).min(fv.min(1.0 - fv));
    let calle = 1.0 - smoothstep(0.08, 0.14, borde);

    // Día: bloques grises azulados, más claros los más altos; calles oscuras.
    let tono = 0.28 + 0.4 * h;
    let bloque = vec3(tono * 0.95, tono, tono * 1.1);
    let dia = bloque.lerp(vec3(0.08, 0.08, 0.1), calle) * lambert(n, l, 0.04);

    // Noche: terminador suave; ventanas (sub-celdas con hash, con márgenes) y calles iluminadas.
    // Algunos distritos enteros quedan apagados (hash por celda) para que no sea una lámina uniforme.
    let noche = smoothstep(0.12, -0.12, n.dot(l));
    let (wu, wv) = (fu * 4.0, fv * 4.0);
    let (wi, wj) = (wu.floor() as i32, wv.floor() as i32);
    let dentro = (wu.fract() - 0.5).abs() < 0.28 && (wv.fract() - 0.5).abs() < 0.28; // margen entre ventanas
    let encendida = dentro && hash_cell(ci * 7 + wi, cj * 7 + wj, cara + 10) > 0.72 - 0.2 * h && hash_cell(ci, cj, cara + 20) > 0.3;
    let ventana = if encendida { 1.0 } else { 0.0 };
    let luz_calle = smoothstep(0.03, 0.0, (borde - 0.11).abs()) * 0.45 * (if hash_cell(ci, cj, cara + 30) > 0.4 { 1.0 } else { 0.0 });
    let luz = vec3(1.0, 0.78, 0.45) * (ventana * (0.4 + 0.8 * h) * (1.0 - calle)) + vec3(1.0, 0.65, 0.3) * luz_calle;
    dia * (1.0 - 0.6 * noche) + luz * noche * config::CITY_LIGHT_INTENSITY
}
