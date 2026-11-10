//! El contenido común de PDF y EPS: los operadores de dibujo de PDF (`m`, `l`, `c`, `h`, `f*`, `rg`…), que
//! el EPS define con esos mismos nombres en su prólogo. Así los dos son el mismo dibujo.
//!
//! Se dibuja en coordenadas de la imagen (Y hacia abajo): el que llama pone antes la transformación que da
//! la vuelta a la página (`1 0 0 -1 0 alto cm` en PDF, `0 alto translate 1 -1 scale` en EPS).

use crate::exportar::{Estilo, OpcionesExportar, num};
use crate::modelo::{Documento, Relleno, Tramo};
use crate::trazado;
use std::fmt::Write;

fn rgb(c: [u8; 3]) -> String {
    format!(
        "{} {} {}",
        num(c[0] as f64 / 255.0),
        num(c[1] as f64 / 255.0),
        num(c[2] as f64 / 255.0)
    )
}

fn camino(t: &[Tramo], cerrar: bool, out: &mut String) {
    let Some(p) = t.first().map(|s| s.ini()) else {
        return;
    };
    let _ = writeln!(out, "{} {} m", num(p.0), num(p.1));
    for s in t {
        match *s {
            Tramo::Linea(_, b) => {
                let _ = writeln!(out, "{} {} l", num(b.0), num(b.1));
            }
            Tramo::Cubica(_, b, c, d) => {
                let _ = writeln!(
                    out,
                    "{} {} {} {} {} {} c",
                    num(b.0),
                    num(b.1),
                    num(c.0),
                    num(c.1),
                    num(d.0),
                    num(d.1)
                );
            }
        }
    }
    if cerrar {
        out.push_str("h\n");
    }
}

/// Las paradas, de 0 a 1 y estrictamente crecientes, como piden los sombreados de PDF y PostScript.
fn paradas_limpias(p: &[(f64, [u8; 3])]) -> Vec<(f64, [u8; 3])> {
    let mut v: Vec<(f64, [u8; 3])> = p.iter().map(|(o, c)| (o.clamp(0.0, 1.0), *c)).collect();
    if v.is_empty() {
        return vec![(0.0, [0; 3]), (1.0, [0; 3])];
    }
    if v[0].0 > 0.0 {
        v.insert(0, (0.0, v[0].1));
    }
    if v[v.len() - 1].0 < 1.0 {
        let c = v[v.len() - 1].1;
        v.push((1.0, c));
    }
    // Dos paradas en el mismo sitio: la segunda, una milésima después (el salto se nota igual).
    for i in 1..v.len() {
        if v[i].0 <= v[i - 1].0 {
            v[i].0 = (v[i - 1].0 + 1e-3).min(1.0);
        }
    }
    v.dedup_by(|b, a| b.0 <= a.0);
    if v.len() < 2 {
        let c = v[0].1;
        v = vec![(0.0, c), (1.0, c)];
    }
    v
}

/// La función del degradado: una interpolación lineal por cada par de paradas (tipo 2), cosidas (tipo 3).
fn funcion(p: &[(f64, [u8; 3])]) -> String {
    let c = |c: [u8; 3]| format!("[{}]", rgb(c));
    let tramo = |a: [u8; 3], b: [u8; 3]| {
        format!(
            "<< /FunctionType 2 /Domain [0 1] /C0 {} /C1 {} /N 1 >>",
            c(a),
            c(b)
        )
    };
    let p = paradas_limpias(p);
    if p.len() == 2 {
        return tramo(p[0].1, p[1].1);
    }
    let funciones: Vec<String> = p.windows(2).map(|w| tramo(w[0].1, w[1].1)).collect();
    let limites: Vec<String> = p[1..p.len() - 1].iter().map(|x| num_fino(x.0)).collect();
    let codif: Vec<&str> = (0..funciones.len()).map(|_| "0 1").collect();
    format!(
        "<< /FunctionType 3 /Domain [0 1] /Functions [{}] /Bounds [{}] /Encode [{}] >>",
        funciones.join(" "),
        limites.join(" "),
        codif.join(" ")
    )
}

/// Con cuatro decimales: los límites de las paradas tienen que seguir siendo crecientes.
fn num_fino(x: f64) -> String {
    let r = (x * 10_000.0).round() / 10_000.0;
    format!("{r}")
}

/// El diccionario de sombreado del degradado (vale igual en PDF y en PostScript de nivel 3).
pub fn sombreado(r: &Relleno) -> Option<String> {
    Some(match r {
        Relleno::Color(_) => return None,
        Relleno::Lineal { p1, p2, paradas } => format!(
            "<< /ShadingType 2 /ColorSpace /DeviceRGB /Coords [{} {} {} {}] /Function {} /Extend [true true] >>",
            num(p1.0),
            num(p1.1),
            num(p2.0),
            num(p2.1),
            funcion(paradas)
        ),
        Relleno::Radial { c, r, paradas } => format!(
            "<< /ShadingType 3 /ColorSpace /DeviceRGB /Coords [{} {} 0 {} {} {}] /Function {} /Extend [true true] >>",
            num(c.0),
            num(c.1),
            num(c.0),
            num(c.1),
            num(*r),
            funcion(paradas)
        ),
    })
}

/// El contenido. `pinta_sombreado(i, diccionario)` devuelve la orden que pinta el degradado número `i`
/// dentro del recorte (en PDF, `/Sh0 sh` con el diccionario en los recursos; en PostScript, el diccionario
/// seguido de `shfill`).
pub fn contenido(
    doc: &Documento,
    o: &OpcionesExportar,
    pinta_sombreado: &mut dyn FnMut(usize, String) -> String,
) -> String {
    let mut out = String::new();
    let mut n = 0;
    let mut relleno = |r: &Relleno, camino_txt: &str, out: &mut String| match sombreado(r) {
        None => {
            let c = match r {
                Relleno::Color(c) => *c,
                _ => r.color_medio(),
            };
            let _ = write!(out, "{} rg\n{camino_txt}f*\n", rgb(c));
        }
        Some(dic) => {
            let orden = pinta_sombreado(n, dic);
            n += 1;
            let _ = write!(out, "q\n{camino_txt}W* n\n{orden}\nQ\n");
        }
    };
    match o.estilo {
        Estilo::Rellenar => {
            if let Some(f) = &doc.fondo {
                let rect = format!("0 0 {} {} re\n", doc.ancho, doc.alto);
                relleno(f, &rect, &mut out);
            }
            for capa in &doc.capas {
                let mut c = String::new();
                for f in &capa.formas {
                    camino(&trazado::tramos(f), true, &mut c);
                }
                relleno(&capa.relleno, &c, &mut out);
            }
        }
        Estilo::TrazarFormas | Estilo::TrazarBordes => {
            let _ = writeln!(out, "{} w 1 j 1 J", num(o.grosor));
            let mut trazos: Vec<([u8; 3], String)> = Vec::new();
            if o.estilo == Estilo::TrazarFormas {
                for capa in &doc.capas {
                    let mut c = String::new();
                    for f in &capa.formas {
                        camino(&trazado::tramos(f), true, &mut c);
                    }
                    trazos.push((trazado::color_de(&capa.relleno), c));
                }
            } else {
                for b in trazado::bordes(doc) {
                    let mut c = String::new();
                    camino(&b.tramos, b.cerrado, &mut c);
                    trazos.push((b.color, c));
                }
            }
            for (color, c) in trazos {
                let _ = write!(out, "{} RG\n{c}S\n", rgb(o.color_trazo.unwrap_or(color)));
            }
        }
    }
    out
}
