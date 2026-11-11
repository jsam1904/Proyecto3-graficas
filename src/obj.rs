//! Cargador de OBJ: v, vt, vn, f (v, v/vt, v//vn, v/vt/vn), caras de n vértices
//! (triangula en abanico), índices negativos. Ignora o/g/s/usemtl/mtllib.

use crate::math::*;
use crate::mesh::{Mesh, Vertex};
use std::collections::HashMap;

/// Convierte un índice OBJ (1-based, o negativo = relativo al final) a 0-based.
fn indice(s: &str, len: usize) -> Result<Option<usize>, String> {
    if s.is_empty() { return Ok(None); }
    let i: i64 = s.parse().map_err(|_| format!("índice inválido '{s}'"))?;
    let r = if i > 0 { i - 1 } else { len as i64 + i };
    if r < 0 || r as usize >= len { return Err(format!("índice fuera de rango: {i}")); }
    Ok(Some(r as usize))
}

pub fn load(path: &str) -> Result<Mesh, String> {
    let texto = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    parse(&texto)
}

pub fn parse(texto: &str) -> Result<Mesh, String> {
    let (mut pos, mut uvs, mut nors): (Vec<Vec3>, Vec<Vec2>, Vec<Vec3>) = (vec![], vec![], vec![]);
    let mut vertices: Vec<Vertex> = vec![];
    let mut tris: Vec<[u32; 3]> = vec![];
    // (v, vt, vn) únicos -> índice de vértice. OBJ indexa cada atributo por separado, nosotros unificamos.
    let mut unicos: HashMap<(usize, Option<usize>, Option<usize>), u32> = HashMap::new();
    let mut faltan_normales = false;

    for (n, linea) in texto.lines().enumerate() {
        let mut it = linea.split_whitespace();
        let Some(tag) = it.next() else { continue };
        let nums = |it: std::str::SplitWhitespace| -> Result<Vec<f32>, String> {
            it.map(|s| s.parse::<f32>().map_err(|_| format!("línea {}: número inválido '{s}'", n + 1))).collect()
        };
        match tag {
            "v" => { let v = nums(it)?; pos.push(vec3(v[0], v[1], v[2])); }
            "vt" => { let v = nums(it)?; uvs.push(vec2(v[0], v.get(1).copied().unwrap_or(0.0))); }
            "vn" => { let v = nums(it)?; nors.push(vec3(v[0], v[1], v[2]).normalize()); }
            "f" => {
                let mut cara: Vec<u32> = vec![];
                for tok in it {
                    let p: Vec<&str> = tok.split('/').collect();
                    let vi = indice(p[0], pos.len())?.ok_or("cara sin vértice")?;
                    let ti = if p.len() > 1 { indice(p[1], uvs.len())? } else { None };
                    let ni = if p.len() > 2 { indice(p[2], nors.len())? } else { None };
                    let id = *unicos.entry((vi, ti, ni)).or_insert_with(|| {
                        if ni.is_none() { faltan_normales = true; }
                        vertices.push(Vertex {
                            pos: pos[vi],
                            normal: ni.map(|i| nors[i]).unwrap_or(Vec3::ZERO),
                            uv: ti.map(|i| uvs[i]).unwrap_or_default(),
                            tangent: Vec3::ZERO,
                        });
                        (vertices.len() - 1) as u32
                    });
                    cara.push(id);
                }
                if cara.len() < 3 { return Err(format!("línea {}: cara con menos de 3 vértices", n + 1)); }
                for k in 1..cara.len() - 1 { tris.push([cara[0], cara[k], cara[k + 1]]); } // abanico
            }
            _ => {} // o, g, s, usemtl, mtllib, comentarios...
        }
    }
    let mut mesh = Mesh::new(vertices, tris);
    if faltan_normales { mesh.recompute_normals(); }
    Ok(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quad_abanico_e_indices_negativos() {
        let m = parse("v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\nf -4 -3 -2\n").unwrap();
        assert_eq!(m.tris.len(), 3);
        assert!(m.vertices[0].normal.z.abs() > 0.99); // normal calculada
    }
}
