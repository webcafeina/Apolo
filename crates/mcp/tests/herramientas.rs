//! Las diez herramientas sobre ficheros de verdad, en una carpeta temporal
//! con el corpus (ADR 0023). Lo que vigilan, sobre todo: que nunca se
//! sobrescribe nada y que los errores dicen cómo salir.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use apolo_mcp::entrada::{EntradaFormato, FormatoMcp};
use apolo_mcp::herramientas::*;
use apolo_servicio::Servicio;
use serde_json::Value;

static CUENTA: AtomicU32 = AtomicU32::new(0);

/// Una carpeta temporal con `foto.webp` y `orientacion-6.png`, y un contexto
/// con su propia carpeta de presets.
fn preparar() -> (PathBuf, Contexto) {
    let n = CUENTA.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("apolo-mcp-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("fotos")).unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus");
    for f in ["foto.webp", "orientacion-6.png"] {
        std::fs::copy(corpus.join(f), dir.join("fotos").join(f)).unwrap();
    }
    let cx = Contexto::con(Servicio::nuevo(dir.join("config")));
    (dir, cx)
}

fn ruta(p: &Path) -> String {
    p.display().to_string()
}

fn optimizar(ruta: &Path, formato: EntradaFormato) -> EntradaOptimizar {
    EntradaOptimizar {
        ruta: self::ruta(ruta),
        formato,
        salida: None,
        medir: false,
    }
}

fn webp() -> EntradaFormato {
    EntradaFormato {
        formato: Some(FormatoMcp::Webp),
        ..Default::default()
    }
}

#[test]
fn optimizar_nunca_sobrescribe_y_llega_a_la_nota_80() {
    let (dir, cx) = preparar();
    let foto = dir.join("fotos/foto.webp");
    let antes = std::fs::read(&foto).unwrap();
    let r1 = optimizar_imagen(&cx, optimizar(&foto, webp()), &Seguimiento::nada()).unwrap();
    let r2 = optimizar_imagen(&cx, optimizar(&foto, webp()), &Seguimiento::nada()).unwrap();
    // Un WebP a WebP: con -apolo, y la segunda vez con -2.
    assert_eq!(r1.datos["salida"], ruta(&dir.join("fotos/foto-apolo.webp")));
    assert_eq!(
        r2.datos["salida"],
        ruta(&dir.join("fotos/foto-apolo-2.webp"))
    );
    assert_eq!(
        std::fs::read(&foto).unwrap(),
        antes,
        "el original no se toca"
    );
    let h = &r1.datos["hallada"];
    assert_eq!(h["alcanzada"], true, "{}", r1.texto);
    assert!(h["nota"].as_f64().unwrap() >= 80.0);
    assert!(
        r1.texto.contains("Orden equivalente: cwebp"),
        "{}",
        r1.texto
    );
}

#[test]
fn una_salida_con_otra_extension_es_un_error() {
    let (dir, cx) = preparar();
    let mut e = optimizar(&dir.join("fotos/foto.webp"), webp());
    e.salida = Some(ruta(&dir.join("fotos/foto.png")));
    let err = optimizar_imagen(&cx, e, &Seguimiento::nada()).unwrap_err();
    assert_eq!(err.error, "extension_no_cuadra");
    assert!(err.pista.is_some());
    // Una carpeta sí vale.
    let mut e = optimizar(&dir.join("fotos/foto.webp"), webp());
    e.salida = Some(format!("{}/", ruta(&dir)));
    let r = optimizar_imagen(&cx, e, &Seguimiento::nada()).unwrap();
    assert_eq!(r.datos["salida"], ruta(&dir.join("foto-apolo.webp")));
}

#[test]
fn png_con_nota_es_un_error_con_pista() {
    let (dir, cx) = preparar();
    let e = EntradaFormato {
        formato: Some(FormatoMcp::Png),
        nota_objetivo: Some(80.0),
        ..Default::default()
    };
    let err = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/foto.webp"), e),
        &Seguimiento::nada(),
    )
    .unwrap_err();
    assert_eq!(err.error, "nota_no_admitida");
    assert!(err.texto().contains("PISTAS:"));
}

#[test]
fn se_endereza_solo_la_foto_que_lo_necesita() {
    let (dir, cx) = preparar();
    let girada = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/orientacion-6.png"), webp()),
        &Seguimiento::nada(),
    )
    .unwrap();
    // 64×32 con orientación 6: sale de pie, y sin orden equivalente.
    assert_eq!(
        (
            girada.datos["ancho"].as_u64(),
            girada.datos["alto"].as_u64()
        ),
        (Some(32), Some(64))
    );
    assert_eq!(girada.datos["motivo"], "enderezada");
    assert!(girada.datos["orden_equivalente"].is_null());
    let recta = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/foto.webp"), webp()),
        &Seguimiento::nada(),
    )
    .unwrap();
    assert!(recta.datos["motivo"].is_null(), "{}", recta.texto);
    assert!(recta.datos["orden_equivalente"].is_string());
}

#[test]
fn comparar_sin_guardar_no_escribe_nada() {
    let (dir, cx) = preparar();
    let e = EntradaComparar {
        ruta: ruta(&dir.join("fotos/foto.webp")),
        formatos: Some(vec![FormatoMcp::Webp, FormatoMcp::Jpeg]),
        nota_objetivo: None,
        guardar: Guardar::Ninguno,
        salida: None,
        ancho: None,
        alto: None,
        enderezar: None,
        recorte: None,
        paleta: None,
    };
    let r = comparar_formatos(&cx, e, &Seguimiento::nada()).unwrap();
    assert_eq!(std::fs::read_dir(dir.join("fotos")).unwrap().count(), 2);
    assert_eq!(r.datos["guardados"], Value::Array(vec![]));
    assert_eq!(r.datos["resultados"].as_array().unwrap().len(), 2);
    assert!(r.datos["mejor"].is_string());
}

#[test]
fn comparar_guarda_el_mejor() {
    let (dir, cx) = preparar();
    let e = EntradaComparar {
        ruta: ruta(&dir.join("fotos/foto.webp")),
        formatos: Some(vec![FormatoMcp::Webp, FormatoMcp::Png]),
        nota_objetivo: Some(70.0),
        guardar: Guardar::Mejor,
        salida: None,
        ancho: None,
        alto: None,
        enderezar: None,
        recorte: None,
        paleta: None,
    };
    let r = comparar_formatos(&cx, e, &Seguimiento::nada()).unwrap();
    let g = r.datos["guardados"].as_array().unwrap();
    assert_eq!(g.len(), 1);
    assert!(Path::new(g[0].as_str().unwrap()).exists());
}

#[test]
fn medir_dos_iguales_y_dos_que_no_miden_lo_mismo() {
    let (dir, cx) = preparar();
    let foto = ruta(&dir.join("fotos/foto.webp"));
    let r = medir_calidad(
        &cx,
        EntradaMedir {
            original: foto.clone(),
            resultado: foto.clone(),
        },
    )
    .unwrap();
    assert!(r.datos["psnr"].is_null(), "idénticas: PSNR infinito");
    assert!(r.datos["ssimulacra2"].as_f64().unwrap() > 99.0);
    let err = medir_calidad(
        &cx,
        EntradaMedir {
            original: foto,
            resultado: ruta(&dir.join("fotos/orientacion-6.png")),
        },
    )
    .unwrap_err();
    assert_eq!(err.error, "tamano_distinto");
}

fn ver(ruta: String) -> EntradaVer {
    EntradaVer {
        ruta,
        lado_max: None,
        zona: None,
        diferencias_con: None,
        tipo_mapa: Default::default(),
    }
}

#[test]
fn ver_da_jpeg_si_es_opaca_y_png_si_tiene_alfa() {
    let (dir, cx) = preparar();
    let r = ver_imagen(&cx, ver(ruta(&dir.join("fotos/foto.webp")))).unwrap();
    assert_eq!(r.imagen.as_ref().unwrap().mime, "image/jpeg");
    // Una con transparencia.
    let mut rgba = vec![200u8; 40 * 20 * 4];
    rgba[3] = 0;
    let alfa = dir.join("fotos/alfa.png");
    std::fs::write(
        &alfa,
        apolo_nucleo::formatos::png::png_de_pixeles(40, 20, &rgba, None).unwrap(),
    )
    .unwrap();
    let r = ver_imagen(&cx, ver(ruta(&alfa))).unwrap();
    assert_eq!(r.imagen.as_ref().unwrap().mime, "image/png");
    assert_eq!(r.datos["ancho"], 40);
}

#[test]
fn ver_una_zona_y_el_mapa() {
    let (dir, cx) = preparar();
    let mut e = ver(ruta(&dir.join("fotos/foto.webp")));
    e.zona = Some(Zona {
        x: 100,
        y: 100,
        ancho: 50,
        alto: 10,
    });
    assert_eq!(ver_imagen(&cx, e).unwrap_err().error, "zona_fuera");
    let mut e = ver(ruta(&dir.join("fotos/foto.webp")));
    e.zona = Some(Zona {
        x: 10,
        y: 20,
        ancho: 50,
        alto: 10,
    });
    let r = ver_imagen(&cx, e).unwrap();
    assert_eq!(
        (r.datos["ancho"].as_u64(), r.datos["alto"].as_u64()),
        (Some(50), Some(10))
    );
    // Una zona va sin pérdida si cabe: es para buscar defectos.
    assert_eq!(r.imagen.as_ref().unwrap().mime, "image/png");
    // El mapa de una optimizada frente a su original.
    let o = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/foto.webp"), webp()),
        &Seguimiento::nada(),
    )
    .unwrap();
    let mut e = ver(o.datos["salida"].as_str().unwrap().to_string());
    e.diferencias_con = Some(ruta(&dir.join("fotos/foto.webp")));
    let r = ver_imagen(&cx, e).unwrap();
    assert!(r.texto.starts_with("Mapa de diferencias"));
    assert!(r.imagen.unwrap().base64.len() <= 1024 * 1024);
}

#[test]
fn analizar_una_imagen_una_carpeta_y_lo_que_no_existe() {
    let (dir, cx) = preparar();
    let r = analizar_imagen(
        &cx,
        EntradaAnalizar {
            ruta: ruta(&dir.join("fotos/orientacion-6.png")),
        },
    )
    .unwrap();
    assert_eq!(r.datos["orientacion"], 6);
    assert!(r.texto.contains("Orientación EXIF 6"));
    let r = analizar_imagen(
        &cx,
        EntradaAnalizar {
            ruta: ruta(&dir.join("fotos")),
        },
    )
    .unwrap();
    assert_eq!(r.datos["imagenes"], 2, "{}", r.texto);
    let err = analizar_imagen(
        &cx,
        EntradaAnalizar {
            ruta: ruta(&dir.join("no-hay.png")),
        },
    )
    .unwrap_err();
    assert_eq!(err.error, "no_existe");
    std::fs::write(dir.join("texto.png"), b"hola").unwrap();
    let err = analizar_imagen(
        &cx,
        EntradaAnalizar {
            ruta: ruta(&dir.join("texto.png")),
        },
    )
    .unwrap_err();
    assert_eq!(err.error, "no_es_imagen");
}

#[test]
fn los_presets_no_se_pisan() {
    let (_dir, cx) = preparar();
    let r = listar_presets(&cx).unwrap();
    assert_eq!(
        r.datos["presets"],
        Value::Array(vec![]),
        "una lista vacía es []"
    );
    let nuevo = || EntradaGuardarPreset {
        nombre: "Web ligera".into(),
        formato: EntradaFormato {
            formato: Some(FormatoMcp::Avif),
            nota_objetivo: Some(75.0),
            ancho: Some(1920),
            ..Default::default()
        },
    };
    guardar_preset(&cx, nuevo()).unwrap();
    assert_eq!(
        guardar_preset(&cx, nuevo()).unwrap_err().error,
        "preset_existe"
    );
    let r = listar_presets(&cx).unwrap();
    assert_eq!(r.datos["presets"][0]["nombre"], "Web ligera");
    assert_eq!(r.datos["presets"][0]["nota_objetivo"], 75.0);
    // El proceso se ve: un preset que reduce el ancho lo dice.
    assert_eq!(r.datos["presets"][0]["proceso"], "ancho máx. 1920 px");
    assert!(r.texto.contains("ancho máx. 1920 px"), "{}", r.texto);
    // Y sirve de base.
    let e = EntradaFormato {
        preset: Some("Web ligera".into()),
        ..Default::default()
    };
    assert!(e.ajuste(&cx.servicio.presets(), None, 1).is_ok());
}

#[test]
fn un_lote_de_principio_a_fin() {
    let (dir, cx) = preparar();
    let r = empezar_lote(
        &cx,
        EntradaLote {
            entradas: vec![ruta(&dir.join("fotos"))],
            formatos: None,
            preset: None,
            orden: None,
            calidad: None,
            nota_objetivo: None,
            sin_perdida: false,
            ancho: None,
            alto: None,
            enderezar: None,
            recorte: None,
            paleta: None,
            mas_ligero: false,
            medir: false,
            salida: None,
        },
    )
    .unwrap();
    let id = r.datos["id"].as_str().unwrap().to_string();
    assert_eq!(r.datos["salida"], ruta(&dir.join("fotos-webp")));
    let mut desde = 0;
    let mut vistas = 0;
    loop {
        let v = ver_lote(
            &cx,
            EntradaVerLote {
                id: id.clone(),
                desde,
                esperar_segundos: 20,
            },
        )
        .unwrap();
        vistas += v.datos["nuevas"].as_array().unwrap().len();
        desde = v.datos["siguiente_desde"].as_u64().unwrap() as usize;
        if v.datos["terminado"] == true {
            assert!(v.texto.contains("Resumen: 2 convertidas"), "{}", v.texto);
            break;
        }
    }
    assert_eq!(vistas, 2);
    assert!(dir.join("fotos-webp/foto.webp").exists());
    assert!(dir.join("fotos-webp/orientacion-6.webp").exists());
    assert_eq!(
        ver_lote(
            &cx,
            EntradaVerLote {
                id: "l-999".into(),
                desde: 0,
                esperar_segundos: 0
            }
        )
        .unwrap_err()
        .error,
        "lote_no_existe"
    );
}

#[test]
fn una_optimizacion_cancelada_no_escribe_nada() {
    let (dir, cx) = preparar();
    let cancelado = std::sync::atomic::AtomicBool::new(true);
    let s = Seguimiento {
        avisar: &|_| {},
        cancelado: &cancelado,
    };
    let err =
        optimizar_imagen(&cx, optimizar(&dir.join("fotos/foto.webp"), webp()), &s).unwrap_err();
    assert_eq!(err.error, "cancelado");
    assert_eq!(std::fs::read_dir(dir.join("fotos")).unwrap().count(), 2);
}

#[test]
fn el_ancho_es_un_maximo_y_no_agranda() {
    let (dir, cx) = preparar();
    let e = EntradaFormato {
        formato: Some(FormatoMcp::Webp),
        ancho: Some(800),
        ..Default::default()
    };
    // foto.webp mide 128 × 128: pedir 800 de ancho la deja como está.
    let r = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/foto.webp"), e),
        &Seguimiento::nada(),
    )
    .unwrap();
    assert_eq!(
        (r.datos["ancho"].as_u64(), r.datos["alto"].as_u64()),
        (Some(128), Some(128))
    );
}

#[test]
fn recortar_y_reducir_la_paleta() {
    use apolo_mcp::entrada::RecorteMcp;
    let (dir, cx) = preparar();
    // Recorte de 64×32 de foto.webp (128×128), sin redimensionar.
    let e = EntradaFormato {
        formato: Some(FormatoMcp::Png),
        recorte: Some(RecorteMcp {
            x: 10,
            y: 20,
            ancho: 64,
            alto: 32,
        }),
        paleta: Some(8),
        ..Default::default()
    };
    let r = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/foto.webp"), e),
        &Seguimiento::nada(),
    )
    .unwrap();
    assert_eq!(
        (r.datos["ancho"].as_u64(), r.datos["alto"].as_u64()),
        (Some(64), Some(32))
    );
    // Como mucho 8 colores en el PNG escrito.
    let datos = std::fs::read(r.datos["salida"].as_str().unwrap()).unwrap();
    let img = apolo_nucleo::entrada::leer(&datos, Default::default()).unwrap();
    let (_, _, px) = apolo_nucleo::vista::rgba(&img).unwrap();
    let colores: std::collections::HashSet<&[u8]> = px.chunks(4).collect();
    assert!(colores.len() <= 8, "{} colores", colores.len());

    // Un recorte que empieza fuera, y una paleta imposible: errores con pista.
    let fuera = EntradaFormato {
        recorte: Some(RecorteMcp {
            x: 500,
            y: 0,
            ancho: 10,
            alto: 10,
        }),
        ..Default::default()
    };
    let err = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/foto.webp"), fuera),
        &Seguimiento::nada(),
    )
    .unwrap_err();
    assert_eq!(err.error, "recorte_fuera");
    let mala = EntradaFormato {
        paleta: Some(1000),
        ..Default::default()
    };
    let err = optimizar_imagen(
        &cx,
        optimizar(&dir.join("fotos/foto.webp"), mala),
        &Seguimiento::nada(),
    )
    .unwrap_err();
    assert_eq!(err.error, "paleta_fuera");

    // Un preset con los dos, y lo dice.
    guardar_preset(
        &cx,
        EntradaGuardarPreset {
            nombre: "Logos".into(),
            formato: EntradaFormato {
                formato: Some(FormatoMcp::Png),
                recorte: Some(RecorteMcp {
                    x: 0,
                    y: 0,
                    ancho: 100,
                    alto: 50,
                }),
                paleta: Some(16),
                ..Default::default()
            },
        },
    )
    .unwrap();
    let l = listar_presets(&cx).unwrap();
    assert_eq!(
        l.datos["presets"][0]["proceso"],
        "recorta 100×50 en 0,0 · paleta de 16 colores"
    );
}
