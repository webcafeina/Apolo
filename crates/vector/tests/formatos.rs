//! Los cuatro formatos son el mismo dibujo: cada uno se rasteriza por su cuenta y se compara con el SVG
//! pintado por resvg.
//!
//! - **PDF** con `pdftoppm` (Poppler) y **EPS** con Ghostscript, si están instalados. Con
//!   `APOLO_FORMATOS_ESTRICTO=1` (en la CI), que falten es un fallo.
//! - **DXF** validado con ezdxf (`validar_dxf.py`, por `uvx`), que es lo más parecido a abrirlo en AutoCAD
//!   sin AutoCAD; también con `APOLO_FORMATOS_ESTRICTO=1`.
//! - **DXF** pintado con un lector de esta prueba que reconstruye cada entidad con su significado del estándar
//!   (arcos por su abombamiento, splines con de Boor, elipses por su eje y su razón): vigila el signo de los
//!   arcos, los nudos, la elipse y la vuelta del eje Y, que no se ven leyendo el texto.

use apolo_vector::{
    Capa, Documento, Estilo, Forma, FormatoVector, NivelDxf, OpcionesExportar, OpcionesVector,
    Relleno, Rgba, exportar, vectorizar, vista,
};
use resvg::tiny_skia;
use std::path::{Path, PathBuf};
use std::process::Command;

fn estricto() -> bool {
    std::env::var("APOLO_FORMATOS_ESTRICTO").is_ok_and(|v| v == "1")
}

fn hay(programa: &str) -> bool {
    let ok = Command::new(programa).arg("-v").output().is_ok()
        || Command::new(programa).arg("--version").output().is_ok();
    if !ok {
        assert!(!estricto(), "falta {programa} (APOLO_FORMATOS_ESTRICTO=1)");
        eprintln!("sin {programa}: se salta esta comprobación");
    }
    ok
}

fn dir(nombre: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("apolo-formatos-{}-{nombre}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

/// Una imagen de prueba con círculo, cuadrado girado, estrella, elipse y media luna (curvas libres),
/// suavizada con 4 × 4 submuestras.
fn imagen() -> Rgba {
    let (w, h) = (140u32, 140u32);
    let dentro_poli = |p: &[(f64, f64)], x: f64, y: f64| {
        let mut c = false;
        let mut j = p.len() - 1;
        for i in 0..p.len() {
            let (a, b) = (p[i], p[j]);
            if (a.1 > y) != (b.1 > y) && x < (b.0 - a.0) * (y - a.1) / (b.1 - a.1) + a.0 {
                c = !c;
            }
            j = i;
        }
        c
    };
    let giro = |c: (f64, f64), pts: &[(f64, f64)], a: f64| -> Vec<(f64, f64)> {
        let (s, co) = a.sin_cos();
        pts.iter()
            .map(|p| (c.0 + p.0 * co - p.1 * s, c.1 + p.0 * s + p.1 * co))
            .collect()
    };
    let cuadrado = giro(
        (100.0, 42.0),
        &[(-20.0, -20.0), (20.0, -20.0), (20.0, 20.0), (-20.0, 20.0)],
        0.35,
    );
    let estrella: Vec<(f64, f64)> = (0..10)
        .map(|i| {
            let a = i as f64 * std::f64::consts::PI / 5.0 - std::f64::consts::FRAC_PI_2;
            let r = if i % 2 == 0 { 28.0 } else { 12.0 };
            (68.0 + r * a.cos(), 100.0 + r * a.sin())
        })
        .collect();
    let color = |x: f64, y: f64| -> [u8; 3] {
        let d = |cx: f64, cy: f64| ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
        let (ex, ey) = {
            let (s, c) = (-0.5f64).sin_cos();
            let (u, v) = (x - 28.0, y - 108.0);
            (u * c + v * s, -u * s + v * c)
        };
        if d(40.0, 40.0) < 24.0 {
            [220, 40, 40]
        } else if dentro_poli(&cuadrado, x, y) {
            [30, 70, 200]
        } else if dentro_poli(&estrella, x, y) {
            [30, 160, 60]
        } else if (ex / 18.0).powi(2) + (ey / 8.0).powi(2) < 1.0 {
            [20, 20, 20]
        } else if d(112.0, 108.0) < 22.0 && d(121.0, 100.0) > 17.0 {
            [240, 150, 20]
        } else {
            [245, 242, 230]
        }
    };
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let mut s = [0u32; 3];
            for sy in 0..4 {
                for sx in 0..4 {
                    let c = color(
                        x as f64 + (sx as f64 + 0.5) / 4.0,
                        y as f64 + (sy as f64 + 0.5) / 4.0,
                    );
                    for k in 0..3 {
                        s[k] += c[k] as u32;
                    }
                }
            }
            px.extend([(s[0] / 16) as u8, (s[1] / 16) as u8, (s[2] / 16) as u8, 255]);
        }
    }
    Rgba::nueva(w, h, px)
}

/// Un documento con degradados (lineal de tres paradas y radial).
fn con_degradados() -> Documento {
    Documento {
        ancho: 100,
        alto: 80,
        fondo: Some(Relleno::Lineal {
            p1: (0.0, 0.0),
            p2: (100.0, 0.0),
            paradas: vec![(0.0, [255, 0, 0]), (0.5, [0, 200, 0]), (1.0, [0, 0, 255])],
        }),
        capas: vec![Capa {
            relleno: Relleno::Radial {
                c: (50.0, 40.0),
                r: 30.0,
                paradas: vec![(0.0, [255, 255, 255]), (1.0, [40, 40, 40])],
            },
            formas: vec![Forma::Circulo {
                c: (50.0, 40.0),
                r: 28.0,
            }],
        }],
    }
}

/// Diferencia media (0–255, por canal) y fracción de píxeles muy distintos (más de 64 en algún canal).
fn diferencia(a: &[u8], b: &[u8]) -> (f64, f64) {
    assert_eq!(a.len(), b.len());
    let mut suma = 0u64;
    let mut malos = 0usize;
    for (pa, pb) in a.chunks(4).zip(b.chunks(4)) {
        let mut peor = 0;
        for k in 0..3 {
            let d = (pa[k] as i32 - pb[k] as i32).unsigned_abs();
            suma += d as u64;
            peor = peor.max(d);
        }
        if peor > 64 {
            malos += 1;
        }
    }
    let n = a.len() / 4;
    (suma as f64 / (n * 3) as f64, malos as f64 / n as f64)
}

fn comprobar(nombre: &str, referencia: &[u8], otro: &[u8]) {
    comprobar_con(nombre, referencia, otro, 2.5, 0.01);
}

fn comprobar_con(nombre: &str, referencia: &[u8], otro: &[u8], max_media: f64, max_malos: f64) {
    let (media, malos) = diferencia(referencia, otro);
    eprintln!(
        "{nombre}: diferencia media {media:.2}, píxeles muy distintos {:.2} %",
        malos * 100.0
    );
    assert!(
        media < max_media && malos < max_malos,
        "{nombre}: media {media:.2}, muy distintos {:.2} %",
        malos * 100.0
    );
}

fn png(ruta: &Path) -> Vec<u8> {
    let p = tiny_skia::Pixmap::load_png(ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()));
    p.data().to_vec()
}

fn referencia(doc: &Documento, o: &OpcionesExportar) -> Vec<u8> {
    let svg = exportar(doc, FormatoVector::Svg, o);
    vista::rasterizar_svg(&svg, doc.ancho, doc.alto).unwrap()
}

fn pdf_a_png(doc: &Documento, o: &OpcionesExportar, d: &Path) -> Vec<u8> {
    std::fs::write(d.join("a.pdf"), exportar(doc, FormatoVector::Pdf, o)).unwrap();
    let st = Command::new("pdftoppm")
        .args(["-r", "72", "-singlefile", "-png"])
        .arg(d.join("a.pdf"))
        .arg(d.join("pdf"))
        .status()
        .unwrap();
    assert!(st.success(), "pdftoppm");
    png(&d.join("pdf.png"))
}

/// Ghostscript: el EPS (recortado a su caja) o el PDF, a 72 ppp.
fn gs_a_png(entrada: &Path, salida: &Path) -> Vec<u8> {
    let st = Command::new("gs")
        .args([
            "-q",
            "-dSAFER",
            "-dBATCH",
            "-dNOPAUSE",
            "-sDEVICE=png16m",
            "-r72",
        ])
        .args(["-dGraphicsAlphaBits=4", "-dTextAlphaBits=4"])
        .args(
            entrada
                .extension()
                .is_some_and(|e| e == "eps")
                .then_some("-dEPSCrop"),
        )
        .arg(format!("-sOutputFile={}", salida.display()))
        .arg(entrada)
        .status()
        .unwrap();
    assert!(st.success(), "gs {}", entrada.display());
    png(salida)
}

fn vectorizado() -> Documento {
    vectorizar(&imagen(), &OpcionesVector::default()).unwrap()
}

#[test]
fn pdf_igual_que_svg() {
    if !hay("pdftoppm") {
        return;
    }
    let d = dir("pdf");
    for (nombre, doc) in [
        ("vectorizado", vectorizado()),
        ("degradados", con_degradados()),
    ] {
        for estilo in [Estilo::Rellenar, Estilo::TrazarFormas, Estilo::TrazarBordes] {
            let o = OpcionesExportar {
                estilo,
                grosor: 1.5,
                ..Default::default()
            };
            comprobar(
                &format!("PDF {nombre} {estilo:?}"),
                &referencia(&doc, &o),
                &pdf_a_png(&doc, &o, &d),
            );
        }
    }
}

/// El EPS sale del mismo contenido que el PDF: pintados los dos por Ghostscript tienen que coincidir casi al
/// píxel. Frente al SVG de resvg, con más margen: el suavizado de Ghostscript es otro (sobre todo en trazos
/// finos; en la CI, «trazar bordes» da 3,1 de media).
#[test]
fn eps_igual_que_pdf_y_svg() {
    if !hay("gs") {
        return;
    }
    let d = dir("eps");
    for (nombre, doc) in [
        ("vectorizado", vectorizado()),
        ("degradados", con_degradados()),
    ] {
        for estilo in [Estilo::Rellenar, Estilo::TrazarFormas, Estilo::TrazarBordes] {
            let o = OpcionesExportar {
                estilo,
                grosor: 1.5,
                ..Default::default()
            };
            std::fs::write(d.join("a.eps"), exportar(&doc, FormatoVector::Eps, &o)).unwrap();
            std::fs::write(d.join("a.pdf"), exportar(&doc, FormatoVector::Pdf, &o)).unwrap();
            let eps = gs_a_png(&d.join("a.eps"), &d.join("eps.png"));
            let pdf = gs_a_png(&d.join("a.pdf"), &d.join("pdf-gs.png"));
            comprobar_con(
                &format!("EPS frente a PDF, {nombre} {estilo:?}"),
                &pdf,
                &eps,
                0.3,
                0.001,
            );
            comprobar_con(
                &format!("EPS frente a SVG, {nombre} {estilo:?}"),
                &referencia(&doc, &o),
                &eps,
                4.5,
                0.015,
            );
        }
    }
}

/// Una entidad leída: su capa, sus puntos y si es cerrada.
type Leida = (String, Vec<(f64, f64)>, bool);

/// Las entidades de la sección ENTITIES como (capa, polígono con el eje Y hacia arriba).
fn leer_dxf(txt: &str) -> Vec<Leida> {
    let lineas: Vec<&str> = txt.lines().collect();
    let pares: Vec<(i32, &str)> = lineas
        .chunks(2)
        .map(|c| (c[0].trim().parse().unwrap(), c[1].trim()))
        .collect();
    let ini = pares.iter().position(|p| *p == (2, "ENTITIES")).unwrap() + 1;
    let fin = ini
        + pares[ini..]
            .iter()
            .position(|p| *p == (0, "ENDSEC"))
            .unwrap();
    let mut ents = Vec::new();
    let mut i = ini;
    while i < fin {
        let tipo = pares[i].1;
        let mut j = i + 1;
        while j < fin && pares[j].0 != 0 {
            j += 1;
        }
        let campos = &pares[i + 1..j];
        let uno = |c: i32| {
            campos
                .iter()
                .find(|p| p.0 == c)
                .map(|p| p.1.parse::<f64>().unwrap())
                .unwrap_or(0.0)
        };
        let todos = |c: i32| {
            campos
                .iter()
                .filter(|p| p.0 == c)
                .map(|p| p.1.parse::<f64>().unwrap())
                .collect::<Vec<_>>()
        };
        let capa = campos.iter().find(|p| p.0 == 8).unwrap().1.to_string();
        let muestras = 720;
        let (pts, cerrado) = match tipo {
            "CIRCLE" => {
                let (c, r) = ((uno(10), uno(20)), uno(40));
                (
                    (0..muestras)
                        .map(|k| {
                            let a = k as f64 / muestras as f64 * std::f64::consts::TAU;
                            (c.0 + r * a.cos(), c.1 + r * a.sin())
                        })
                        .collect(),
                    true,
                )
            }
            "ELLIPSE" => {
                let (c, e, razon) = ((uno(10), uno(20)), (uno(11), uno(21)), uno(40));
                let m = (-e.1 * razon, e.0 * razon);
                (
                    (0..muestras)
                        .map(|k| {
                            let a = k as f64 / muestras as f64 * std::f64::consts::TAU;
                            (
                                c.0 + e.0 * a.cos() + m.0 * a.sin(),
                                c.1 + e.1 * a.cos() + m.1 * a.sin(),
                            )
                        })
                        .collect(),
                    true,
                )
            }
            "LWPOLYLINE" => {
                // Cada vértice: 10, 20 y, si lo hay, su 42.
                let mut v: Vec<((f64, f64), f64)> = Vec::new();
                for p in campos {
                    match p.0 {
                        10 => v.push(((p.1.parse().unwrap(), 0.0), 0.0)),
                        20 => v.last_mut().unwrap().0.1 = p.1.parse().unwrap(),
                        42 => v.last_mut().unwrap().1 = p.1.parse().unwrap(),
                        _ => {}
                    }
                }
                let cerrada = uno(70) as i32 & 1 == 1;
                let mut pts = Vec::new();
                let tramos = if cerrada { v.len() } else { v.len() - 1 };
                for k in 0..tramos {
                    let (a, b, bu) = (v[k].0, v[(k + 1) % v.len()].0, v[k].1);
                    pts.push(a);
                    if bu != 0.0 {
                        // Arco: ángulo 4·atan(b), centro a la distancia que da la cuerda.
                        let th = 4.0 * bu.atan();
                        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
                        let cuerda = (dx * dx + dy * dy).sqrt();
                        let r = cuerda / (2.0 * (th / 2.0).sin());
                        let hh = r * (th / 2.0).cos();
                        let (mx, my) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
                        let c = (mx - hh * dy / cuerda, my + hh * dx / cuerda);
                        let a0 = (a.1 - c.1).atan2(a.0 - c.0);
                        for s in 1..32 {
                            let ang = a0 + th * s as f64 / 32.0;
                            pts.push((c.0 + r.abs() * ang.cos(), c.1 + r.abs() * ang.sin()));
                        }
                    }
                }
                if !cerrada {
                    pts.push(v[v.len() - 1].0);
                }
                (pts, cerrada)
            }
            "SPLINE" => {
                let grado = uno(71) as usize;
                let nudos = todos(40);
                let (xs, ys) = (todos(10), todos(20));
                let ctrl: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();
                let (t0, t1) = (nudos[grado], nudos[nudos.len() - grado - 1]);
                let de_boor = |t: f64| -> (f64, f64) {
                    let mut k = grado;
                    while k + 1 < nudos.len() - grado - 1 && nudos[k + 1] <= t {
                        k += 1;
                    }
                    let mut dd: Vec<(f64, f64)> =
                        (0..=grado).map(|j| ctrl[j + k - grado]).collect();
                    for r in 1..=grado {
                        for j in (r..=grado).rev() {
                            let i = j + k - grado;
                            let den = nudos[i + grado + 1 - r] - nudos[i];
                            let al = if den == 0.0 {
                                0.0
                            } else {
                                (t - nudos[i]) / den
                            };
                            dd[j] = (
                                (1.0 - al) * dd[j - 1].0 + al * dd[j].0,
                                (1.0 - al) * dd[j - 1].1 + al * dd[j].1,
                            );
                        }
                    }
                    dd[grado]
                };
                let n = 64 * (t1 - t0).max(1.0) as usize;
                let pts: Vec<(f64, f64)> = (0..=n)
                    .map(|s| de_boor(t0 + (t1 - t0) * s as f64 / n as f64))
                    .collect();
                let cerr = {
                    let (a, b) = (pts[0], pts[pts.len() - 1]);
                    (a.0 - b.0).abs() < 1e-6 && (a.1 - b.1).abs() < 1e-6
                };
                (pts, cerr)
            }
            otro => panic!("entidad inesperada {otro}"),
        };
        ents.push((capa, pts, cerrado));
        i = j;
    }
    ents
}

/// El color de cada capa del DXF, de su código 420.
fn colores_dxf(txt: &str) -> std::collections::HashMap<String, [u8; 3]> {
    let lineas: Vec<&str> = txt.lines().map(str::trim).collect();
    let mut m = std::collections::HashMap::new();
    let mut i = 0;
    while i + 1 < lineas.len() {
        if lineas[i] == "0" && lineas[i + 1] == "LAYER" {
            let mut nombre = String::new();
            let mut j = i + 2;
            while j + 1 < lineas.len() && lineas[j] != "0" {
                match lineas[j] {
                    "2" => nombre = lineas[j + 1].to_string(),
                    "420" => {
                        let v: u32 = lineas[j + 1].parse().unwrap();
                        m.insert(nombre.clone(), [(v >> 16) as u8, (v >> 8) as u8, v as u8]);
                    }
                    _ => {}
                }
                j += 2;
            }
            i = j;
        } else {
            i += 2;
        }
    }
    m
}

/// El DXF pintado: el fondo del documento y, encima, cada capa rellena par-impar (o trazada).
fn dxf_a_png(doc: &Documento, o: &OpcionesExportar) -> Vec<u8> {
    let txt = String::from_utf8(exportar(doc, FormatoVector::Dxf, o)).unwrap();
    let colores = colores_dxf(&txt);
    let ents = leer_dxf(&txt);
    let mut lienzo = tiny_skia::Pixmap::new(doc.ancho, doc.alto).unwrap();
    lienzo.fill(tiny_skia::Color::WHITE);
    let pintura = |c: [u8; 3]| {
        let mut p = tiny_skia::Paint::default();
        p.set_color_rgba8(c[0], c[1], c[2], 255);
        p.anti_alias = true;
        p
    };
    let alto = doc.alto as f32;
    if o.estilo == Estilo::Rellenar
        && let Some(f) = &doc.fondo
    {
        lienzo.fill_rect(
            tiny_skia::Rect::from_xywh(0.0, 0.0, doc.ancho as f32, alto).unwrap(),
            &pintura(f.color_medio()),
            tiny_skia::Transform::identity(),
            None,
        );
    }
    // Las entidades seguidas de una misma capa forman un trazado.
    let mut k = 0;
    while k < ents.len() {
        let capa = &ents[k].0;
        let mut pb = tiny_skia::PathBuilder::new();
        while k < ents.len() && &ents[k].0 == capa {
            let (_, pts, cerrado) = &ents[k];
            pb.move_to(pts[0].0 as f32, alto - pts[0].1 as f32);
            for p in &pts[1..] {
                pb.line_to(p.0 as f32, alto - p.1 as f32);
            }
            if *cerrado {
                pb.close();
            }
            k += 1;
        }
        let camino = pb.finish().unwrap();
        let c = colores[capa];
        if o.estilo == Estilo::Rellenar {
            lienzo.fill_path(
                &camino,
                &pintura(c),
                tiny_skia::FillRule::EvenOdd,
                tiny_skia::Transform::identity(),
                None,
            );
        } else {
            let trazo = tiny_skia::Stroke {
                width: o.grosor as f32,
                line_join: tiny_skia::LineJoin::Round,
                line_cap: tiny_skia::LineCap::Round,
                ..Default::default()
            };
            lienzo.stroke_path(
                &camino,
                &pintura(c),
                &trazo,
                tiny_skia::Transform::identity(),
                None,
            );
        }
    }
    lienzo.data().to_vec()
}

#[test]
fn dxf_igual_que_svg() {
    let doc = vectorizado();
    for nivel in [NivelDxf::Rectas, NivelDxf::Arcos, NivelDxf::Splines] {
        for estilo in [Estilo::Rellenar, Estilo::TrazarFormas, Estilo::TrazarBordes] {
            let o = OpcionesExportar {
                estilo,
                grosor: 1.5,
                nivel_dxf: nivel,
                ..Default::default()
            };
            comprobar(
                &format!("DXF {nivel:?} {estilo:?}"),
                &referencia(&doc, &o),
                &dxf_a_png(&doc, &o),
            );
        }
    }
}

#[test]
fn dxf_identificadores() {
    // Cada identificador (5, y 105 en DIMSTYLE) es único, y cada dueño (330) existe.
    let o = OpcionesExportar {
        nivel_dxf: NivelDxf::Splines,
        ..Default::default()
    };
    let txt = String::from_utf8(exportar(&vectorizado(), FormatoVector::Dxf, &o)).unwrap();
    let l: Vec<&str> = txt.lines().map(str::trim).collect();
    assert_eq!(l.len() % 2, 0);
    let pares: Vec<(&str, &str)> = l.chunks(2).map(|c| (c[0], c[1])).collect();
    let semilla_en = pares.iter().position(|p| *p == ("9", "$HANDSEED")).unwrap() + 1;
    let mut ids = std::collections::HashSet::new();
    for (_, p) in pares
        .iter()
        .enumerate()
        .filter(|(i, p)| *i != semilla_en && (p.0 == "5" || p.0 == "105"))
    {
        assert!(ids.insert(p.1), "identificador repetido {}", p.1);
    }
    for p in pares
        .iter()
        .filter(|p| p.0 == "330" || p.0 == "340" || p.0 == "350")
    {
        assert!(p.1 == "0" || ids.contains(p.1), "{} {} no existe", p.0, p.1);
    }
    // $HANDSEED, mayor que todos.
    let semilla = u32::from_str_radix(pares[semilla_en].1, 16).unwrap();
    assert!(
        ids.iter()
            .all(|h| u32::from_str_radix(h, 16).unwrap() < semilla)
    );
    // Las entidades usadas.
    for e in ["CIRCLE", "ELLIPSE", "SPLINE", "LWPOLYLINE"] {
        assert!(pares.contains(&("0", e)), "falta {e}");
    }
}

#[test]
fn pdf_bien_formado() {
    // Los desplazamientos de la tabla xref apuntan a cada «n 0 obj» (en bytes: el PDF lleva binario).
    let pdf = exportar(
        &con_degradados(),
        FormatoVector::Pdf,
        &OpcionesExportar::default(),
    );
    let busca = |aguja: &[u8]| pdf.windows(aguja.len()).rposition(|w| w == aguja).unwrap();
    let x = busca(b"startxref\n") + 10;
    let fin = x + pdf[x..].iter().position(|b| *b == b'\n').unwrap();
    let xref: usize = std::str::from_utf8(&pdf[x..fin]).unwrap().parse().unwrap();
    let cola = std::str::from_utf8(&pdf[xref..]).unwrap();
    assert!(cola.starts_with("xref\n"));
    let entradas: Vec<&str> = cola
        .lines()
        .skip(3)
        .take_while(|l| l.trim_end().ends_with(" n"))
        .collect();
    assert_eq!(entradas.len(), 5);
    for (i, e) in entradas.iter().enumerate() {
        // 20 bytes por entrada, con el salto de línea.
        assert_eq!(e.len(), 19, "entrada {i}");
        let d: usize = e[..10].parse().unwrap();
        assert!(
            pdf[d..].starts_with(format!("{} 0 obj", i + 1).as_bytes()),
            "objeto {}",
            i + 1
        );
    }
}

#[test]
fn dxf_valido_para_ezdxf() {
    if !hay("uvx") {
        return;
    }
    let d = dir("ezdxf");
    let mut rutas = Vec::new();
    for (nombre, doc) in [
        ("vectorizado", vectorizado()),
        ("degradados", con_degradados()),
    ] {
        for nivel in [NivelDxf::Rectas, NivelDxf::Arcos, NivelDxf::Splines] {
            for estilo in [Estilo::Rellenar, Estilo::TrazarBordes] {
                let o = OpcionesExportar {
                    estilo,
                    nivel_dxf: nivel,
                    ..Default::default()
                };
                let r = d.join(format!("{nombre}-{nivel:?}-{estilo:?}.dxf"));
                std::fs::write(&r, exportar(&doc, FormatoVector::Dxf, &o)).unwrap();
                rutas.push(r);
            }
        }
    }
    let guion = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/validar_dxf.py");
    let salida = Command::new("uvx")
        .args(["--with", "ezdxf==1.4.2", "python", "-I"])
        .arg(guion)
        .args(&rutas)
        .output()
        .unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout);
    eprintln!(
        "{texto}{}",
        String::from_utf8_lossy(&salida.stderr)
            .lines()
            .filter(|l| !l.contains("Download") && !l.contains("Installed"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(salida.status.success(), "ezdxf: {texto}");
}
