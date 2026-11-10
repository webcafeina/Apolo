//! Degradados: si los píxeles de una región no son de un color plano sino que cambian suavemente, la
//! región se rellena con un degradado SVG (lineal o radial) en vez de con su color de la paleta.
//!
//! Se ajusta por mínimos cuadrados, canal a canal, a los píxeles del interior de la región (los que son
//! de ese color sin mezcla): lineal, c = a + s·t, con t la proyección sobre la dirección común de los tres
//! canales; radial, c = a + s·r, con el centro del ajuste cuadrático isótropo de la luminancia. Se usa si
//! explica la región bastante mejor que el color plano.

use crate::imagen::Rgba;
use crate::modelo::Relleno;

/// Paradas a lo largo de `u` (de `u0` a `u1`): el color medio de los píxeles en cada tramo (5 tramos), o
/// el del ajuste lineal donde hay pocos.
fn paradas(
    u: &[f64],
    col: &[[f64; 3]],
    par: &[(f64, f64); 3],
    u0: f64,
    u1: f64,
    desde: f64,
) -> Vec<(f64, [u8; 3])> {
    const K: usize = 5;
    let mut suma = [[0.0f64; 3]; K + 1];
    let mut n = [0usize; K + 1];
    for (x, c) in u.iter().zip(col) {
        let k = (((x - u0) / (u1 - u0) * K as f64).round() as usize).min(K);
        for ch in 0..3 {
            suma[k][ch] += c[ch];
        }
        n[k] += 1;
    }
    (0..=K)
        .map(|k| {
            let uk = u0 + (u1 - u0) * k as f64 / K as f64;
            let c = if n[k] >= 20 {
                [0, 1, 2].map(|ch| (suma[k][ch] / n[k] as f64).round().clamp(0.0, 255.0) as u8)
            } else {
                color(par, uk)
            };
            let pos = if u1 > 0.0 && desde >= 0.0 {
                (desde + (1.0 - desde) * k as f64 / K as f64).clamp(0.0, 1.0)
            } else {
                k as f64 / K as f64
            };
            (pos, c)
        })
        .collect()
}

/// Ajuste 1D c = a + s·u por canal; devuelve (a, s) por canal y el error cuadrático total.
fn ajuste_1d(u: &[f64], col: &[[f64; 3]]) -> ([(f64, f64); 3], f64) {
    let n = u.len() as f64;
    let mu = u.iter().sum::<f64>() / n;
    let var = u.iter().map(|x| (x - mu).powi(2)).sum::<f64>().max(1e-9);
    let mut par = [(0.0, 0.0); 3];
    let mut e = 0.0;
    for ch in 0..3 {
        let mc = col.iter().map(|c| c[ch]).sum::<f64>() / n;
        let s = u
            .iter()
            .zip(col)
            .map(|(x, c)| (x - mu) * (c[ch] - mc))
            .sum::<f64>()
            / var;
        let a = mc - s * mu;
        e += u
            .iter()
            .zip(col)
            .map(|(x, c)| (a + s * x - c[ch]).powi(2))
            .sum::<f64>();
        par[ch] = (a, s);
    }
    (par, e)
}

fn color(par: &[(f64, f64); 3], u: f64) -> [u8; 3] {
    [0, 1, 2].map(|ch| (par[ch].0 + par[ch].1 * u).round().clamp(0.0, 255.0) as u8)
}

/// `pix`: los píxeles (x, y) del interior de la región.
pub fn ajustar(im: &Rgba, pix: &[(u32, u32)]) -> Option<Relleno> {
    if pix.len() < 200 {
        return None;
    }
    // Submuestreo para no tardar en regiones grandes.
    let paso = (pix.len() / 4000).max(1);
    let pts: Vec<(f64, f64)> = pix
        .iter()
        .step_by(paso)
        .map(|&(x, y)| (x as f64 + 0.5, y as f64 + 0.5))
        .collect();
    let col: Vec<[f64; 3]> = pix
        .iter()
        .step_by(paso)
        .map(|&(x, y)| {
            let c = im.color(x, y);
            [c[0] as f64, c[1] as f64, c[2] as f64]
        })
        .collect();
    let n = pts.len() as f64;
    let media = [0, 1, 2].map(|ch| col.iter().map(|c| c[ch]).sum::<f64>() / n);
    let e_plano: f64 = col
        .iter()
        .map(|c| (0..3).map(|ch| (c[ch] - media[ch]).powi(2)).sum::<f64>())
        .sum();
    if (e_plano / (3.0 * n)).sqrt() < 4.0 {
        return None;
    } // casi plano: no compensa
    // Lineal: dirección común de los gradientes de los tres canales.
    let (mx, my) = (
        pts.iter().map(|p| p.0).sum::<f64>() / n,
        pts.iter().map(|p| p.1).sum::<f64>() / n,
    );
    let (mut sxx, mut sxy, mut syy) = (0.0, 0.0, 0.0);
    for p in &pts {
        let (dx, dy) = (p.0 - mx, p.1 - my);
        sxx += dx * dx;
        sxy += dx * dy;
        syy += dy * dy;
    }
    let mut g = [(0.0f64, 0.0f64); 3];
    for ch in 0..3 {
        let (mut sxc, mut syc) = (0.0, 0.0);
        for (p, c) in pts.iter().zip(&col) {
            sxc += (p.0 - mx) * (c[ch] - media[ch]);
            syc += (p.1 - my) * (c[ch] - media[ch]);
        }
        let det = sxx * syy - sxy * sxy;
        if det.abs() < 1e-9 {
            return None;
        }
        g[ch] = ((sxc * syy - syc * sxy) / det, (sxx * syc - sxy * sxc) / det);
    }
    let (mut a, mut b, mut c) = (0.0, 0.0, 0.0);
    for v in g {
        a += v.0 * v.0;
        b += v.0 * v.1;
        c += v.1 * v.1;
    }
    let th = 0.5 * (2.0 * b).atan2(a - c);
    let d = (th.cos(), th.sin());
    let u: Vec<f64> = pts
        .iter()
        .map(|p| (p.0 - mx) * d.0 + (p.1 - my) * d.1)
        .collect();
    let (par_l, e_l) = ajuste_1d(&u, &col);
    // Radial: centro del ajuste L = k0 + k1 x + k2 y + k3 (x² + y²) a la luminancia.
    let lum: Vec<f64> = col
        .iter()
        .map(|c| 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2])
        .collect();
    let rad = (|| {
        // Mínimos cuadrados 4×4 resolviendo por eliminación sobre las ecuaciones normales.
        let f = |p: &(f64, f64)| {
            [
                1.0,
                p.0 - mx,
                p.1 - my,
                (p.0 - mx).powi(2) + (p.1 - my).powi(2),
            ]
        };
        let mut m = [[0.0f64; 4]; 4];
        let mut v = [0.0f64; 4];
        for (p, l) in pts.iter().zip(&lum) {
            let q = f(p);
            for i in 0..4 {
                v[i] += q[i] * l;
                for j in 0..4 {
                    m[i][j] += q[i] * q[j];
                }
            }
        }
        // Gauss.
        for i in 0..4 {
            let piv = (i..4).max_by(|&a, &b| m[a][i].abs().total_cmp(&m[b][i].abs()))?;
            m.swap(i, piv);
            v.swap(i, piv);
            if m[i][i].abs() < 1e-12 {
                return None;
            }
            for k in i + 1..4 {
                let t = m[k][i] / m[i][i];
                let fila_i = m[i];
                for (j, x) in m[k].iter_mut().enumerate().skip(i) {
                    *x -= t * fila_i[j];
                }
                v[k] -= t * v[i];
            }
        }
        let mut s = [0.0f64; 4];
        for i in (0..4).rev() {
            s[i] = (v[i] - (i + 1..4).map(|j| m[i][j] * s[j]).sum::<f64>()) / m[i][i];
        }
        if s[3].abs() < 1e-9 {
            return None;
        }
        let centro = (mx - s[1] / (2.0 * s[3]), my - s[2] / (2.0 * s[3]));
        let r: Vec<f64> = pts
            .iter()
            .map(|p| ((p.0 - centro.0).powi(2) + (p.1 - centro.1).powi(2)).sqrt())
            .collect();
        let (par, e) = ajuste_1d(&r, &col);
        Some((centro, r, par, e))
    })();
    let mejor_rad = rad.as_ref().map(|x| x.3).unwrap_or(f64::MAX);
    if e_l.min(mejor_rad) > 0.5 * e_plano {
        return None;
    }
    if e_l <= mejor_rad {
        let (u0, u1) = u
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        if u1 - u0 < 2.0 {
            return None;
        }
        Some(Relleno::Lineal {
            p1: (mx + d.0 * u0, my + d.1 * u0),
            p2: (mx + d.0 * u1, my + d.1 * u1),
            paradas: paradas(&u, &col, &par_l, u0, u1, 0.0),
        })
    } else {
        let (centro, r, par, _) = rad?;
        let (r0, r1) = r
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &x| (a.min(x), b.max(x)));
        if r1 - r0 < 2.0 || r1 > 4.0 * (im.ancho.max(im.alto) as f64) {
            return None;
        }
        Some(Relleno::Radial {
            c: centro,
            r: r1,
            paradas: paradas(&r, &col, &par, r0, r1, r0 / r1),
        })
    }
}

impl Relleno {
    /// El color del degradado en (x, y).
    pub fn color_en(&self, x: f64, y: f64) -> [f64; 3] {
        let (u, p) = match self {
            Relleno::Color(c) => return [c[0] as f64, c[1] as f64, c[2] as f64],
            Relleno::Lineal { p1, p2, paradas } => {
                let d = (p2.0 - p1.0, p2.1 - p1.1);
                let l2 = (d.0 * d.0 + d.1 * d.1).max(1e-9);
                (((x - p1.0) * d.0 + (y - p1.1) * d.1) / l2, paradas)
            }
            Relleno::Radial { c, r, paradas } => (
                (((x - c.0).powi(2) + (y - c.1).powi(2)).sqrt() / r.max(1e-9)),
                paradas,
            ),
        };
        let u = u.clamp(0.0, 1.0);
        let i = p.iter().rposition(|s| s.0 <= u).unwrap_or(0);
        let j = (i + 1).min(p.len() - 1);
        let t = if j == i || p[j].0 <= p[i].0 {
            0.0
        } else {
            (u - p[i].0) / (p[j].0 - p[i].0)
        };
        [0, 1, 2].map(|ch| p[i].1[ch] as f64 * (1.0 - t) + p[j].1[ch] as f64 * t)
    }
}

/// Fusiona en un degradado los colores de la paleta que lo continúan: si los píxeles de otro color
/// encajan en el degradado de una región (error medio de menos de 9 niveles), pasan a esa región. Cambia
/// `mezcla` y devuelve el relleno de cada color (degradado o nada).
pub fn fusionar(
    im: &Rgba,
    mezcla: &mut [crate::subpixel::Mezcla],
    n: usize,
    transparente: Option<usize>,
) -> Vec<Option<Relleno>> {
    let w = im.ancho;
    let interiores = |m: &[crate::subpixel::Mezcla]| -> Vec<Vec<(u32, u32)>> {
        let mut pix = vec![Vec::new(); n];
        for (i, x) in m.iter().enumerate() {
            if x.a == x.b && x.t >= 0.999 && Some(x.a as usize) != transparente {
                pix[x.a as usize].push(((i as u32) % w, (i as u32) / w));
            }
        }
        pix
    };
    let mut rell: Vec<Option<Relleno>> = (0..n).map(|_| None).collect();
    let mut pix = interiores(mezcla);
    let mut orden: Vec<usize> = (0..n).collect();
    orden.sort_by(|a, b| pix[*b].len().cmp(&pix[*a].len()));
    let mut fusionado = vec![false; n];
    for &k in &orden {
        if fusionado[k] || Some(k) == transparente {
            continue;
        }
        let Some(mut r) = ajustar(im, &pix[k]) else {
            continue;
        };
        loop {
            let mut uno = false;
            for j in 0..n {
                if j == k
                    || fusionado[j]
                    || Some(j) == transparente
                    || pix[j].len() < 20
                    || rell[j].is_some()
                {
                    continue;
                }
                let paso = (pix[j].len() / 500).max(1);
                let (mut e, mut m) = (0.0, 0);
                for &(x, y) in pix[j].iter().step_by(paso) {
                    let p = r.color_en(x as f64 + 0.5, y as f64 + 0.5);
                    let c = im.color(x, y);
                    e += (0..3).map(|ch| (p[ch] - c[ch] as f64).powi(2)).sum::<f64>() / 3.0;
                    m += 1;
                }
                if (e / m as f64).sqrt() < 9.0 {
                    for x in mezcla.iter_mut() {
                        if x.a as usize == j {
                            x.a = k as u16;
                        }
                        if x.b as usize == j {
                            x.b = k as u16;
                        }
                    }
                    fusionado[j] = true;
                    uno = true;
                }
            }
            if !uno {
                break;
            }
            pix = interiores(mezcla);
            match ajustar(im, &pix[k]) {
                Some(nr) => r = nr,
                None => break,
            }
        }
        rell[k] = Some(r);
    }
    rell
}
