//! La promesa de la ADR 0021: el mismo fichero que el `avifenc` 1.4.2 y el
//! `cjxl` 0.12.0 oficiales, byte a byte.
//!
//! Necesita los binarios: `APOLO_AVIFENC` y `APOLO_CJXL` (`make referencias`
//! los descarga). Sin ellos, la prueba avisa y no hace nada, salvo que
//! `APOLO_REFERENCIAS_OBLIGATORIAS=1` (CI), que la hace fallar.
//!
//! Las dos herramientas leen PNG y JPEG, y Apolo les pasa el fichero tal cual:
//! del corpus se usan los PNG y los JPEG. Cada combinación pasa por todo el
//! camino de Apolo: leer la orden, volver a escribirla y llamar a la
//! herramienta compilada dentro.

use std::process::{Command, Stdio};

use apolo_nucleo::formatos::{avif, jxl};

mod comun;
use comun::{corpus, tempdir};

const OPCIONES_AVIFENC: &[&str] = &[
    "",
    "-q 30",
    "-q 90 -s 4",
    "-s 10",
    "-l -s 8",
    "-y 420",
    "-y 422 -q 50",
    "-y 400",
    "-y 420 --sharpyuv",
    "-d 10",
    "-d 12 -q 70 -s 8",
    "-r limited",
    "-a tune=ssim",
    "-a tune=psnr -a sharpness=3",
    "--qalpha 30",
    "-p",
    "--progressive",
    "--ignore-exif --ignore-xmp --ignore-icc",
    "-a color:enable-chroma-deltaq=1",
    "--autotiling -s 9",
    "--min 10 --max 40",
];

const OPCIONES_CJXL: &[&str] = &[
    "",
    "-d 0",
    "-q 50",
    "-q 95 -e 3",
    "-e 1",
    "-e 9 -d 2",
    "-p",
    "-m 1 -q 80",
    "--lossless_jpeg=0",
    "--lossless_jpeg=0 -q 70",
    "-a 1",
    "--photon_noise_iso=400",
    "--epf=0 --gaborish=0",
    "--faster_decoding=2",
    "--resampling=2 -d 3",
    "-I 50 -d 0 -e 4",
    "--modular_palette_colors=0 -d 0",
    "--container=1",
    "--compress_boxes=0",
    "--progressive_dc=1",
];

fn binario(var: &str) -> Option<String> {
    match std::env::var(var) {
        Ok(v) => Some(v),
        Err(_) => {
            assert!(
                std::env::var("APOLO_REFERENCIAS_OBLIGATORIAS").is_err(),
                "APOLO_REFERENCIAS_OBLIGATORIAS está puesto y {var} no"
            );
            eprintln!("Sin {var}: esta equivalencia no se comprueba.");
            None
        }
    }
}

/// Los PNG y JPEG del corpus, con la extensión que mira la herramienta.
fn entradas() -> Vec<(comun::Caso, &'static str)> {
    corpus::generar()
        .into_iter()
        .filter_map(|c| match c.datos.first() {
            Some(0x89) => Some((c, "png")),
            Some(0xFF) => Some((c, "jpg")),
            _ => None,
        })
        .collect()
}

#[test]
fn misma_salida_que_avifenc() {
    let Some(avifenc) = binario("APOLO_AVIFENC") else {
        return;
    };
    let dir = tempdir().join("avif");
    std::fs::create_dir_all(&dir).unwrap();
    let mut fallos = Vec::new();
    let mut iguales = 0;
    for (caso, ext) in entradas() {
        let ruta = dir.join(&caso.nombre);
        std::fs::write(&ruta, &caso.datos).unwrap();
        for opciones in OPCIONES_AVIFENC {
            let args: Vec<String> = opciones.split_whitespace().map(String::from).collect();
            let salida = dir.join("ref.avif");
            let _ = std::fs::remove_file(&salida);
            let estado = Command::new(&avifenc)
                .args(&args)
                .arg(&ruta)
                .arg(&salida)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            let apolo = avif::leer_orden(&args)
                .and_then(|o| avif::codificar(&caso.datos, ext, &o.opciones))
                .map_err(|e| e.to_string());
            comparar(
                &mut fallos,
                &mut iguales,
                &caso.nombre,
                opciones,
                estado.success(),
                &salida,
                apolo,
            );
        }
    }
    eprintln!(
        "Equivalencia con avifenc 1.4.2: {iguales} iguales, {} distintos",
        fallos.len()
    );
    assert!(fallos.is_empty(), "\n{}", fallos.join("\n"));
}

#[test]
fn misma_salida_que_cjxl() {
    let Some(cjxl) = binario("APOLO_CJXL") else {
        return;
    };
    let dir = tempdir().join("jxl");
    std::fs::create_dir_all(&dir).unwrap();
    let mut fallos = Vec::new();
    let mut iguales = 0;
    for (caso, ext) in entradas() {
        let ruta = dir.join(&caso.nombre);
        std::fs::write(&ruta, &caso.datos).unwrap();
        for opciones in OPCIONES_CJXL {
            let args: Vec<String> = opciones.split_whitespace().map(String::from).collect();
            let salida = dir.join("ref.jxl");
            let _ = std::fs::remove_file(&salida);
            let estado = Command::new(&cjxl)
                .arg(&ruta)
                .arg(&salida)
                .args(&args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap();
            let apolo = jxl::leer_orden(&args)
                .and_then(|o| jxl::codificar(&caso.datos, ext, &o.opciones))
                .map_err(|e| e.to_string());
            comparar(
                &mut fallos,
                &mut iguales,
                &caso.nombre,
                opciones,
                estado.success(),
                &salida,
                apolo,
            );
        }
    }
    eprintln!(
        "Equivalencia con cjxl 0.12.0: {iguales} iguales, {} distintos",
        fallos.len()
    );
    assert!(fallos.is_empty(), "\n{}", fallos.join("\n"));
}

fn comparar(
    fallos: &mut Vec<String>,
    iguales: &mut usize,
    nombre: &str,
    opciones: &str,
    referencia_bien: bool,
    salida: &std::path::Path,
    apolo: Result<Vec<u8>, String>,
) {
    match (referencia_bien, apolo) {
        (true, Ok(r)) => {
            let referencia = std::fs::read(salida).unwrap();
            if referencia == r {
                *iguales += 1;
            } else {
                fallos.push(format!(
                    "{nombre} [{opciones}]: {} bytes en la referencia, {} en Apolo",
                    referencia.len(),
                    r.len()
                ));
            }
        }
        (false, Err(_)) => {}
        (true, Err(e)) => fallos.push(format!(
            "{nombre} [{opciones}]: la referencia sí, Apolo no: {e}"
        )),
        (false, Ok(_)) => fallos.push(format!("{nombre} [{opciones}]: Apolo sí, la referencia no")),
    }
}
