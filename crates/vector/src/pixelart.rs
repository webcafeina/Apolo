//! Pixel art: el contorno exacto de los píxeles, sin suavizar.
//!
//! Las capas se apilan como en el resto del motor: la capa de rango r cubre los colores de rango r o más,
//! así que entre dos colores nunca queda una rendija. Cada contorno sigue las aristas de la cuadrícula,
//! con los tramos rectos unidos en uno solo.

use crate::geometria::P;
use crate::imagen::Rgba;
use crate::modelo::{Capa, Documento, Forma, Relleno, Tramo};
use std::collections::HashMap;

/// `im` ya lleva los colores de `pal` exactos.
pub fn trazar(im: &Rgba, pal: &[[u8; 3]]) -> Documento {
    let (w, h) = (im.ancho as i64, im.alto as i64);
    let indice: HashMap<[u8; 3], usize> = pal.iter().enumerate().map(|(i, c)| (*c, i)).collect();
    let etiqueta: Vec<usize> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| *indice.get(&im.color(x as u32, y as u32)).unwrap_or(&0))
        .collect();
    let mut area = vec![0usize; pal.len()];
    for &k in &etiqueta {
        area[k] += 1;
    }
    let mut orden: Vec<usize> = (0..pal.len()).filter(|&k| area[k] > 0).collect();
    orden.sort_by(|a, b| area[*b].cmp(&area[*a]).then(a.cmp(b)));
    let mut rango = vec![0usize; pal.len()];
    for (r, &k) in orden.iter().enumerate() {
        rango[k] = r;
    }
    let mut doc = Documento {
        ancho: im.ancho,
        alto: im.alto,
        fondo: orden.first().map(|&k| Relleno::Color(pal[k])),
        capas: Vec::new(),
    };
    for (r, &k) in orden.iter().enumerate().skip(1) {
        let dentro = |x: i64, y: i64| {
            x >= 0 && y >= 0 && x < w && y < h && rango[etiqueta[(y * w + x) as usize]] >= r
        };
        let formas: Vec<Forma> = lazos(w, h, &dentro)
            .into_iter()
            .map(|l| {
                let n = l.len();
                Forma::Tramos((0..n).map(|i| Tramo::Linea(l[i], l[(i + 1) % n])).collect())
            })
            .collect();
        if !formas.is_empty() {
            doc.capas.push(Capa {
                relleno: Relleno::Color(pal[k]),
                formas,
            });
        }
    }
    doc
}

/// Los contornos de la máscara sobre la cuadrícula, con el interior a la izquierda y sin vértices en
/// medio de un tramo recto.
fn lazos(w: i64, h: i64, dentro: &dyn Fn(i64, i64) -> bool) -> Vec<Vec<P>> {
    // Aristas dirigidas (de vértice a vértice de la cuadrícula) entre un píxel de dentro y uno de fuera.
    let mut sale: HashMap<(i64, i64), Vec<(i64, i64)>> = HashMap::new();
    for y in 0..h {
        for x in 0..w {
            if !dentro(x, y) {
                continue;
            }
            if !dentro(x, y - 1) {
                sale.entry((x + 1, y)).or_default().push((x, y));
            }
            if !dentro(x - 1, y) {
                sale.entry((x, y)).or_default().push((x, y + 1));
            }
            if !dentro(x, y + 1) {
                sale.entry((x, y + 1)).or_default().push((x + 1, y + 1));
            }
            if !dentro(x + 1, y) {
                sale.entry((x + 1, y + 1)).or_default().push((x + 1, y));
            }
        }
    }
    let mut claves: Vec<(i64, i64)> = sale.keys().copied().collect();
    claves.sort();
    let mut lazos = Vec::new();
    for ini in claves {
        while let Some(siguientes) = sale.get_mut(&ini) {
            let Some(primero) = siguientes.pop() else {
                break;
            };
            let mut camino = vec![ini];
            let (mut prev, mut cur) = (ini, primero);
            while cur != ini {
                camino.push(cur);
                let dir = (cur.0 - prev.0, cur.1 - prev.1);
                let Some(op) = sale.get_mut(&cur) else { break };
                if op.is_empty() {
                    break;
                }
                // En un cruce en damero, se gira a la izquierda para no mezclar dos regiones.
                let izquierda = (cur.0 + dir.1, cur.1 - dir.0);
                let pos = op.iter().position(|&q| q == izquierda).unwrap_or(0);
                let sig = op.swap_remove(pos);
                prev = cur;
                cur = sig;
            }
            // Sin vértices en medio de un tramo recto.
            let n = camino.len();
            let limpio: Vec<P> = (0..n)
                .filter(|&i| {
                    let (a, b, c) = (camino[(i + n - 1) % n], camino[i], camino[(i + 1) % n]);
                    (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0) != 0
                })
                .map(|i| (camino[i].0 as f64, camino[i].1 as f64))
                .collect();
            if limpio.len() >= 4 {
                lazos.push(limpio);
            }
        }
    }
    lazos
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn cuadrado_con_hueco() {
        // 4×4 blanco con un marco negro de 1 px: la capa negra, un cuadrado exterior y uno interior.
        let mut px = vec![255u8; 4 * 4 * 4];
        for y in 0..4 {
            for x in 0..4 {
                if x == 0 || y == 0 || x == 3 || y == 3 {
                    let i = (y * 4 + x) * 4;
                    px[i..i + 3].copy_from_slice(&[0, 0, 0]);
                }
            }
        }
        let im = Rgba::nueva(4, 4, px);
        let doc = trazar(&im, &[[0, 0, 0], [255, 255, 255]]);
        assert_eq!(doc.fondo, Some(Relleno::Color([0, 0, 0])));
        assert_eq!(doc.capas.len(), 1);
        assert_eq!(
            doc.capas[0].formas.len(),
            1,
            "el blanco: un cuadrado de 2×2"
        );
        if let Forma::Tramos(t) = &doc.capas[0].formas[0] {
            assert_eq!(t.len(), 4);
        } else {
            panic!()
        }
    }
}
