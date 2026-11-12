//! Lo transparente no se dibuja: ni rectángulo de fondo ni capa encima de los agujeros.

use apolo_vector::{OpcionesVector, Rgba, TipoImagen, svg, vectorizar, vista};

/// Una imagen con fondo transparente, suavizada con 4 × 4 submuestras. `color(x, y)` da el color y el alfa
/// (0 o 255) en cada punto.
fn imagen(lado: u32, color: impl Fn(f64, f64) -> [u8; 4]) -> Rgba {
    let mut px = Vec::new();
    for y in 0..lado {
        for x in 0..lado {
            // Promedio premultiplicado, como un rasterizador.
            let mut s = [0u32; 4];
            for sy in 0..4 {
                for sx in 0..4 {
                    let c = color(
                        x as f64 + (sx as f64 + 0.5) / 4.0,
                        y as f64 + (sy as f64 + 0.5) / 4.0,
                    );
                    for k in 0..3 {
                        s[k] += c[k] as u32 * c[3] as u32 / 255;
                    }
                    s[3] += c[3] as u32;
                }
            }
            let a = s[3] / 16;
            // Sin premultiplicar; con alfa 0, negro (no se ve).
            let des = |v: u32| ((v / 16) * 255).checked_div(a).unwrap_or(0).min(255) as u8;
            px.extend([des(s[0]), des(s[1]), des(s[2]), a as u8]);
        }
    }
    Rgba::nueva(lado, lado, px)
}

fn distancia(x: f64, y: f64, cx: f64, cy: f64) -> f64 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt()
}

#[test]
fn logo_sobre_transparente_sin_fondo() {
    // Un cuadrado azul y un círculo rojo, sin nada detrás.
    let im = imagen(96, |x, y| {
        if distancia(x, y, 64.0, 60.0) < 20.0 {
            [220, 40, 40, 255]
        } else if (12.0..44.0).contains(&x) && (14.0..46.0).contains(&y) {
            [30, 70, 200, 255]
        } else {
            [0, 0, 0, 0]
        }
    });
    assert!(im.con_transparencia());
    let doc = vectorizar(&im, &OpcionesVector::default()).unwrap();
    assert!(doc.fondo.is_none(), "sin fondo: {:?}", doc.fondo);
    assert_eq!(doc.capas.len(), 2, "dos capas, el azul y el rojo");
    let s = svg::escribir(&doc);
    assert!(!s.contains("<rect"), "{s}");
    assert!(!s.contains("#ffffff"), "ningún blanco que no existía: {s}");
    // Sobre blanco, la vista previa se parece al original aplanado sobre blanco.
    let v = vista::rasterizar(&doc).unwrap();
    let error = im
        .px
        .iter()
        .zip(&v)
        .map(|(a, b)| (*a as i32 - *b as i32).unsigned_abs())
        .sum::<u32>() as f64
        / im.px.len() as f64;
    assert!(error < 2.0, "error medio {error}");
}

#[test]
fn el_agujero_de_un_aro_queda_transparente() {
    // Un aro verde: el agujero es transparente, no blanco.
    let im = imagen(80, |x, y| {
        let d = distancia(x, y, 40.0, 40.0);
        if (15.0..30.0).contains(&d) {
            [40, 160, 60, 255]
        } else {
            [0, 0, 0, 0]
        }
    });
    let doc = vectorizar(&im, &OpcionesVector::default()).unwrap();
    assert!(doc.fondo.is_none());
    // Pintado sobre transparente, el centro del agujero y una esquina siguen transparentes.
    let s = svg::escribir(&doc);
    let arbol = resvg::usvg::Tree::from_data(s.as_bytes(), &Default::default()).unwrap();
    let mut lienzo = resvg::tiny_skia::Pixmap::new(80, 80).unwrap();
    resvg::render(
        &arbol,
        resvg::tiny_skia::Transform::identity(),
        &mut lienzo.as_mut(),
    );
    let alfa = |x: usize, y: usize| lienzo.data()[(y * 80 + x) * 4 + 3];
    assert_eq!(alfa(40, 40), 0, "el agujero");
    assert_eq!(alfa(2, 2), 0, "la esquina");
    assert_eq!(alfa(40, 18), 255, "el aro");
}

#[test]
fn blanco_de_verdad_sobre_transparente() {
    // Un círculo blanco con borde negro sobre transparente: el blanco es un color, no el fondo.
    let im = imagen(64, |x, y| {
        let d = distancia(x, y, 32.0, 32.0);
        if d < 20.0 {
            [255, 255, 255, 255]
        } else if d < 26.0 {
            [10, 10, 10, 255]
        } else {
            [0, 0, 0, 0]
        }
    });
    let doc = vectorizar(&im, &OpcionesVector::default()).unwrap();
    assert!(doc.fondo.is_none());
    let s = svg::escribir(&doc);
    assert!(s.contains("#ffffff"), "el blanco del círculo: {s}");
}

#[test]
fn pixel_art_con_transparencia() {
    // Un corazón de pixel art, 16 × 16, con el fondo transparente.
    let mut px = Vec::new();
    for y in 0..16 {
        for x in 0..16 {
            let dentro = (4..12).contains(&x) && (4..12).contains(&y) && !(x == 4 && y == 4);
            px.extend(if dentro {
                [200, 20, 60, 255]
            } else {
                [0, 0, 0, 0]
            });
        }
    }
    let im = Rgba::nueva(16, 16, px);
    let o = OpcionesVector {
        tipo: TipoImagen::PixelArt,
        ..Default::default()
    };
    let doc = vectorizar(&im, &o).unwrap();
    assert!(doc.fondo.is_none());
    assert_eq!(doc.capas.len(), 1);
}

#[test]
fn opaca_como_siempre() {
    // Sin transparencia, nada cambia: hay fondo.
    let im = imagen(48, |x, y| {
        if distancia(x, y, 24.0, 24.0) < 12.0 {
            [0, 0, 0, 255]
        } else {
            [255, 255, 255, 255]
        }
    });
    assert!(!im.con_transparencia());
    let doc = vectorizar(&im, &OpcionesVector::default()).unwrap();
    assert!(doc.fondo.is_some());
}

/// El error cuadrático del dibujo (sobre blanco) frente a la imagen aplanada sobre blanco.
fn error(im: &Rgba) -> f64 {
    let doc = vectorizar(im, &OpcionesVector::default()).unwrap();
    let r = vista::rasterizar(&doc).unwrap();
    im.px
        .iter()
        .zip(&r)
        .map(|(a, b)| (*a as f64 - *b as f64).powi(2))
        .sum::<f64>()
}

#[test]
fn el_borde_sobre_transparente_sale_como_sobre_blanco() {
    // El sol de la web, con las esquinas transparentes, frente al mismo aplanado sobre blanco. Daba un 35 % más
    // de error (nota 71 frente a 80): los lados de la placa entran en las esquinas sin girar, y sin el escalón
    // que deja el blanco junto al marco la forma entera era una curva sin ajustar, hecha de rectas cortas y con
    // los lados un pelo torcidos a lo largo del borde (ADR 0039).
    let p = resvg::tiny_skia::Pixmap::load_png(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../pruebas/vector/apolo-plano.png"
    ))
    .unwrap();
    let mut px = p.data().to_vec();
    for c in px.as_chunks_mut::<4>().0 {
        let a = c[3] as u32;
        if a > 0 && a < 255 {
            for v in &mut c[..3] {
                *v = ((*v as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    let transparente = Rgba::nueva(p.width(), p.height(), px);
    assert!(transparente.con_transparencia());
    let blanco = Rgba::nueva(p.width(), p.height(), transparente.px.clone());
    let (t, b) = (error(&transparente), error(&blanco));
    assert!(
        t <= 1.1 * b,
        "sobre transparente {t:.0}, sobre blanco {b:.0}"
    );
}
