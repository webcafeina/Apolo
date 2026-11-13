//! Contornos por debajo del píxel (hipótesis central del banco).
//!
//! Cada píxel de borde se explica como mezcla de dos colores planos de su alrededor: c ≈ t·A + (1−t)·B.
//! Para cada capa (un conjunto de colores), la cobertura de un píxel es la parte de su mezcla que cae en
//! la capa: 1 dentro, 0 fuera y t en el borde. El contorno de la capa es la línea de nivel 0,5 de esa
//! cobertura, con «marching squares» sobre los centros de los píxeles: sus vértices caen entre píxeles,
//! donde el suavizado dice que estaba el borde de verdad. Es lo que Vectorizer.ai describe como colocar
//! los bordes «según los valores del suavizado».

use crate::imagen::Rgba;
use std::collections::HashMap;

fn dist2(a: [f32; 3], b: [f32; 3]) -> f32 {
    (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)
}

fn f3(c: [u8; 3]) -> [f32; 3] {
    [c[0] as f32, c[1] as f32, c[2] as f32]
}

/// Por píxel: los dos colores de la paleta (índices) y la parte del primero.
#[derive(Clone, Copy)]
pub struct Mezcla {
    pub a: u16,
    pub b: u16,
    pub t: f32,
}

pub fn mezclas(im: &Rgba, paleta: &[[u8; 3]], radio: i64) -> Vec<Mezcla> {
    let (w, h) = (im.ancho as i64, im.alto as i64);
    let pal: Vec<[f32; 3]> = paleta.iter().map(|c| f3(*c)).collect();
    let cercano = |c: [f32; 3]| {
        (0..pal.len())
            .min_by(|&i, &j| dist2(pal[i], c).total_cmp(&dist2(pal[j], c)))
            .unwrap_or(0) as u16
    };
    let mut plano: Vec<Option<u16>> = vec![None; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let c = im.color(x as u32, y as u32);
            let ok = im.alfa((y * w + x) as usize) == 255
                && (-1..=1).all(|dy| {
                    (-1..=1).all(|dx| {
                        let (nx, ny) = (x + dx, y + dy);
                        if nx < 0 || ny < 0 || nx >= w || ny >= h {
                            return true;
                        }
                        let n = im.color(nx as u32, ny as u32);
                        im.alfa((ny * w + nx) as usize) == 255
                            && (0..3).all(|i| (n[i] as i32 - c[i] as i32).abs() <= 10)
                    })
                });
            if ok {
                plano[(y * w + x) as usize] = Some(cercano(f3(c)));
            }
        }
    }
    // El color de la paleta más parecido a cada píxel.
    let mut out = Vec::with_capacity((w * h) as usize);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if let Some(k) = plano[i] {
                out.push(Mezcla { a: k, b: k, t: 1.0 });
                continue;
            }
            let c = f3(im.color(x as u32, y as u32));
            let mut cand: Vec<u16> = Vec::new();
            for dy in -radio..=radio {
                for dx in -radio..=radio {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx >= 0
                        && ny >= 0
                        && nx < w
                        && ny < h
                        && let Some(k) = plano[(ny * w + nx) as usize]
                        && !cand.contains(&k)
                    {
                        cand.push(k);
                    }
                }
            }
            // El color global más parecido siempre es candidato: en un trazo de 1 o 2 px de ancho no hay
            // ningún píxel plano de su color cerca, y sin esto el trazo desaparece.
            let propio = cercano(c);
            if !cand.contains(&propio) {
                cand.push(propio);
            }
            // El par (o el color solo) que mejor explica el píxel como mezcla.
            let (mejor, _) = elegir(&pal, &cand, c);
            out.push(mejor);
        }
    }
    out
}

/// El par de colores de `cand` (o el color solo) que mejor explica `c` como mezcla, y su error cuadrático.
fn elegir(pal: &[[f32; 3]], cand: &[u16], c: [f32; 3]) -> (Mezcla, f32) {
    let mut mejor = Mezcla {
        a: cand[0],
        b: cand[0],
        t: 1.0,
    };
    let mut err = dist2(pal[cand[0] as usize], c);
    for &k in cand {
        let e = dist2(pal[k as usize], c);
        if e < err {
            err = e;
            mejor = Mezcla { a: k, b: k, t: 1.0 };
        }
    }
    for (ia, &ka) in cand.iter().enumerate() {
        for &kb in &cand[ia + 1..] {
            let (pa, pb) = (pal[ka as usize], pal[kb as usize]);
            let d = [pa[0] - pb[0], pa[1] - pb[1], pa[2] - pb[2]];
            let n = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
            if n < 1.0 {
                continue;
            }
            let t = (((c[0] - pb[0]) * d[0] + (c[1] - pb[1]) * d[1] + (c[2] - pb[2]) * d[2]) / n)
                .clamp(0.0, 1.0);
            let m = [pb[0] + t * d[0], pb[1] + t * d[1], pb[2] + t * d[2]];
            let e = dist2(m, c);
            if e < err {
                err = e;
                mejor = Mezcla { a: ka, b: kb, t };
            }
        }
    }
    (mejor, err)
}

/// Lo mal que puede quedar explicado un píxel (distancia RGB) antes de buscarle otra mezcla en `afinar`.
const MAL_EXPLICADO: f32 = 25.0;

/// Lo cerca (distancia RGB) que tiene que estar un píxel de un color de la paleta para contar como muestra pura.
const PURO: f32 = 12.0;

/// Una segunda pasada sobre los píxeles mal explicados, ya decididos los degradados: se prueba también con el
/// color de cada vecino que es muestra pura de un color plano de la paleta (a menos de 12), aunque no sea plano
/// él mismo. El centro de un trazo fino (la punta de un rayo, un trazo de una silueta, una estrella pequeña) no
/// tiene píxeles planos, y el gris de su borde es mezcla de su color y el fondo; antes se explicaba con el color
/// plano más cercano, con un error de 40 o más. Solo se cambia si la mezcla nueva lo explica al menos el doble de
/// bien, y no se tocan los píxeles ni los colores de un degradado (`rellenos`): cambiar los bordes antes de
/// decidir los degradados alteraba qué colores se unían y deshacía el brillo del sol, y medir un píxel contra el
/// color de su degradado en ese punto reasignaba mal los bordes del brillo.
pub fn afinar(
    im: &Rgba,
    paleta: &[[u8; 3]],
    m: &mut [Mezcla],
    radio: i64,
    rellenos: &[Option<crate::modelo::Relleno>],
) -> usize {
    let (w, h) = (im.ancho as i64, im.alto as i64);
    let pal: Vec<[f32; 3]> = paleta.iter().map(|c| f3(*c)).collect();
    let libre =
        |k: u16| (k as usize) < pal.len() && rellenos.get(k as usize).is_none_or(|r| r.is_none());
    let puros: Vec<Option<u16>> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| {
            if im.alfa((y * w + x) as usize) < 255 {
                return None;
            }
            let c = f3(im.color(x as u32, y as u32));
            let k = (0..pal.len())
                .filter(|&k| libre(k as u16))
                .min_by(|&i, &j| dist2(pal[i], c).total_cmp(&dist2(pal[j], c)))?;
            (dist2(pal[k], c) <= PURO * PURO).then_some(k as u16)
        })
        .collect();
    let mut cambiados = 0;
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let actual = m[i];
            if im.alfa(i) < 255 || !libre(actual.a) || !libre(actual.b) {
                continue;
            }
            let c = f3(im.color(x as u32, y as u32));
            let mut cand = vec![actual.a];
            if actual.b != actual.a {
                cand.push(actual.b);
            }
            let (_, err) = elegir(&pal, &cand, c);
            if err <= MAL_EXPLICADO * MAL_EXPLICADO {
                continue;
            }
            let antes = cand.len();
            for dy in -radio..=radio {
                for dx in -radio..=radio {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx >= 0
                        && ny >= 0
                        && nx < w
                        && ny < h
                        && let Some(k) = puros[(ny * w + nx) as usize]
                        && !cand.contains(&k)
                    {
                        cand.push(k);
                    }
                }
            }
            if cand.len() == antes {
                continue;
            }
            let (nueva, e2) = elegir(&pal, &cand, c);
            if e2 < 0.5 * err {
                m[i] = nueva;
                cambiados += 1;
            }
        }
    }
    cambiados
}

/// Lo transparente como un color más, el `transparente` (que no se dibuja): un píxel transparente del todo
/// es solo eso, y uno a medias, mezcla de su color y el transparente con t = alfa, así que el contorno de una
/// figura sobre transparente sale por debajo del píxel como entre dos colores. Su color es el que tendría
/// opaco (se quita el blanco del aplanado).
pub fn transparencia(im: &Rgba, paleta: &[[u8; 3]], m: &mut [Mezcla], transparente: u16) {
    let pal: Vec<[f32; 3]> = paleta.iter().map(|c| f3(*c)).collect();
    for (i, x) in m.iter_mut().enumerate() {
        let a = im.alfa(i);
        if a == 255 {
            continue;
        }
        if a == 0 || pal.is_empty() {
            *x = Mezcla {
                a: transparente,
                b: transparente,
                t: 1.0,
            };
            continue;
        }
        let t = a as f32 / 255.0;
        let plano = f3(im.color(i as u32 % im.ancho, i as u32 / im.ancho));
        let c = plano.map(|v| ((v - 255.0 * (1.0 - t)) / t).clamp(0.0, 255.0));
        let k = (0..pal.len())
            .min_by(|&p, &q| dist2(pal[p], c).total_cmp(&dist2(pal[q], c)))
            .unwrap_or(0) as u16;
        *x = Mezcla {
            a: k,
            b: transparente,
            t,
        };
    }
}

/// Cobertura de la capa `dentro` en cada píxel.
pub fn cobertura(m: &[Mezcla], dentro: &[bool]) -> Vec<f32> {
    m.iter()
        .map(|x| {
            (if dentro[x.a as usize] { x.t } else { 0.0 })
                + (if dentro[x.b as usize] { 1.0 - x.t } else { 0.0 })
        })
        .collect()
}

type Pt = (f64, f64);
/// Arista de la cuadrícula: (i, j, 0) horizontal de (i,j) a (i+1,j); (i, j, 1) vertical de (i,j) a (i,j+1).
type Arista = (i64, i64, u8);

/// Línea de nivel 0,5 de `v` (ancho×alto, en los centros de los píxeles), con un borde de 0 alrededor
/// para que todo contorno se cierre. Devuelve polígonos cerrados en coordenadas de la imagen.
pub fn contornos(v: &[f32], ancho: usize, alto: usize) -> Vec<Vec<Pt>> {
    let (w, h) = (ancho as i64 + 2, alto as i64 + 2);
    let val = |i: i64, j: i64| -> f32 {
        if i <= 0 || j <= 0 || i >= w - 1 || j >= h - 1 {
            0.0
        } else {
            v[((j - 1) * ancho as i64 + (i - 1)) as usize]
        }
    };
    // Aristas: (i, j, 0) horizontal de (i,j) a (i+1,j); (i, j, 1) vertical de (i,j) a (i,j+1).
    let punto = |e: (i64, i64, u8)| -> Pt {
        let (i, j, d) = e;
        let (a, (i2, j2)) = (val(i, j), if d == 0 { (i + 1, j) } else { (i, j + 1) });
        let b = val(i2, j2);
        let t = if (b - a).abs() < 1e-6 {
            0.5
        } else {
            ((0.5 - a) / (b - a)).clamp(0.0, 1.0)
        } as f64;
        let (x0, y0) = (i as f64 - 0.5, j as f64 - 0.5);
        if d == 0 { (x0 + t, y0) } else { (x0, y0 + t) }
    };
    let mut vecinos: HashMap<Arista, Vec<Arista>> = HashMap::new();
    let mut une = |a: (i64, i64, u8), b: (i64, i64, u8)| {
        vecinos.entry(a).or_default().push(b);
        vecinos.entry(b).or_default().push(a);
    };
    for j in 0..h - 1 {
        for i in 0..w - 1 {
            let (v00, v10, v01, v11) = (val(i, j), val(i + 1, j), val(i, j + 1), val(i + 1, j + 1));
            let k = (v00 >= 0.5) as u8
                | ((v10 >= 0.5) as u8) << 1
                | ((v11 >= 0.5) as u8) << 2
                | ((v01 >= 0.5) as u8) << 3;
            let (arriba, derecha, abajo, izquierda) =
                ((i, j, 0), (i + 1, j, 1), (i, j + 1, 0), (i, j, 1));
            match k {
                0 | 15 => {}
                1 | 14 => une(izquierda, arriba),
                2 | 13 => une(arriba, derecha),
                3 | 12 => une(izquierda, derecha),
                4 | 11 => une(derecha, abajo),
                6 | 9 => une(arriba, abajo),
                7 | 8 => une(izquierda, abajo),
                5 | 10 => {
                    let centro = (v00 + v10 + v01 + v11) / 4.0 >= 0.5;
                    if (k == 5) == centro {
                        une(izquierda, abajo);
                        une(arriba, derecha);
                    } else {
                        une(izquierda, arriba);
                        une(derecha, abajo);
                    }
                }
                _ => unreachable!(),
            }
        }
    }
    let mut hechos: std::collections::HashSet<(i64, i64, u8)> = std::collections::HashSet::new();
    let mut claves: Vec<_> = vecinos.keys().copied().collect();
    claves.sort();
    let mut lazos = Vec::new();
    for ini in claves {
        if hechos.contains(&ini) {
            continue;
        }
        let mut lazo = vec![punto(ini)];
        hechos.insert(ini);
        let (mut prev, mut cur) = (ini, vecinos[&ini][0]);
        while cur != ini {
            if !hechos.insert(cur) {
                break;
            }
            lazo.push(punto(cur));
            let n = &vecinos[&cur];
            let sig = if n[0] == prev && n.len() > 1 {
                n[1]
            } else {
                n[0]
            };
            prev = cur;
            cur = sig;
        }
        if lazo.len() >= 3 {
            lazos.push(lazo);
        }
    }
    lazos
}

/// Douglas-Peucker sobre un polígono cerrado.
pub fn simplificar(p: &[Pt], tol: f64) -> Vec<Pt> {
    if p.len() < 4 {
        return p.to_vec();
    }
    fn dp(p: &[Pt], tol: f64, out: &mut Vec<Pt>) {
        let (a, b) = (p[0], p[p.len() - 1]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let l = (dx * dx + dy * dy).sqrt().max(1e-9);
        let (mut im, mut dm) = (0, 0.0);
        for (i, q) in p.iter().enumerate().take(p.len() - 1).skip(1) {
            let d = ((q.0 - a.0) * dy - (q.1 - a.1) * dx).abs() / l;
            if d > dm {
                dm = d;
                im = i;
            }
        }
        if dm > tol {
            dp(&p[..=im], tol, out);
            out.pop();
            dp(&p[im..], tol, out);
        } else {
            out.push(a);
            out.push(b);
        }
    }
    // Se parte por el punto más lejano del primero, para que el cierre no dependa del arranque.
    let lejos = (1..p.len())
        .max_by(|&i, &j| {
            let d = |k: usize| (p[k].0 - p[0].0).powi(2) + (p[k].1 - p[0].1).powi(2);
            d(i).total_cmp(&d(j))
        })
        .unwrap_or(p.len() / 2);
    let mut a: Vec<Pt> = p[..=lejos].to_vec();
    let mut b: Vec<Pt> = p[lejos..].to_vec();
    b.push(p[0]);
    let mut out = Vec::new();
    dp(&a, tol, &mut out);
    out.pop();
    let mut otra = Vec::new();
    dp(&b, tol, &mut otra);
    out.extend(otra);
    out.pop();
    a.clear();
    out
}

/// Suavizado de Taubin (λ|μ) sobre un lazo cerrado: quita el serrado de la cuadrícula sin encoger la forma.
/// Los puntos donde el contorno gira mucho (esquinas) no se mueven.
pub fn suavizar(p: &[Pt], pasadas: usize) -> Vec<Pt> {
    let n = p.len();
    if n < 8 || pasadas == 0 {
        return p.to_vec();
    }
    // Esquinas: giro de más de 35° entre los vecinos a 2 puntos.
    let fijo: Vec<bool> = (0..n)
        .map(|i| {
            let (a, b, c) = (p[(i + n - 2) % n], p[i], p[(i + 2) % n]);
            let (u, v) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
            let (lu, lv) = (
                (u.0 * u.0 + u.1 * u.1).sqrt(),
                (v.0 * v.0 + v.1 * v.1).sqrt(),
            );
            if lu < 1e-9 || lv < 1e-9 {
                return true;
            }
            ((u.0 * v.0 + u.1 * v.1) / (lu * lv))
                .clamp(-1.0, 1.0)
                .acos()
                .to_degrees()
                > 35.0
        })
        .collect();
    let mut q = p.to_vec();
    for _ in 0..pasadas {
        for k in [0.5, -0.53] {
            let r = q.clone();
            for i in 0..n {
                if fijo[i] {
                    continue;
                }
                let (a, c) = (r[(i + n - 1) % n], r[(i + 1) % n]);
                q[i] = (
                    r[i].0 + k * ((a.0 + c.0) / 2.0 - r[i].0),
                    r[i].1 + k * ((a.1 + c.1) / 2.0 - r[i].1),
                );
            }
        }
    }
    q
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_borde_de_un_trazo_fino_es_mezcla_con_su_color() {
        // Un trazo negro de un píxel sobre blanco, con su borde gris arriba y abajo. La paleta tiene el negro (de
        // otra parte del dibujo) y un naranja; el gris se explicaba con el blanco y el naranja, que es el color
        // más parecido, y no con el negro del centro del trazo, que no es plano.
        let (w, h) = (9u32, 5u32);
        let mut px = Vec::new();
        for y in 0..h {
            for _ in 0..w {
                let v = match y {
                    2 => 0,
                    1 | 3 => 128,
                    _ => 255,
                };
                px.extend([v, v, v, 255]);
            }
        }
        let im = Rgba::nueva(w, h, px);
        let pal = [[255, 255, 255], [0, 0, 0], [200, 120, 60]];
        let mut m = vec![Mezcla { a: 0, b: 0, t: 1.0 }; (w * h) as usize];
        for x in 0..w as usize {
            m[2 * w as usize + x] = Mezcla { a: 1, b: 1, t: 1.0 };
            for y in [1, 3] {
                m[y * w as usize + x] = Mezcla { a: 2, b: 0, t: 0.6 };
            }
        }
        let cambiados = afinar(&im, &pal, &mut m, 2, &[None, None, None]);
        assert_eq!(cambiados, 2 * w as usize);
        let g = m[w as usize + 4];
        assert!(
            (g.a, g.b) == (1, 0) || (g.a, g.b) == (0, 1),
            "el gris, de blanco y negro: {}-{}",
            g.a,
            g.b
        );
        // Con el color de un degradado, el píxel no se toca.
        let mut m2 = m.clone();
        m2[w as usize + 4] = Mezcla { a: 2, b: 0, t: 0.6 };
        let degradado = Some(crate::modelo::Relleno::Color([200, 120, 60]));
        afinar(&im, &pal, &mut m2, 2, &[None, None, degradado]);
        assert_eq!(m2[w as usize + 4].a, 2);
    }
}
