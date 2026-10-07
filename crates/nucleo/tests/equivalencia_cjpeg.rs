//! La promesa de la ADR 0020: con las mismas opciones, el mismo fichero que el
//! `cjpeg` oficial de MozJPEG 4.1.5, byte a byte.
//!
//! Necesita el binario: `APOLO_CJPEG=/ruta/a/cjpeg` (`make cjpeg-oficial` lo
//! compila desde la fuente). Sin él, la prueba avisa y no hace nada, salvo que
//! `APOLO_CJPEG_OBLIGATORIO=1` (CI), que la hace fallar.
//!
//! El corpus es el mismo que el de cwebp. De él, cjpeg lee PNG, JPEG y PNM; lo
//! demás (TIFF, WebP, YUV) lo convierte Apolo pero no hay con qué compararlo.

use std::process::Command;

use apolo_nucleo::jpeg;

mod comun;
use comun::{corpus, tempdir};

const OPCIONES: &[&str] = &[
    "",
    "-quality 90",
    "-quality 80",
    "-quality 79.5",
    "-quality 30",
    "-quality 100",
    "-quality 85,60",
    "-quality 50 -baseline",
    "-progressive -quality 70",
    "-baseline -progressive",
    "-quant-baseline -quality 10",
    "-revert",
    "-revert -quality 60 -optimize",
    "-notrellis -revert",
    "-tune-psnr",
    "-tune-ssim -quality 65",
    "-tune-ms-ssim",
    "-quant-table 2 -tune-psnr",
    "-tune-psnr -quant-table 5",
    "-quant-table 0 -quality 75",
    "-quant-table 8",
    "-lambda1 10 -lambda2 12",
    "-grayscale",
    "-rgb -quality 70",
    "-sample 1x1",
    "-sample 2x2 -quality 95",
    "-qslots 0,1,1 -quality 60",
    "-fastcrush",
    "-dc-scan-opt 0",
    "-dc-scan-opt 2",
    "-notrellis",
    "-notrellis-dc",
    "-trellis-dc -trellis-dc-ver-weight 0.5",
    "-noovershoot",
    "-nojfif",
    "-dct float",
    "-dct fast -revert",
    "-smooth 40",
    "-restart 1",
    "-restart 20B",
    "-q 50 -dc int",
];

#[test]
fn misma_salida_que_cjpeg() {
    let Ok(cjpeg) = std::env::var("APOLO_CJPEG") else {
        assert!(
            std::env::var("APOLO_CJPEG_OBLIGATORIO").is_err(),
            "APOLO_CJPEG_OBLIGATORIO está puesto y APOLO_CJPEG no"
        );
        eprintln!("Sin APOLO_CJPEG: la equivalencia con cjpeg no se comprueba.");
        return;
    };
    let version = Command::new(&cjpeg)
        .arg("-version")
        .output()
        .expect("no se puede ejecutar cjpeg");
    let version = String::from_utf8_lossy(&version.stderr).to_string();
    assert!(
        version.contains("mozjpeg version 4.1.5"),
        "el cjpeg de referencia tiene que ser MozJPEG 4.1.5: {version}"
    );

    let dir = tempdir().join("cjpeg");
    std::fs::create_dir_all(&dir).unwrap();
    let mut fallos = Vec::new();
    let mut iguales = 0;
    for caso in corpus::generar() {
        // Lo que cjpeg lee: PNG, JPEG y PNM (P2, P3, P5 y P6).
        let lee = match caso.datos.first() {
            Some(0x89 | 0xFF) => true,
            Some(b'P') => matches!(caso.datos.get(1), Some(b'2' | b'3' | b'5' | b'6')),
            _ => false,
        };
        if !lee {
            continue;
        }
        let ruta = dir.join(&caso.nombre);
        std::fs::write(&ruta, &caso.datos).unwrap();
        for opciones in OPCIONES {
            let args: Vec<String> = opciones.split_whitespace().map(String::from).collect();
            let salida = dir.join("ref.jpg");
            let _ = std::fs::remove_file(&salida);
            let estado = Command::new(&cjpeg)
                .args(&args)
                .arg("-outfile")
                .arg(&salida)
                .arg(&ruta)
                .stderr(std::process::Stdio::null())
                .status()
                .unwrap();
            // Por el camino de la interfaz: leer las opciones, escribirlas y
            // ejecutar lo escrito. Así se prueba también la precedencia.
            let apolo = jpeg::OpcionesJpeg::leer(&args)
                .map_err(|e| e.to_string())
                .and_then(|o| {
                    let e = jpeg::leer(&caso.datos).map_err(|e| e.to_string())?;
                    jpeg::cjpeg::ejecutar(&e, &o.orden(), &Default::default())
                        .map_err(|e| e.to_string())
                });
            match (estado.success(), apolo) {
                (true, Ok(r)) => {
                    let referencia = std::fs::read(&salida).unwrap();
                    if referencia == r {
                        iguales += 1;
                    } else {
                        fallos.push(format!(
                            "{} [{opciones}]: {} bytes en cjpeg, {} en Apolo",
                            caso.nombre,
                            referencia.len(),
                            r.len()
                        ));
                    }
                }
                (false, Err(_)) => {} // los dos lo rechazan: también es coincidir
                (true, Err(e)) => fallos.push(format!(
                    "{} [{opciones}]: cjpeg sí, Apolo no: {e}",
                    caso.nombre
                )),
                (false, Ok(_)) => {
                    fallos.push(format!("{} [{opciones}]: Apolo sí, cjpeg no", caso.nombre))
                }
            }
        }
    }
    eprintln!(
        "Equivalencia con cjpeg 4.1.5: {iguales} iguales, {} distintos",
        fallos.len()
    );
    assert!(fallos.is_empty(), "\n{}", fallos.join("\n"));
}
