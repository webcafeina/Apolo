//! SVG 1.1 a partir del [`Documento`].
//!
//! Los círculos y las elipses se escriben con cuatro cuartos de arco: dos medios arcos entre puntos
//! opuestos son ambiguos en SVG y el visor puede deformarlos (docs/trampas.md). Cada capa es un `path`
//! con `fill-rule="evenodd"`: una figura dentro de otra es un hueco.

use crate::modelo::{Documento, Forma, Relleno, Tramo};
use std::fmt::Write;

fn r2(x: f64) -> f64 {
    let r = (x * 100.0).round() / 100.0;
    if r == 0.0 { 0.0 } else { r }
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// El atributo `d` de una forma.
pub fn d(f: &Forma, out: &mut String) {
    match f {
        Forma::Circulo { c, r } => elipse(*c, *r, *r, 0.0, out),
        Forma::Elipse { c, rx, ry, ang } => elipse(*c, *rx, *ry, *ang, out),
        Forma::Tramos(t) => {
            let Some(p0) = t.first().map(|s| s.ini()) else {
                return;
            };
            let _ = write!(out, "M{} {}", r2(p0.0), r2(p0.1));
            for s in t {
                match *s {
                    Tramo::Linea(_, b) => {
                        let _ = write!(out, "L{} {}", r2(b.0), r2(b.1));
                    }
                    Tramo::Cubica(_, c1, c2, b) => {
                        let _ = write!(
                            out,
                            "C{} {} {} {} {} {}",
                            r2(c1.0),
                            r2(c1.1),
                            r2(c2.0),
                            r2(c2.1),
                            r2(b.0),
                            r2(b.1)
                        );
                    }
                }
            }
            out.push('Z');
        }
    }
}

fn elipse(c: (f64, f64), rx: f64, ry: f64, ang: f64, out: &mut String) {
    let (s, co) = ang.sin_cos();
    let pto = |u: f64, v: f64| (c.0 + u * co - v * s, c.1 + u * s + v * co);
    let pts = [
        pto(rx, 0.0),
        pto(0.0, ry),
        pto(-rx, 0.0),
        pto(0.0, -ry),
        pto(rx, 0.0),
    ];
    let _ = write!(out, "M{} {}", r2(pts[0].0), r2(pts[0].1));
    for q in &pts[1..] {
        let _ = write!(
            out,
            "A{} {} {} 0 1 {} {}",
            r2(rx),
            r2(ry),
            r2(ang.to_degrees()),
            r2(q.0),
            r2(q.1)
        );
    }
    out.push('Z');
}

fn degradado(id: &str, r: &Relleno, defs: &mut String) {
    let stops = |p: &[(f64, [u8; 3])]| -> String {
        p.iter()
            .map(|(o, c)| format!(r#"<stop offset="{}" stop-color="{}"/>"#, r2(*o), hex(*c)))
            .collect()
    };
    match r {
        Relleno::Lineal { p1, p2, paradas } => {
            let _ = write!(
                defs,
                r#"<linearGradient id="{id}" gradientUnits="userSpaceOnUse" x1="{}" y1="{}" x2="{}" y2="{}">{}</linearGradient>"#,
                r2(p1.0),
                r2(p1.1),
                r2(p2.0),
                r2(p2.1),
                stops(paradas)
            );
        }
        Relleno::Radial { c, r, paradas } => {
            let _ = write!(
                defs,
                r#"<radialGradient id="{id}" gradientUnits="userSpaceOnUse" cx="{}" cy="{}" r="{}">{}</radialGradient>"#,
                r2(c.0),
                r2(c.1),
                r2(*r),
                stops(paradas)
            );
        }
        Relleno::Color(_) => {}
    }
}

/// El relleno como valor de `fill`, añadiendo su degradado a `defs` si lo es.
fn relleno(r: &Relleno, n: &mut usize, defs: &mut String) -> String {
    match r {
        Relleno::Color(c) => hex(*c),
        _ => {
            let id = format!("d{n}");
            *n += 1;
            degradado(&id, r, defs);
            format!("url(#{id})")
        }
    }
}

/// El SVG del documento, con su tamaño en píxeles y su `viewBox`.
pub fn escribir(doc: &Documento) -> String {
    let (w, h) = (doc.ancho, doc.alto);
    let mut defs = String::new();
    let mut n = 0;
    let mut cuerpo = String::new();
    if let Some(f) = &doc.fondo {
        let fill = relleno(f, &mut n, &mut defs);
        let _ = write!(cuerpo, r#"<rect width="{w}" height="{h}" fill="{fill}"/>"#);
    }
    for c in &doc.capas {
        let fill = relleno(&c.relleno, &mut n, &mut defs);
        let mut dd = String::new();
        for f in &c.formas {
            d(f, &mut dd);
        }
        let _ = write!(
            cuerpo,
            r#"<path fill="{fill}" fill-rule="evenodd" d="{dd}"/>"#
        );
    }
    let defs = if defs.is_empty() {
        String::new()
    } else {
        format!("<defs>{defs}</defs>")
    };
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" version="1.1" width="{w}" height="{h}" viewBox="0 0 {w} {h}">{defs}{cuerpo}</svg>"#
    )
}
