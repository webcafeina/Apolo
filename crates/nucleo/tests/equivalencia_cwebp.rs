//! La promesa de la ADR 0002: con las mismas opciones, el mismo fichero que el
//! `cwebp` oficial de la misma versión de libwebp, byte a byte.
//!
//! Necesita el binario: `APOLO_CWEBP=/ruta/a/cwebp`. Sin él, la prueba avisa y
//! no hace nada, salvo que `APOLO_CWEBP_OBLIGATORIO=1` (CI), que la hace fallar.
//!
//! El corpus se genera aquí, a partir de `pruebas/corpus/foto.webp` (la foto de
//! ejemplo de libwebp, BSD): cada variante cubre un camino distinto de los
//! lectores de cwebp —alfa, 16 bits, gamma, paleta, gris, metadatos, TIFF,
//! PNM, WebP, YUV— y se cruza con una lista de opciones.

use std::process::Command;

use apolo_nucleo::cwebp;

mod comun;
use comun::{corpus, tempdir};

const OPCIONES: &[&str] = &[
    "",
    "-q 90 -m 6",
    "-q 10 -m 0",
    "-q 82.5",
    "-lossless",
    "-z 0",
    "-z 9",
    "-near_lossless 60",
    "-exact -lossless",
    "-preset photo -sharp_yuv",
    "-preset drawing",
    "-preset icon",
    "-preset text -q 60",
    "-resize 64 0",
    "-resize 300 200 -resize_mode up_only",
    "-resize 50 50 -resize_mode down_only -exact",
    "-crop 3 5 40 30 -q 50",
    "-size 3000",
    "-psnr 40",
    "-mt",
    "-alpha_q 30 -exact",
    "-alpha_method 0 -alpha_filter best",
    "-alpha_filter none",
    "-noalpha",
    "-blend_alpha 0xc0e0d0",
    "-af -sns 80 -f 20 -sharpness 4 -nostrong",
    "-segments 1 -pass 4",
    "-qrange 20 60 -q 90",
    "-jpeg_like",
    "-low_memory",
    "-hint graph -lossless -q 100 -m 6",
    "-pre 2",
    "-partition_limit 70",
    "-metadata all",
    "-metadata icc,xmp",
];

#[test]
fn misma_salida_que_cwebp() {
    let Ok(cwebp) = std::env::var("APOLO_CWEBP") else {
        assert!(
            std::env::var("APOLO_CWEBP_OBLIGATORIO").is_err(),
            "APOLO_CWEBP_OBLIGATORIO está puesto y APOLO_CWEBP no"
        );
        eprintln!("Sin APOLO_CWEBP: la equivalencia con cwebp no se comprueba.");
        return;
    };
    let version = Command::new(&cwebp)
        .arg("-version")
        .output()
        .expect("no se puede ejecutar cwebp");
    let version = String::from_utf8_lossy(&version.stdout)
        .lines()
        .next()
        .unwrap_or("")
        .to_string();
    assert_eq!(
        version,
        apolo_nucleo::motores::version_libwebp(),
        "el cwebp de referencia tiene que ser de la misma versión de libwebp"
    );

    let dir = tempdir();
    let casos = corpus::generar();
    let mut fallos = Vec::new();
    let mut iguales = 0;
    for caso in &casos {
        let ruta = dir.join(&caso.nombre);
        std::fs::write(&ruta, &caso.datos).unwrap();
        for opciones in OPCIONES {
            let mut args: Vec<String> = caso.previas.clone();
            args.extend(opciones.split_whitespace().map(String::from));
            let salida = dir.join("ref.webp");
            let _ = std::fs::remove_file(&salida);
            let estado = Command::new(&cwebp)
                .arg("-quiet")
                .args(&args)
                .arg(&ruta)
                .arg("-o")
                .arg(&salida)
                .status()
                .unwrap();
            let orden = cwebp::leer(&args).unwrap();
            let apolo = cwebp::ejecutar(&orden, &caso.datos, None);
            match (estado.success(), apolo) {
                (true, Ok(r)) => {
                    let referencia = std::fs::read(&salida).unwrap();
                    if referencia == r.datos {
                        iguales += 1;
                    } else {
                        fallos.push(format!(
                            "{} [{opciones}]: {} bytes en cwebp, {} en Apolo",
                            caso.nombre,
                            referencia.len(),
                            r.datos.len()
                        ));
                    }
                }
                (false, Err(_)) => {} // los dos lo rechazan: también es coincidir
                (true, Err(e)) => fallos.push(format!(
                    "{} [{opciones}]: cwebp sí, Apolo no: {e}",
                    caso.nombre
                )),
                (false, Ok(_)) => {
                    fallos.push(format!("{} [{opciones}]: Apolo sí, cwebp no", caso.nombre))
                }
            }
        }
    }
    eprintln!(
        "Equivalencia con cwebp {version}: {iguales} iguales, {} distintos",
        fallos.len()
    );
    assert!(fallos.is_empty(), "\n{}", fallos.join("\n"));
}
