//! vectorizar_imagen (ADR 0039) sobre ficheros de verdad: dónde escribe, que nunca pisa, el formato por la
//! extensión, la nota y los errores con pista.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use apolo_mcp::herramientas::*;
use apolo_servicio::Servicio;

static CUENTA: AtomicU32 = AtomicU32::new(0);

/// Una carpeta con un logotipo (un cuadrado azul y un círculo rojo) con el fondo transparente.
fn preparar() -> (PathBuf, Contexto) {
    let n = CUENTA.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("apolo-mcp-vector-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("config")).unwrap();
    std::fs::write(dir.join("config/ajustes.json"), r#"{"idioma": "es"}"#).unwrap();
    let lado = 96u32;
    let mut px = Vec::new();
    for y in 0..lado {
        for x in 0..lado {
            let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
            let c = if ((fx - 64.0).powi(2) + (fy - 60.0).powi(2)).sqrt() < 20.0 {
                [220, 40, 40, 255]
            } else if (12.0..44.0).contains(&fx) && (14.0..46.0).contains(&fy) {
                [30, 70, 200, 255]
            } else {
                [0, 0, 0, 0]
            };
            px.extend(c);
        }
    }
    let f = std::fs::File::create(dir.join("logo.png")).unwrap();
    let mut e = png::Encoder::new(f, lado, lado);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&px).unwrap();
    let cx = Contexto::con(Servicio::nuevo(dir.join("config")));
    (dir, cx)
}

fn entrada(ruta: &Path) -> EntradaVectorizar {
    serde_json::from_value(serde_json::json!({ "ruta": ruta.display().to_string() })).unwrap()
}

fn con(ruta: &Path, extra: serde_json::Value) -> EntradaVectorizar {
    let mut v = serde_json::json!({ "ruta": ruta.display().to_string() });
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    serde_json::from_value(v).unwrap()
}

#[test]
fn junto_a_la_original_sin_pisar_y_sin_fondo() {
    let (dir, cx) = preparar();
    let logo = dir.join("logo.png");
    let r1 = vectorizar_imagen(&cx, entrada(&logo), &Seguimiento::nada()).unwrap();
    let r2 = vectorizar_imagen(&cx, entrada(&logo), &Seguimiento::nada()).unwrap();
    assert_eq!(
        r1.datos["salida"],
        dir.join("logo.svg").display().to_string()
    );
    assert_eq!(
        r2.datos["salida"],
        dir.join("logo-2.svg").display().to_string()
    );
    let svg = std::fs::read_to_string(dir.join("logo.svg")).unwrap();
    assert!(
        svg.starts_with("<svg") && !svg.contains("<rect"),
        "transparente, sin fondo: {svg}"
    );
    assert!(r1.datos["nota"].as_f64().unwrap() > 80.0, "{}", r1.texto);
    assert!(r1.texto.contains("SSIMULACRA 2"), "{}", r1.texto);
    // Las capas van apiladas: la de abajo (el rojo, de más área) cubre también el cuadrado azul de encima.
    assert_eq!(r1.datos["formas"], 3);
    assert_eq!(r1.datos["colores"], 2);
}

#[test]
fn el_formato_por_la_extension_o_pedido() {
    let (dir, cx) = preparar();
    let logo = dir.join("logo.png");
    for (salida, cabecera) in [
        ("a.pdf", &b"%PDF"[..]),
        ("a.eps", b"%!PS"),
        ("a.dxf", b"  0\nSECTION"),
    ] {
        let r = vectorizar_imagen(
            &cx,
            con(
                &logo,
                serde_json::json!({ "salida": dir.join(salida).display().to_string() }),
            ),
            &Seguimiento::nada(),
        )
        .unwrap();
        assert_eq!(r.datos["formato"], &salida[2..]);
        assert!(
            std::fs::read(dir.join(salida))
                .unwrap()
                .starts_with(cabecera)
        );
    }
    // Una carpeta y el formato pedido.
    std::fs::create_dir_all(dir.join("sal")).unwrap();
    vectorizar_imagen(
        &cx,
        con(&logo, serde_json::json!({ "salida": dir.join("sal").display().to_string(), "formato": "dxf", "estilo": "trazar_bordes", "nivel_dxf": "rectas" })),
        &Seguimiento::nada(),
    )
    .unwrap();
    let dxf = std::fs::read_to_string(dir.join("sal/logo.dxf")).unwrap();
    assert!(dxf.contains("LWPOLYLINE") && !dxf.contains("SPLINE"));
}

#[test]
fn por_nota() {
    let (dir, cx) = preparar();
    let r = vectorizar_imagen(
        &cx,
        con(
            &dir.join("logo.png"),
            serde_json::json!({ "nota_objetivo": 60, "detalle": 0 }),
        ),
        &Seguimiento::nada(),
    )
    .unwrap();
    assert_eq!(r.datos["hallado"]["alcanzada"], true, "{}", r.texto);
    assert!(r.texto.contains("Nota 60 alcanzada"), "{}", r.texto);
}

#[test]
fn errores_con_pista() {
    let (dir, cx) = preparar();
    let logo = dir.join("logo.png");
    let e = vectorizar_imagen(
        &cx,
        con(
            &logo,
            serde_json::json!({ "salida": dir.join("a.png").display().to_string() }),
        ),
        &Seguimiento::nada(),
    )
    .unwrap_err();
    assert_eq!(e.error, "extension_no_cuadra");
    assert!(e.pista.is_some());
    let e = vectorizar_imagen(
        &cx,
        con(&logo, serde_json::json!({ "salida": dir.join("a.pdf").display().to_string(), "formato": "svg" })),
        &Seguimiento::nada(),
    )
    .unwrap_err();
    assert_eq!(e.error, "extension_no_cuadra");
    let e = vectorizar_imagen(
        &cx,
        con(&logo, serde_json::json!({ "color_trazo": "rojo" })),
        &Seguimiento::nada(),
    )
    .unwrap_err();
    assert_eq!(e.error, "color_no_valido");
    let e = vectorizar_imagen(
        &cx,
        con(&logo, serde_json::json!({ "colores": 1 })),
        &Seguimiento::nada(),
    )
    .unwrap_err();
    assert_eq!(e.error, "colores_fuera");
    // Nada se ha escrito con un error.
    assert!(!dir.join("logo.svg").exists() && !dir.join("a.pdf").exists());
}

#[test]
fn una_vectorizacion_cancelada_no_escribe_nada() {
    let (dir, cx) = preparar();
    let cancelado = std::sync::atomic::AtomicBool::new(true);
    let s = Seguimiento {
        avisar: &|_| {},
        cancelado: &cancelado,
    };
    let e = vectorizar_imagen(&cx, entrada(&dir.join("logo.png")), &s).unwrap_err();
    assert_eq!(e.error, "cancelado");
    assert!(!dir.join("logo.svg").exists());
}
