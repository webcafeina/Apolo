//! armar-mcpb: el `.mcpb` para la publicación (ADR 0023).
//!
//!   armar-mcpb <apolo-mcp> <versión> <macos|windows|linux> <carpeta de salida>
//!
//! Escribe `Apolo-<versión>-<sistema>.mcpb` en la carpeta y dice dónde. En Rust
//! y no con `zip` de la shell: en la máquina Windows de GitHub no hay `zip`
//! (le pasó a Esfinge), y hace falta guardar el bit de ejecución.

use std::path::PathBuf;
use std::process::ExitCode;

use apolo_mcpb::{Sistema, armar, nombre_fichero};

fn main() -> ExitCode {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let [binario, version, sistema, salida] = &a[..] else {
        eprintln!(
            "Uso: armar-mcpb <apolo-mcp> <versión> <macos|windows|linux> <carpeta de salida>"
        );
        return ExitCode::FAILURE;
    };
    let Some(sistema) = Sistema::leer(sistema) else {
        eprintln!("Sistema desconocido: {sistema}");
        return ExitCode::FAILURE;
    };
    let r = std::fs::read(binario)
        .and_then(|b| armar(version.trim_start_matches('v'), sistema, &b))
        .and_then(|p| {
            let destino = PathBuf::from(salida)
                .join(nombre_fichero(version.trim_start_matches('v'), sistema));
            std::fs::create_dir_all(salida)?;
            std::fs::write(&destino, p)?;
            Ok(destino)
        });
    match r {
        Ok(d) => {
            println!("{}", d.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("armar-mcpb: {e}");
            ExitCode::FAILURE
        }
    }
}
