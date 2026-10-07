//! La promesa de la ADR 0020 para PNG y QOI: el mismo fichero que el `oxipng`
//! 10.2.1 oficial y que `qoiconv`, byte a byte.
//!
//! Necesita los binarios: `APOLO_OXIPNG` y `APOLO_QOICONV` (`make referencias`
//! los prepara). Sin ellos, la prueba avisa y no hace nada, salvo que
//! `APOLO_REFERENCIAS_OBLIGATORIAS=1` (CI), que la hace fallar.
//!
//! Los dos leen solo PNG: del corpus se usan los PNG.

use std::process::{Command, Stdio};

use apolo_nucleo::formatos::{png, qoi};

mod comun;
use comun::{corpus, tempdir};

const OPCIONES_OXIPNG: &[&str] = &[
    "",
    "-o 0",
    "-o 1",
    "-o 3",
    "-o 4",
    "-o max",
    "-a",
    "-i on",
    "-i keep",
    "--strip safe",
    "--strip all",
    "-s -a",
    "--nx",
    "--nx -i on",
    "--nb --nc",
    "--np --ng",
    "--nz",
    "-f 0,5",
    "-f 0-4",
    "-f 9 --brute-lines 2 --brute-level 3",
    "--zc 12",
    "--zc 0",
    "-z --zi 2",
    "--fast",
    "--scale16",
    "--force",
    "--fix",
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

fn pngs() -> Vec<comun::Caso> {
    corpus::generar()
        .into_iter()
        .filter(|c| c.datos.first() == Some(&0x89))
        .collect()
}

#[test]
fn misma_salida_que_oxipng() {
    let Some(oxipng) = binario("APOLO_OXIPNG") else {
        return;
    };
    let dir = tempdir().join("oxipng");
    std::fs::create_dir_all(&dir).unwrap();
    let mut fallos = Vec::new();
    let mut iguales = 0;
    for caso in pngs() {
        let ruta = dir.join(&caso.nombre);
        std::fs::write(&ruta, &caso.datos).unwrap();
        for opciones in OPCIONES_OXIPNG {
            let args: Vec<String> = opciones.split_whitespace().map(String::from).collect();
            let salida = dir.join("ref.png");
            let _ = std::fs::remove_file(&salida);
            let estado = Command::new(&oxipng)
                .arg("-q")
                .args(&args)
                .arg("--out")
                .arg(&salida)
                .arg(&ruta)
                .stderr(Stdio::null())
                .status()
                .unwrap();
            let apolo = png::leer_orden(&args)
                .map(|o| o.opciones)
                .and_then(|o| png::optimizar(&caso.datos, &o))
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
        "Equivalencia con oxipng 10.2.1: {iguales} iguales, {} distintos",
        fallos.len()
    );
    assert!(fallos.is_empty(), "\n{}", fallos.join("\n"));
}

#[test]
fn misma_salida_que_qoiconv() {
    let Some(qoiconv) = binario("APOLO_QOICONV") else {
        return;
    };
    let dir = tempdir().join("qoi");
    std::fs::create_dir_all(&dir).unwrap();
    let mut fallos = Vec::new();
    let mut iguales = 0;
    for caso in pngs() {
        let ruta = dir.join(&caso.nombre);
        std::fs::write(&ruta, &caso.datos).unwrap();
        let salida = dir.join("ref.qoi");
        let _ = std::fs::remove_file(&salida);
        let estado = Command::new(&qoiconv)
            .arg(&ruta)
            .arg(&salida)
            .stdout(Stdio::null())
            .status()
            .unwrap();
        let apolo = qoi::leer_png(&caso.datos)
            .and_then(|e| qoi::codificar(&e))
            .map_err(|e| e.to_string());
        comparar(
            &mut fallos,
            &mut iguales,
            &caso.nombre,
            "",
            estado.success(),
            &salida,
            apolo,
        );
    }
    eprintln!(
        "Equivalencia con qoiconv: {iguales} iguales, {} distintos",
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
