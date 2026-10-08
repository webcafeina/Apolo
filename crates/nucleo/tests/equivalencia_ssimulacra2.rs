//! La nota de Apolo (ADR 0022) es la de la herramienta `ssimulacra2` de libjxl
//! 0.12.0, la que viene con cjxl: `APOLO_SSIMULACRA2` (`make referencias`).
//! Sin ella, la prueba avisa y no hace nada, salvo con
//! `APOLO_REFERENCIAS_OBLIGATORIAS=1` (CI).
//!
//! De cada imagen del corpus, la referencia y su versión en JPEG a dos
//! calidades, las dos como PNG sin perfil (sRGB): así la herramienta y Apolo
//! parten de los mismos píxeles.

use std::process::Command;

use apolo_nucleo::entrada::{self, Lectura};
use apolo_nucleo::formatos::png;
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida};
use apolo_nucleo::{medir, vista};

mod comun;
use comun::{corpus, tempdir};

#[test]
fn misma_nota_que_ssimulacra2() {
    let herramienta = match std::env::var("APOLO_SSIMULACRA2") {
        Ok(v) => v,
        Err(_) => {
            assert!(
                std::env::var("APOLO_REFERENCIAS_OBLIGATORIAS").is_err(),
                "APOLO_REFERENCIAS_OBLIGATORIAS está puesto y APOLO_SSIMULACRA2 no"
            );
            eprintln!("Sin APOLO_SSIMULACRA2: esta equivalencia no se comprueba.");
            return;
        }
    };
    let dir = tempdir().join("ssimulacra2");
    std::fs::create_dir_all(&dir).unwrap();
    let mut fallos = Vec::new();
    let mut iguales = 0;
    for caso in corpus::generar() {
        let Ok(img) = entrada::leer(&caso.datos, Lectura::default()) else {
            continue;
        };
        let Ok((w, h, referencia)) = vista::rgba(&img) else {
            continue;
        };
        if w < 8 || h < 8 {
            continue;
        }
        let ruta_ref = dir.join("referencia.png");
        std::fs::write(
            &ruta_ref,
            png::png_de_pixeles(w, h, &referencia, None).unwrap(),
        )
        .unwrap();
        for calidad in [30.0, 75.0] {
            let mut a = Ajuste {
                formato: FormatoSalida::Jpeg,
                ..Default::default()
            };
            a.jpeg.calidad = vec![calidad];
            let c = salida::codificar(&caso.datos, &img, &a, None).unwrap();
            let (_, _, distorsionada) = vista::decodificar(&c.datos).unwrap();
            // El JPEG no tiene alfa: se mide como queda, con la transparencia
            // perdida, como lo mediría el Estudio.
            let ruta_dis = dir.join("distorsionada.png");
            std::fs::write(
                &ruta_dis,
                png::png_de_pixeles(w, h, &distorsionada, None).unwrap(),
            )
            .unwrap();
            let salida = Command::new(&herramienta)
                .arg(&ruta_ref)
                .arg(&ruta_dis)
                .output()
                .unwrap();
            let oficial: f64 = String::from_utf8_lossy(&salida.stdout)
                .trim()
                .parse()
                .unwrap();
            let apolo = medir::nota(&referencia, &distorsionada, w, h)
                .unwrap()
                .unwrap();
            // La herramienta imprime 8 decimales.
            if (oficial - apolo).abs() < 1e-7 {
                iguales += 1;
            } else {
                fallos.push(format!(
                    "{} [calidad {calidad}]: oficial {oficial:.8}, Apolo {apolo:.8}",
                    caso.nombre
                ));
            }
        }
    }
    eprintln!(
        "Equivalencia con ssimulacra2 de libjxl 0.12.0: {iguales} iguales, {} distintas",
        fallos.len()
    );
    assert!(iguales > 0);
    assert!(fallos.is_empty(), "\n{}", fallos.join("\n"));
}
