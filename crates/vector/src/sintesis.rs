//! Ajuste por síntesis: se mueven los vértices de un polígono hasta que, pintado con su suavizado, coincida
//! con la cobertura observada en el original.
//!
//! El contorno al 50 % pierde lo que mide menos de un píxel (las puntas de una estrella), pero el original
//! sí lo tiene medio pintado. Pintando el polígono con submuestras (6 × 6 por píxel) y comparando con la
//! cobertura de la capa, el error dice hacia dónde mover cada vértice. Descenso por coordenadas, con
//! pasos de medio píxel a tres centésimas, recalculando solo los píxeles que toca cada vértice.

use crate::geometria::P;

const SUB: usize = 6;

/// Píxeles de margen de la rejilla de error alrededor de la figura: lo más que se aleja un vértice.
const HOLGURA: i64 = 12;

fn dentro(poli: &[P], x: f64, y: f64) -> bool {
    let n = poli.len();
    let mut c = false;
    let mut j = n - 1;
    for i in 0..n {
        let (a, b) = (poli[i], poli[j]);
        if (a.1 > y) != (b.1 > y) && x < (b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0 {
            c = !c;
        }
        j = i;
    }
    c
}

/// Cobertura del polígono en cada píxel de la caja `k` (fracción de submuestras dentro), por filas de
/// submuestras: en cada fila se cortan los lados una vez y se cuentan las submuestras entre cortes. Da lo
/// mismo que preguntar submuestra a submuestra (par o impar de cruces a la derecha) en O(lados) por fila.
fn cubre(poli: &[P], k: &Caja, out: &mut Vec<f64>, cortes: &mut Vec<f64>) {
    let (w, h) = ((k.x1 - k.x0).max(0) as usize, (k.y1 - k.y0).max(0) as usize);
    out.clear();
    out.resize(w * h, 0.0);
    let n = poli.len();
    // Las submuestras, en las mismas posiciones que `dentro` en cada píxel: i + (s + 0,5) / SUB.
    let sub = |p: i64, s: usize| p as f64 + (s as f64 + 0.5) / SUB as f64;
    let mut cuenta = vec![0u32; w * h];
    for fila in 0..h * SUB {
        let y = sub(k.y0 + (fila / SUB) as i64, fila % SUB);
        cortes.clear();
        let mut j = n - 1;
        for i in 0..n {
            let (a, b) = (poli[i], poli[j]);
            if (a.1 > y) != (b.1 > y) {
                cortes.push((b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0);
            }
            j = i;
        }
        if cortes.is_empty() {
            continue;
        }
        cortes.sort_by(f64::total_cmp);
        let base = (fila / SUB) * w;
        // Una submuestra en x está dentro si hay un número impar de cortes estrictamente a su derecha.
        let mut c = 0;
        for sx in 0..w * SUB {
            let x = sub(k.x0 + (sx / SUB) as i64, sx % SUB);
            while c < cortes.len() && cortes[c] <= x {
                c += 1;
            }
            if (cortes.len() - c) % 2 == 1 {
                cuenta[base + sx / SUB] += 1;
            }
        }
    }
    for (o, c) in out.iter_mut().zip(&cuenta) {
        *o = *c as f64 / (SUB * SUB) as f64;
    }
}

struct Caja {
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
}

fn caja(pts: &[P], margen: f64) -> Caja {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for q in pts {
        x0 = x0.min(q.0);
        y0 = y0.min(q.1);
        x1 = x1.max(q.0);
        y1 = y1.max(q.1);
    }
    Caja {
        x0: (x0 - margen).floor() as i64,
        y0: (y0 - margen).floor() as i64,
        x1: (x1 + margen).ceil() as i64,
        y1: (y1 + margen).ceil() as i64,
    }
}

/// Refina los vértices. `obs(i, j)`: cobertura observada de la capa en el píxel. Devuelve si cambió algo.
pub fn refinar(poli: &mut [P], obs: &dyn Fn(i64, i64) -> f64, ancho: u32, alto: u32) -> bool {
    let n = poli.len();
    // Solo figuras pequeñas: es donde el contorno al 50 % pierde detalle (puntas de estrellas) y donde
    // compensa el coste.
    if !(3..=40).contains(&n) {
        return false;
    }
    let c = caja(poli, 2.0);
    // Lejos del borde de la imagen (fuera no hay observación) y no demasiado grande.
    if c.x0 < 0 || c.y0 < 0 || c.x1 > ancho as i64 || c.y1 > alto as i64 {
        return false;
    }
    if c.x1 - c.x0 > 68 || c.y1 - c.y0 > 68 {
        return false;
    }
    // ¿Relleno o hueco? Media observada dentro y fuera del polígono.
    let (mut sd, mut nd, mut sf, mut nf) = (0.0, 0, 0.0, 0);
    for j in c.y0..c.y1 {
        for i in c.x0..c.x1 {
            if dentro(poli, i as f64 + 0.5, j as f64 + 0.5) {
                sd += obs(i, j);
                nd += 1;
            } else {
                sf += obs(i, j);
                nf += 1;
            }
        }
    }
    if nd == 0 || nf == 0 {
        return false;
    }
    let (md, mf) = (sd / nd as f64, sf / nf as f64);
    if (md - mf).abs() < 0.5 {
        return false;
    }
    let (v_in, v_out) = if md > mf { (1.0, 0.0) } else { (0.0, 1.0) };
    // El error de cada píxel con el polígono actual, en una rejilla holgada alrededor de la figura: así el
    // error de antes de mover un vértice se suma de la rejilla, y solo se pinta el polígono movido.
    let g = Caja {
        x0: c.x0 - HOLGURA,
        y0: c.y0 - HOLGURA,
        x1: c.x1 + HOLGURA,
        y1: c.y1 + HOLGURA,
    };
    let gw = (g.x1 - g.x0) as usize;
    let mut ob = Vec::with_capacity(gw * (g.y1 - g.y0) as usize);
    for j in g.y0..g.y1 {
        for i in g.x0..g.x1 {
            ob.push(obs(i, j));
        }
    }
    let err_px = |m: f64, o: f64| {
        let d = m * v_in + (1.0 - m) * v_out - o;
        d * d
    };
    let (mut m, mut cortes) = (Vec::new(), Vec::new());
    cubre(poli, &g, &mut m, &mut cortes);
    let mut rejilla: Vec<f64> = m.iter().zip(&ob).map(|(m, o)| err_px(*m, *o)).collect();
    let dentro_de_g = |k: &Caja| k.x0 >= g.x0 && k.y0 >= g.y0 && k.x1 <= g.x1 && k.y1 <= g.y1;
    let mut cambio = false;
    for paso in [0.5, 0.25, 0.12, 0.06, 0.03] {
        for _ in 0..12 {
            let mut mejoro = false;
            for v in 0..n {
                let vecinos = [poli[(v + n - 1) % n], poli[v], poli[(v + 1) % n]];
                for (dx, dy) in [(paso, 0.0), (-paso, 0.0), (0.0, paso), (0.0, -paso)] {
                    let movido = (poli[v].0 + dx, poli[v].1 + dy);
                    let k = caja(&[vecinos[0], vecinos[1], vecinos[2], movido], 1.0);
                    // Si la figura se ha ido fuera de la rejilla, se deja de refinar.
                    if !dentro_de_g(&k) {
                        continue;
                    }
                    let kw = (k.x1 - k.x0) as usize;
                    let idx = |i: i64, j: i64| (j - g.y0) as usize * gw + (i - g.x0) as usize;
                    let mut antes = 0.0;
                    for j in k.y0..k.y1 {
                        for i in k.x0..k.x1 {
                            antes += rejilla[idx(i, j)];
                        }
                    }
                    let viejo = poli[v];
                    poli[v] = movido;
                    cubre(poli, &k, &mut m, &mut cortes);
                    let mut despues = 0.0;
                    for j in k.y0..k.y1 {
                        for i in k.x0..k.x1 {
                            despues += err_px(
                                m[(j - k.y0) as usize * kw + (i - k.x0) as usize],
                                ob[idx(i, j)],
                            );
                        }
                    }
                    if despues + 1e-9 < antes {
                        mejoro = true;
                        cambio = true;
                        for j in k.y0..k.y1 {
                            for i in k.x0..k.x1 {
                                let q = idx(i, j);
                                rejilla[q] = err_px(
                                    m[(j - k.y0) as usize * kw + (i - k.x0) as usize],
                                    ob[q],
                                );
                            }
                        }
                    } else {
                        poli[v] = viejo;
                    }
                }
            }
            if !mejoro {
                break;
            }
        }
    }
    cambio
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El barrido por filas cuenta las mismas submuestras que `dentro` una a una.
    #[test]
    fn barrido_igual_que_submuestras() {
        // Una estrella de cinco puntas, girada para que no caiga en la cuadrícula.
        let estrella: Vec<P> = (0..10)
            .map(|i| {
                let a = i as f64 * std::f64::consts::PI / 5.0 + 0.3;
                let r = if i % 2 == 0 { 7.3 } else { 2.9 };
                (10.2 + r * a.cos(), 9.7 + r * a.sin())
            })
            .collect();
        let k = caja(&estrella, 1.0);
        let (mut m, mut cortes) = (Vec::new(), Vec::new());
        cubre(&estrella, &k, &mut m, &mut cortes);
        let w = (k.x1 - k.x0) as usize;
        for j in k.y0..k.y1 {
            for i in k.x0..k.x1 {
                let mut n = 0;
                for sy in 0..SUB {
                    for sx in 0..SUB {
                        let x = i as f64 + (sx as f64 + 0.5) / SUB as f64;
                        let y = j as f64 + (sy as f64 + 0.5) / SUB as f64;
                        n += dentro(&estrella, x, y) as usize;
                    }
                }
                let esperado = n as f64 / (SUB * SUB) as f64;
                assert_eq!(
                    m[(j - k.y0) as usize * w + (i - k.x0) as usize],
                    esperado,
                    "píxel {i},{j}"
                );
            }
        }
    }
}
