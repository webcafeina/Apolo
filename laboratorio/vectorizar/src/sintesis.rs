//! Ajuste por síntesis: se mueven los vértices de un polígono hasta que, pintado con su suavizado, coincida
//! con la cobertura observada en el original.
//!
//! El contorno al 50 % pierde lo que mide menos de un píxel (las puntas de una estrella), pero el original
//! sí lo tiene medio pintado. Pintando el polígono con submuestras (6 × 6 por píxel) y comparando con la
//! cobertura de la capa, el error dice hacia dónde mover cada vértice. Descenso por coordenadas, con
//! pasos de medio píxel a tres centésimas, recalculando solo los píxeles que toca cada vértice.

pub type P = (f64, f64);

const SUB: usize = 6;

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

/// Cobertura del polígono en el píxel (i, j): fracción de submuestras dentro.
fn cubre(poli: &[P], i: i64, j: i64) -> f64 {
    let mut n = 0;
    for sy in 0..SUB {
        for sx in 0..SUB {
            let (x, y) = (i as f64 + (sx as f64 + 0.5) / SUB as f64, j as f64 + (sy as f64 + 0.5) / SUB as f64);
            if dentro(poli, x, y) { n += 1; }
        }
    }
    n as f64 / (SUB * SUB) as f64
}

struct Caja { x0: i64, y0: i64, x1: i64, y1: i64 }

fn caja(pts: &[P], margen: f64) -> Caja {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for q in pts { x0 = x0.min(q.0); y0 = y0.min(q.1); x1 = x1.max(q.0); y1 = y1.max(q.1); }
    Caja { x0: (x0 - margen).floor() as i64, y0: (y0 - margen).floor() as i64, x1: (x1 + margen).ceil() as i64, y1: (y1 + margen).ceil() as i64 }
}

/// Refina los vértices. `obs(i, j)`: cobertura observada de la capa en el píxel. Devuelve si cambió algo.
pub fn refinar(poli: &mut [P], obs: &dyn Fn(i64, i64) -> f64, ancho: u32, alto: u32) -> bool {
    let n = poli.len();
    // Solo figuras pequeñas: es donde el contorno al 50 % pierde detalle (puntas de estrellas) y donde
    // compensa el coste.
    if !(3..=40).contains(&n) { return false; }
    let c = caja(poli, 2.0);
    // Lejos del borde de la imagen (fuera no hay observación) y no demasiado grande.
    if c.x0 < 0 || c.y0 < 0 || c.x1 > ancho as i64 || c.y1 > alto as i64 { return false; }
    if c.x1 - c.x0 > 68 || c.y1 - c.y0 > 68 { return false; }
    // ¿Relleno o hueco? Media observada dentro y fuera del polígono.
    let (mut sd, mut nd, mut sf, mut nf) = (0.0, 0, 0.0, 0);
    for j in c.y0..c.y1 {
        for i in c.x0..c.x1 {
            if dentro(poli, i as f64 + 0.5, j as f64 + 0.5) { sd += obs(i, j); nd += 1; } else { sf += obs(i, j); nf += 1; }
        }
    }
    if nd == 0 || nf == 0 { return false; }
    let (md, mf) = (sd / nd as f64, sf / nf as f64);
    if (md - mf).abs() < 0.5 { return false; }
    let (v_in, v_out) = if md > mf { (1.0, 0.0) } else { (0.0, 1.0) };
    let error = |p: &[P], k: &Caja| -> f64 {
        let mut e = 0.0;
        for j in k.y0..k.y1 {
            for i in k.x0..k.x1 {
                let m = cubre(p, i, j);
                let d = m * v_in + (1.0 - m) * v_out - obs(i, j);
                e += d * d;
            }
        }
        e
    };
    let mut cambio = false;
    for paso in [0.5, 0.25, 0.12, 0.06, 0.03] {
        for _ in 0..12 {
            let mut mejoro = false;
            for v in 0..n {
                let vecinos = [poli[(v + n - 1) % n], poli[v], poli[(v + 1) % n]];
                for (dx, dy) in [(paso, 0.0), (-paso, 0.0), (0.0, paso), (0.0, -paso)] {
                    let movido = (poli[v].0 + dx, poli[v].1 + dy);
                    let k = caja(&[vecinos[0], vecinos[1], vecinos[2], movido], 1.0);
                    let antes = error(poli, &k);
                    let viejo = poli[v];
                    poli[v] = movido;
                    let despues = error(poli, &k);
                    if despues + 1e-9 < antes {
                        mejoro = true;
                        cambio = true;
                    } else {
                        poli[v] = viejo;
                    }
                }
            }
            if !mejoro { break; }
        }
    }
    cambio
}
