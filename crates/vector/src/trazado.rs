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

/// El punto de parámetro `t` del tramo.
fn en(s: &Tramo, t: f64) -> P {
    match *s {
        Tramo::Linea(a, b) => add(a, mul(sub(b, a), t)),
        Tramo::Cubica(a, b, c, d) => punto(a, b, c, d, t),
    }
}

/// De Casteljau en `t`.
fn partir_en(q: [P; 4], t: f64) -> ([P; 4], [P; 4]) {
    let m = |p: P, r: P| add(p, mul(sub(r, p), t));
    let (ab, bc, cd) = (m(q[0], q[1]), m(q[1], q[2]), m(q[2], q[3]));
    let (abc, bcd) = (m(ab, bc), m(bc, cd));
    let x = m(abc, bcd);
    ([q[0], ab, abc, x], [x, bcd, cd, q[3]])
}

/// El trozo del tramo entre los parámetros `a` y `b` (0 ≤ a < b ≤ 1). Los extremos 0 y 1 salen exactos, para
/// que los trozos sigan pegados a los tramos de al lado.
fn trozo(s: &Tramo, a: f64, b: f64) -> Tramo {
    match *s {
        Tramo::Linea(p, q) => Tramo::Linea(
            if a <= 0.0 { p } else { en(s, a) },
            if b >= 1.0 { q } else { en(s, b) },
        ),
        Tramo::Cubica(p0, p1, p2, p3) => {
            let mut c = [p0, p1, p2, p3];
            if b < 1.0 {
                c = partir_en(c, b).0;
            }
            if a > 0.0 {
                c = partir_en(c, a / b).1;
            }
            Tramo::Cubica(c[0], c[1], c[2], c[3])
        }
    }
}

/// Un trozo de borde que se queda: de qué tramo de la forma es y entre qué parámetros.
struct Trozo {
    i: usize,
    a: f64,
    b: f64,
}

/// Paso de las muestras con que se mira si un tramo coincide con lo ya dibujado (en píxeles).
const PASO: f64 = 0.5;

/// Un tramo que vuelve a coincidir con lo dibujado menos de esto (al cruzarse con otro borde, por ejemplo)
/// no se parte ahí (en píxeles).
const COINCIDE_POCO: f64 = 2.0;

/// Un trozo que se aparta de lo dibujado menos de esto, entre dos que coinciden, no se dibuja: es el mismo
/// borde con un poco de temblor (en píxeles).
const SE_APARTA_POCO: f64 = 1.0;

/// Lo que se alarga un trozo dentro de lo que coincide, buscando el punto más cercano a lo dibujado, para que
/// el corte llegue hasta el otro borde y no deje un puente (en píxeles).
const ALARGAR: f64 = 2.0;

/// Los bordes de las capas, cada uno una vez. Las capas están apiladas (cada una contiene a las de
/// encima), así que donde un color de arriba toca uno de abajo, los contornos de varias capas pasan casi por
/// el mismo sitio: se recorren de arriba abajo y, de cada tramo, se quitan los trozos que caen a menos de
/// 0,75 px de un borde ya dibujado. Un tramo que coincide solo en parte se parte donde deja de coincidir, y el
/// trozo que queda se alarga hasta el punto más cercano del otro borde: cada borde se corta una vez, sin
/// huecos donde se juntan.
///
/// Fuera también los trozos que van por el marco de la imagen (sería cortar el borde de la hoja). El color es
/// negro: un borde separa dos colores, y el de la capa de encima puede ser el del fondo (blanco sobre
/// blanco); para cortar no importa, y quien lo quiera de otro color lo pide en las opciones.
pub fn bordes(doc: &Documento) -> Vec<Borde> {
    let (w, h) = (doc.ancho as f64, doc.alto as f64);
    let en_marco = |p: &P| p.0 < MARCO || p.1 < MARCO || p.0 > w - MARCO || p.1 > h - MARCO;
    let celda = 2.0;
    let mut rejilla: std::collections::HashMap<(i64, i64), Vec<P>> = Default::default();
    let clave = |p: P| ((p.0 / celda).floor() as i64, (p.1 / celda).floor() as i64);
    // La distancia a lo ya dibujado y el punto más cercano, si está a menos de una celda.
    let cerca =
        |rejilla: &std::collections::HashMap<(i64, i64), Vec<P>>, p: P| -> Option<(f64, P)> {
            let (i, j) = clave(p);
            let mut m: Option<(f64, P)> = None;
            for a in i - 1..=i + 1 {
                for b in j - 1..=j + 1 {
                    for q in rejilla.get(&(a, b)).into_iter().flatten() {
                        let d = dist(*q, p);
                        if m.is_none_or(|x| d < x.0) {
                            m = Some((d, *q));
                        }
                    }
                }
            }
            m.filter(|x| x.0 <= celda)
        };
    let largo = |s: &Tramo| {
        aplanar(std::slice::from_ref(s), 0.05)
            .windows(2)
            .map(|w| dist(w[0], w[1]))
            .sum::<f64>()
    };
    let mut salida = Vec::new();
    for capa in doc.capas.iter().rev() {
        let color = [0, 0, 0];
        for f in &capa.formas {
            let t = tramos(f);
            let mut trozos: Vec<Trozo> = Vec::new();
            for (i, s) in t.iter().enumerate() {
                let l = largo(s);
                let n = (l / PASO).ceil().max(1.0) as usize;
                let paso = l / n as f64;
                let ts: Vec<f64> = (0..=n).map(|k| k as f64 / n as f64).collect();
                let d: Vec<Option<f64>> = ts
                    .iter()
                    .map(|&u| cerca(&rejilla, en(s, u)).map(|x| x.0))
                    .collect();
                let mut fuera: Vec<bool> = ts
                    .iter()
                    .zip(&d)
                    .map(|(&u, d)| !(d.is_some_and(|d| d < MISMO_BORDE) || en_marco(&en(s, u))))
                    .collect();
                // Los tramos de rachas: (desde, hasta, fuera), con índices de muestra.
                let rachas = |fuera: &[bool]| {
                    let mut r: Vec<(usize, usize, bool)> = Vec::new();
                    for (k, &x) in fuera.iter().enumerate() {
                        match r.last_mut() {
                            Some(u) if u.2 == x => u.1 = k,
                            _ => r.push((k, k, x)),
                        }
                    }
                    r
                };
                // Coincidir poco entre dos trozos de fuera (un cruce) no parte; apartarse poco entre dos que
                // coinciden (temblor), tampoco cuenta.
                let r = rachas(&fuera);
                for (m, &(de, a, x)) in r.iter().enumerate() {
                    let dentro = m > 0 && m + 1 < r.len();
                    let medida = (a - de) as f64 * paso;
                    if dentro && ((!x && medida < COINCIDE_POCO) || (x && medida < SE_APARTA_POCO))
                    {
                        fuera[de..=a].fill(!x);
                    }
                }
                // Cada racha de fuera, alargada hacia los lados hasta lo más cercano a lo dibujado.
                let alargar = ((ALARGAR / paso).round() as usize).max(1);
                // El primero, desde el trozo, que está casi tan cerca de lo dibujado como el que más: alargar
                // más solo repetiría borde.
                let mejor = |ks: &mut dyn Iterator<Item = usize>| -> Option<usize> {
                    let v: Vec<(usize, f64)> = ks.map_while(|k| d[k].map(|dk| (k, dk))).collect();
                    let min = v.iter().map(|x| x.1).fold(f64::INFINITY, f64::min);
                    v.iter().find(|x| x.1 <= min + 0.05).map(|x| x.0)
                };
                for (de, a, x) in rachas(&fuera) {
                    if !x {
                        continue;
                    }
                    let ini = if de == 0 {
                        0
                    } else {
                        mejor(&mut (de.saturating_sub(alargar)..de).rev()).unwrap_or(de - 1)
                    };
                    let fin = if a == n {
                        n
                    } else {
                        mejor(&mut (a + 1..=(a + alargar).min(n))).unwrap_or(a + 1)
                    };
                    trozos.push(Trozo {
                        i,
                        a: ts[ini],
                        b: ts[fin],
                    });
                }
            }
            // Los trozos, en cadenas seguidas: uno sigue al anterior si este acaba su tramo y aquel empieza el
            // siguiente.
            let n = t.len();
            let entero = trozos.len() == n && trozos.iter().all(|z| z.a <= 0.0 && z.b >= 1.0);
            let dibujar = |rejilla: &mut std::collections::HashMap<(i64, i64), Vec<P>>| {
                for z in &trozos {
                    let s = trozo(&t[z.i], z.a, z.b);
                    let l = largo(&s);
                    let n = (l / 0.3).ceil().max(1.0) as usize;
                    for k in 0..=n {
                        let p = en(&s, k as f64 / n as f64);
                        rejilla.entry(clave(p)).or_default().push(p);
                    }
                }
            };
            if entero {
                dibujar(&mut rejilla);
                salida.push(Borde {
                    color,
                    tramos: t,
                    cerrado: true,
                });
                continue;
            }
            let mut cadenas: Vec<Vec<Tramo>> = Vec::new();
            let mut previo: Option<&Trozo> = None;
            for z in &trozos {
                let sigue = previo.is_some_and(|p| p.b >= 1.0 && z.a <= 0.0 && z.i == p.i + 1);
                if !sigue {
                    cadenas.push(Vec::new());
                }
                cadenas.last_mut().unwrap().push(trozo(&t[z.i], z.a, z.b));
                previo = Some(z);
            }
            // La última cadena sigue en la primera si la forma da la vuelta por el primer tramo.
            if cadenas.len() > 1
                && trozos.first().is_some_and(|z| z.i == 0 && z.a <= 0.0)
                && trozos.last().is_some_and(|z| z.i == n - 1 && z.b >= 1.0)
            {
                let ultima = cadenas.pop().unwrap();
                let primera = std::mem::take(&mut cadenas[0]);
                cadenas[0] = ultima.into_iter().chain(primera).collect();
            }
            // Cada cadena acaba donde dejó de coincidir, a menos de 0,75 px del otro borde: un tramo recto
            // hasta su punto más cercano la pega a él, para que el corte no deje un puente.
            let pegar = |p: P| {
                cerca(&rejilla, p)
                    .filter(|x| x.0 > 0.02 && x.0 < MISMO_BORDE + 0.25 && !en_marco(&p))
                    .map(|x| x.1)
            };
            for c in &mut cadenas {
                if let Some(q) = pegar(c[0].ini()) {
                    c.insert(0, Tramo::Linea(q, c[0].ini()));
                }
                let fin = c[c.len() - 1].fin();
                if let Some(q) = pegar(fin) {
                    c.push(Tramo::Linea(fin, q));
                }
            }
            dibujar(&mut rejilla);
            for c in cadenas {
                salida.push(Borde {
                    color,
                    tramos: c,
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

    fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Forma {
        Forma::Tramos(vec![
            Tramo::Linea((x0, y0), (x1, y0)),
            Tramo::Linea((x1, y0), (x1, y1)),
            Tramo::Linea((x1, y1), (x0, y1)),
            Tramo::Linea((x0, y1), (x0, y0)),
        ])
    }

    fn doc(capas: Vec<Vec<Forma>>) -> Documento {
        Documento {
            ancho: 100,
            alto: 100,
            fondo: None,
            capas: capas
                .into_iter()
                .map(|formas| crate::modelo::Capa {
                    relleno: Relleno::Color([0, 0, 0]),
                    formas,
                })
                .collect(),
        }
    }

    /// Cuántos bordes pasan a menos de 0,3 px de `p`.
    fn veces(b: &[Borde], p: P) -> usize {
        let a_segmento = |p: P, a: P, c: P| {
            let d = sub(c, a);
            let l = dot(d, d);
            let u = if l < 1e-18 {
                0.0
            } else {
                (dot(sub(p, a), d) / l).clamp(0.0, 1.0)
            };
            dist(p, add(a, mul(d, u)))
        };
        b.iter()
            .filter(|b| {
                aplanar(&b.tramos, 0.02)
                    .windows(2)
                    .any(|w| a_segmento(p, w[0], w[1]) < 0.3)
            })
            .count()
    }

    #[test]
    fn lado_compartido_en_parte_una_vez() {
        // El cuadrado de arriba se apoya en el lado de arriba del rectángulo: ese lado es un solo tramo, que
        // coincide solo de x = 20 a 30. Antes se quedaba entero y ese trozo salía dos veces.
        let b = bordes(&doc(vec![
            vec![rect(10.0, 10.0, 60.0, 50.0)],
            vec![rect(20.0, 10.0, 30.0, 20.0)],
        ]));
        for p in [(25.0, 10.0), (21.0, 10.0), (29.0, 10.0)] {
            assert_eq!(veces(&b, p), 1, "{p:?}: {b:?}");
        }
        // Lo demás, una vez, y sin huecos donde se juntan.
        for p in [
            (15.0, 10.0),
            (45.0, 10.0),
            (25.0, 20.0),
            (60.0, 30.0),
            (10.0, 30.0),
            (20.0, 10.0),
            (30.0, 10.0),
        ] {
            assert!(veces(&b, p) >= 1, "{p:?}: {b:?}");
        }
        for p in [(15.0, 10.0), (45.0, 10.0), (25.0, 20.0), (60.0, 30.0)] {
            assert_eq!(veces(&b, p), 1, "{p:?}: {b:?}");
        }
        extremos_pegados(&b);
    }

    #[test]
    fn curva_compartida_en_parte_una_vez() {
        // Una cuña que comparte la primera mitad del primer cuarto de una elipse.
        let e = elipse_en_cubicas((50.0, 50.0), 30.0, 20.0, 0.0);
        let mitad = trozo(&e[0], 0.0, 0.5);
        let cuna = Forma::Tramos(vec![
            mitad,
            Tramo::Linea(mitad.fin(), (50.0, 50.0)),
            Tramo::Linea((50.0, 50.0), mitad.ini()),
        ]);
        let abajo = Forma::Elipse {
            c: (50.0, 50.0),
            rx: 30.0,
            ry: 20.0,
            ang: 0.0,
        };
        let b = bordes(&doc(vec![vec![abajo], vec![cuna]]));
        for u in [0.1, 0.25, 0.4, 0.6, 0.75, 0.9] {
            let p = en(&e[0], u);
            assert_eq!(veces(&b, p), 1, "t = {u}, {p:?}");
        }
        // El trozo partido de la elipse acaba donde empieza el de la cuña.
        assert!(veces(&b, en(&e[0], 0.5)) >= 1);
        extremos_pegados(&b);
    }

    /// Cada borde abierto empieza y acaba sobre otro borde: si no, al cortar quedaría un puente.
    fn extremos_pegados(b: &[Borde]) {
        for (i, x) in b.iter().filter(|x| !x.cerrado).enumerate() {
            for p in [x.tramos[0].ini(), x.tramos[x.tramos.len() - 1].fin()] {
                let otros: Vec<Borde> = b
                    .iter()
                    .filter(|y| !y.cerrado)
                    .enumerate()
                    .filter(|(j, _)| *j != i)
                    .map(|(_, y)| y.clone())
                    .chain(b.iter().filter(|y| y.cerrado).cloned())
                    .collect();
                assert!(veces(&otros, p) >= 1, "{p:?} suelto: {b:?}");
            }
        }
    }

    #[test]
    fn un_cruce_no_parte_el_borde() {
        // Un rectángulo estrecho que cruza el lado de otro: ese lado sigue de una pieza (no se parte en el
        // cruce, que coincide un instante).
        let b = bordes(&doc(vec![
            vec![rect(10.0, 10.0, 90.0, 60.0)],
            vec![rect(40.0, 5.0, 41.0, 30.0)],
        ]));
        let lado_de_arriba = b
            .iter()
            .filter(|x| {
                x.tramos
                    .iter()
                    .any(|s| s.ini().1 == 10.0 && s.fin().1 == 10.0)
            })
            .count();
        assert_eq!(lado_de_arriba, 1, "{b:?}");
    }

    #[test]
    fn trozo_de_cubica() {
        let e = elipse_en_cubicas((0.0, 0.0), 10.0, 10.0, 0.0);
        let z = trozo(&e[1], 0.2, 0.7);
        for k in 0..=10 {
            let u = k as f64 / 10.0;
            let p = en(&z, u);
            assert!((dist(p, en(&e[1], 0.2 + 0.5 * u))) < 1e-9, "{u}");
        }
        assert_eq!(trozo(&e[1], 0.0, 1.0), e[1]);
    }
}
