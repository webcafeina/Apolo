//! `apolo qoi entrada salida.qoi`: el mismo fichero que `qoiconv` (ADR 0020).
//! QOI no tiene opciones. Con un fichero que no sea PNG (que qoiconv no lee),
//! Apolo lo convierte igual.

use std::process::ExitCode;

use apolo_nucleo::entrada::Formato;
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida};

use apolo_nucleo::{tr, trs};

use crate::comun;

pub fn ejecutar(args: &[String]) -> ExitCode {
    let (entrada, salida) = match args {
        [e, s] if s.ends_with(".qoi") => (e, s),
        _ => {
            println!(
                "{}",
                trs!(
                    "Uso: apolo qoi entrada salida.qoi",
                    "Usage: apolo qoi input output.qoi"
                )
            );
            return ExitCode::FAILURE;
        }
    };
    let r = (|| -> Result<(), String> {
        let datos = std::fs::read(entrada).map_err(|e| {
            tr!(
                "no se puede leer «{entrada}»: {e}",
                "can't read “{entrada}”: {e}"
            )
        })?;
        let img = comun::leer(&datos)?;
        if img.formato != Formato::Png {
            eprintln!(
                "{}",
                tr!(
                    "Aviso: qoiconv solo lee PNG; Apolo convierte «{entrada}» igual.",
                    "Warning: qoiconv only reads PNG; Apolo converts “{entrada}” anyway."
                )
            );
        }
        let ajuste = Ajuste {
            formato: FormatoSalida::Qoi,
            ..Default::default()
        };
        let r = salida::codificar(&datos, &img, &ajuste, None).map_err(|e| e.to_string())?;
        std::fs::write(salida, &r.datos).map_err(|e| {
            tr!(
                "no se puede escribir «{salida}»: {e}",
                "can't write “{salida}”: {e}"
            )
        })
    })();
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
