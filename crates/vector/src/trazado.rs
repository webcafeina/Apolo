//! Lo que comparten los escritores: las formas como Bézier, aplanadas a rectas o aproximadas por arcos, y
//! los bordes de cada capa sin repetir (el estilo «trazar bordes»).

use crate::geometria::{P, add, dist, dot, mul, sub};
use crate::modelo::{Documento, Forma, Relleno, Tramo};

/// Un cuarto de círculo como cúbica: los puntos de control a esta fracción del radio (error del 0,027 %).
const KAPPA: f64 = 0.552_284_749_830_793_4;

/// Los cuatro cuartos de una elipse como cúbicas, desde el extremo del semieje `rx`, en el mismo sentido
/// que el SVG (cuatro cuartos, docs/trampas.md).
pub fn elipse_en_cubicas(c: P, rx: f64, ry: f64, ang: f64) -> Vec<Tramo> {
    let (s, co) = ang.sin_cos();
    let pto = |u: f64, v: f64| (c.0 + u * co - v * s, c.1 + u * s + v * co);
    // Extremos y tangentes de cada cuarto en el sistema de la elipse.
    let ext = [(rx, 0.0), (0.0, ry), (-rx, 0.0), (0.0, -ry)];
    let tan = [(0.0, ry), (-rx, 0.0), (0.0, -ry), (rx, 0.0)];
    (0..4)
        .map(|i| {
            let j = (i + 1) % 4;
            let (a, b) = (ext[i], ext[j]);
            let (ta, tb) = (tan[i], tan[j]);
            Tramo::Cubica(
                pto(a.0, a.1),
                pto(a.0 + ta.0 * KAPPA, a.1 + ta.1 * KAPPA),
                pto(b.0 - tb.0 * KAPPA, b.1 - tb.1 * KAPPA),
                pto(b.0, b.1),
            )
        })
        .collect()
}

/// La forma como tramos de rectas y cúbicas.
pub fn tramos(f: &Forma) -> Vec<Tramo> {
    match f {
        Forma::Circulo { c, r } => elipse_en_cubicas(*c, *r, *r, 0.0),
        Forma::Elipse { c, rx, ry, ang } => elipse_en_cubicas(*c, *rx, *ry, *ang),
        Forma::Tramos(t) => t.clone(),
    }
}

fn punto(a: P, b: P, c: P, d: P, t: f64) -> P {
    let u = 1.0 - t;
    add(
        add(mul(a, u * u * u), mul(b, 3.0 * u * u * t)),
        add(mul(c, 3.0 * u * t * t), mul(d, t * t * t)),
    )
}

/// De Casteljau en t = 0,5.
fn partir(a: P, b: P, c: P, d: P) -> ([P; 4], [P; 4]) {
    let m = |p: P, q: P| mul(add(p, q), 0.5);
    let (ab, bc, cd) = (m(a, b), m(b, c), m(c, d));
    let (abc, bcd) = (m(ab, bc), m(bc, cd));
    let x = m(abc, bcd);
    ([a, ab, abc, x], [x, bcd, cd, d])
}

/// Distancia de `p` a la recta a–b (o al punto, si coinciden).
fn a_recta(p: P, a: P, b: P) -> f64 {
    let d = sub(b, a);
    let l = dot(d, d);
    if l < 1e-18 {
        return dist(p, a);
    }
    ((p.0 - a.0) * d.1 - (p.1 - a.1) * d.0).abs() / l.sqrt()
}

fn aplanar_cubica(q: [P; 4], tol: f64, prof: u32, out: &mut Vec<P>) {
    let plana = a_recta(q[1], q[0], q[3]).max(a_recta(q[2], q[0], q[3])) <= tol;
    if plana || prof >= 16 {
        out.push(q[3]);
        return;
    }
    let (a, b) = partir(q[0], q[1], q[2], q[3]);
    aplanar_cubica(a, tol, prof + 1, out);
    aplanar_cubica(b, tol, prof + 1, out);
}

/// Los tramos como polilínea, con las curvas a menos de `tol` de la recta. Incluye el primer punto.
pub fn aplanar(t: &[Tramo], tol: f64) -> Vec<P> {
    let mut out = Vec::new();
    if let Some(s) = t.first() {
        out.push(s.ini());
    }
    for s in t {
        match *s {
            Tramo::Linea(_, b) => out.push(b),
            Tramo::Cubica(a, b, c, d) => aplanar_cubica([a, b, c, d], tol, 0, &mut out),
        }
    }
    out
}

/// Un trozo de polilínea con arco: de un punto al siguiente con un abombamiento (`bulge` de DXF:
/// tan(ángulo / 4), positivo en sentido antihorario con el eje Y hacia arriba; 0 es recta).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertice {
    pub p: P,
    pub bulge: f64,
}

/// El círculo que pasa por tres puntos.
fn circulo_por(a: P, b: P, c: P) -> Option<(P, f64)> {
    let d = 2.0 * (a.0 * (b.1 - c.1) + b.0 * (c.1 - a.1) + c.0 * (a.1 - b.1));
    if d.abs() < 1e-12 {
        return None;
    }
    let (a2, b2, c2) = (dot(a, a), dot(b, b), dot(c, c));
    let o = (
        (a2 * (b.1 - c.1) + b2 * (c.1 - a.1) + c2 * (a.1 - b.1)) / d,
        (a2 * (c.0 - b.0) + b2 * (a.0 - c.0) + c2 * (b.0 - a.0)) / d,
    );
    Some((o, dist(o, a)))
}

/// El abombamiento del arco de `a` a `b` que pasa por `m`.
fn bulge(a: P, m: P, b: P) -> f64 {
    let Some((o, _)) = circulo_por(a, m, b) else {
        return 0.0;
    };
    let ang = |p: P| (p.1 - o.1).atan2(p.0 - o.0);
    let giro = (m.0 - a.0) * (b.1 - a.1) - (m.1 - a.1) * (b.0 - a.0);
    // giro > 0: el triángulo a, m, b gira a la izquierda, así que el arco va en sentido antihorario.
    let tau = std::f64::consts::TAU;
    let barrido = if giro > 0.0 {
        (ang(b) - ang(a)).rem_euclid(tau)
    } else {
        -(ang(a) - ang(b)).rem_euclid(tau)
    };
    (barrido / 4.0).tan()
}

fn arcos_cubica(q: [P; 4], tol: f64, prof: u32, out: &mut Vec<Vertice>) {
    let m = punto(q[0], q[1], q[2], q[3], 0.5);
    let recta = a_recta(q[1], q[0], q[3]).max(a_recta(q[2], q[0], q[3])) <= tol;
    if recta {
        out.push(Vertice {
            p: q[0],
            bulge: 0.0,
        });
        return;
    }
    if let Some((o, r)) = circulo_por(q[0], m, q[3]) {
        let error = [0.125, 0.25, 0.375, 0.625, 0.75, 0.875]
            .iter()
            .map(|&t| (dist(punto(q[0], q[1], q[2], q[3], t), o) - r).abs())
            .fold(0.0, f64::max);
        if error <= tol || prof >= 12 {
            out.push(Vertice {
                p: q[0],
                bulge: bulge(q[0], m, q[3]),
            });
            return;
        }
    } else if prof >= 12 {
        out.push(Vertice {
            p: q[0],
            bulge: 0.0,
        });
        return;
    }
    let (a, b) = partir(q[0], q[1], q[2], q[3]);
    arcos_cubica(a, tol, prof + 1, out);
    arcos_cubica(b, tol, prof + 1, out);
}

/// Los tramos como polilínea de rectas y arcos a menos de `tol` de la curva. Cada vértice lleva el
/// abombamiento del trozo que sale de él; no incluye el último punto (en una figura cerrada es el primero).
/// Los puntos ya tienen que estar con el eje Y hacia arriba para que el signo sea el de DXF.
pub fn en_arcos(t: &[Tramo], tol: f64) -> Vec<Vertice> {
    let mut out = Vec::new();
    for s in t {
        match *s {
            Tramo::Linea(a, _) => out.push(Vertice { p: a, bulge: 0.0 }),
            Tramo::Cubica(a, b, c, d) => arcos_cubica([a, b, c, d], tol, 0, &mut out),
        }
    }
    out
}

/// Aplica `f` a cada punto de los tramos.
pub fn transformar(t: &[Tramo], f: impl Fn(P) -> P) -> Vec<Tramo> {
    t.iter()
        .map(|s| match *s {
            Tramo::Linea(a, b) => Tramo::Linea(f(a), f(b)),
            Tramo::Cubica(a, b, c, d) => Tramo::Cubica(f(a), f(b), f(c), f(d)),
        })
        .collect()
}

/// Un trazo abierto (o cerrado, si acaba donde empieza) con su color: lo que dibuja «trazar bordes».
#[derive(Clone, Debug, PartialEq)]
pub struct Borde {
    pub color: [u8; 3],
    pub tramos: Vec<Tramo>,
    pub cerrado: bool,
}

/// Separación por debajo de la cual dos bordes se consideran el mismo (en píxeles).
const MISMO_BORDE: f64 = 0.75;

/// Un tramo a menos de esto del marco de la imagen va por el marco (en píxeles).
const MARCO: f64 = 0.6;

/// Los bordes de las capas, cada uno una vez. Las capas están apiladas (cada una contiene a las de
/// encima), así que donde un color de arriba toca uno de abajo, los contornos de varias capas pasan casi por
/// el mismo sitio: se recorren de arriba abajo y se quita cada tramo cuyos puntos caen todos a menos de
/// 0,75 px de un tramo ya dibujado. Es por tramos: un tramo que coincide solo en parte se queda entero.
///
/// Fuera también los tramos que van por el marco de la imagen (sería cortar el borde de la hoja). El color es
/// negro: un borde separa dos colores, y el de la capa de encima puede ser el del fondo (blanco sobre
/// blanco); para cortar no importa, y quien lo quiera de otro color lo pide en las opciones.
pub fn bordes(doc: &Documento) -> Vec<Borde> {
    let (w, h) = (doc.ancho as f64, doc.alto as f64);
    let en_marco = |p: &P| p.0 < MARCO || p.1 < MARCO || p.0 > w - MARCO || p.1 > h - MARCO;
    let celda = 2.0;
    let mut rejilla: std::collections::HashMap<(i64, i64), Vec<P>> = Default::default();
    let clave = |p: P| ((p.0 / celda).floor() as i64, (p.1 / celda).floor() as i64);
    let cerca = |rejilla: &std::collections::HashMap<(i64, i64), Vec<P>>, p: P| {
        let (i, j) = clave(p);
        (i - 1..=i + 1).any(|a| {
            (j - 1..=j + 1).any(|b| {
                rejilla
                    .get(&(a, b))
                    .is_some_and(|v| v.iter().any(|q| dist(*q, p) < MISMO_BORDE))
            })
        })
    };
    let muestras = |s: &Tramo, paso: f64| -> Vec<P> {
        let pts = aplanar(std::slice::from_ref(s), 0.05);
        let mut out = vec![pts[0]];
        for w in pts.windows(2) {
            let n = (dist(w[0], w[1]) / paso).ceil().max(1.0) as usize;
            for k in 1..=n {
                out.push(add(w[0], mul(sub(w[1], w[0]), k as f64 / n as f64)));
            }
        }
        out
    };
    let mut salida = Vec::new();
    for capa in doc.capas.iter().rev() {
        let color = [0, 0, 0];
        for f in &capa.formas {
            let t = tramos(f);
            let quedan: Vec<bool> = t
                .iter()
                .map(|s| {
                    let m = muestras(s, 0.5);
                    !m.iter().all(|p| cerca(&rejilla, *p)) && !m.iter().all(en_marco)
                })
                .collect();
            for (s, q) in t.iter().zip(&quedan) {
                if *q {
                    for p in muestras(s, 0.3) {
                        rejilla.entry(clave(p)).or_default().push(p);
                    }
                }
            }
            // Los tramos que quedan, en cadenas seguidas (girando para empezar tras un hueco).
            let n = t.len();
            if quedan.iter().all(|q| *q) {
                salida.push(Borde {
                    color,
                    tramos: t,
                    cerrado: true,
                });
                continue;
            }
            let Some(hueco) = quedan.iter().position(|q| !q) else {
                continue;
            };
            let mut cadena: Vec<Tramo> = Vec::new();
            for k in 1..=n {
                let i = (hueco + k) % n;
                if quedan[i] {
                    cadena.push(t[i]);
                } else if !cadena.is_empty() {
                    salida.push(Borde {
                        color,
                        tramos: std::mem::take(&mut cadena),
                        cerrado: false,
                    });
                }
            }
            if !cadena.is_empty() {
                salida.push(Borde {
                    color,
                    tramos: cadena,
                    cerrado: false,
                });
            }
        }
    }
    // De abajo arriba, como las capas.
    salida.reverse();
    salida
}

/// El color con el que se traza una capa: el suyo, o el medio si es un degradado.
pub fn color_de(r: &Relleno) -> [u8; 3] {
    r.color_medio()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn circulo_en_cubicas() {
        let t = elipse_en_cubicas((10.0, 10.0), 5.0, 5.0, 0.0);
        assert_eq!(t.len(), 4);
        for s in &t {
            if let Tramo::Cubica(a, b, c, d) = *s {
                for k in 0..=10 {
                    let p = punto(a, b, c, d, k as f64 / 10.0);
                    assert!((dist(p, (10.0, 10.0)) - 5.0).abs() < 0.002, "{p:?}");
                }
            }
        }
        assert_eq!(t[0].ini(), t[3].fin());
    }

    #[test]
    fn arcos_de_un_circulo() {
        // Un círculo (en cúbicas) se aproxima con pocos arcos, todos en el mismo sentido.
        let t = elipse_en_cubicas((0.0, 0.0), 50.0, 50.0, 0.0);
        let v = en_arcos(&t, 0.05);
        assert!(v.len() <= 8, "{}", v.len());
        assert!(v.iter().all(|x| x.bulge > 0.0), "{v:?}");
        // La suma de los ángulos da una vuelta.
        let vuelta: f64 = v.iter().map(|x| 4.0 * x.bulge.atan()).sum();
        assert!((vuelta - std::f64::consts::TAU).abs() < 1e-3, "{vuelta}");
    }

    #[test]
    fn aplanar_cerca_de_la_curva() {
        let t = elipse_en_cubicas((0.0, 0.0), 20.0, 10.0, 0.4);
        let p = aplanar(&t, 0.1);
        assert!(p.len() > 8);
        assert_eq!(p.first(), p.last());
    }
}
