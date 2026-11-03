//! armar-mcpb: el `.mcpb` para la publicación (ADR 0023).
//!
//!   armar-mcpb <apolo-mcp> <versión> <macos|windows|linux> <carpeta de salida>
//!   armar-mcpb --lanzador <versión> <carpeta de salida>
//!
//! Escribe `Apolo-<versión>-<sistema>.mcpb` en la carpeta y dice dónde; con
//! `--lanzador`, `Apolo-<versión>-node.mcpb`, el de Smithery (ADR 0036). En Rust
//! y no con `zip` de la shell: en la máquina Windows de GitHub no hay `zip`
//! (le pasó a Esfinge), y hace falta guardar el bit de ejecución.

use std::path::PathBuf;
use std::process::ExitCode;

use apolo_mcpb::{Sistema, armar, armar_lanzador, nombre_fichero, nombre_fichero_lanzador};
use apolo_nucleo::idioma::{self, Idioma};
use apolo_nucleo::{tr, trs};

fn main() -> ExitCode {
    // El idioma del manifiesto (ADR 0025): APOLO_IDIOMA, y si no, el del
    // sistema. La publicación pone `en`.
    idioma::poner(
        std::env::var("APOLO_IDIOMA")
            .ok()
            .and_then(|v| Idioma::de_codigo(&v))
            .unwrap_or_else(idioma::del_sistema),
    );
    let a: Vec<String> = std::env::args().skip(1).collect();
    if let [opcion, version, salida] = &a[..]
        && opcion == "--lanzador"
    {
        let version = version.trim_start_matches('v');
        return escribir(
            armar_lanzador(version),
            salida,
            &nombre_fichero_lanzador(version),
        );
    }
    let [binario, version, sistema, salida] = &a[..] else {
        eprintln!(
            "{}",
            trs!(
                "Uso: armar-mcpb <apolo-mcp> <versión> <macos|windows|linux> <carpeta de salida>\n     armar-mcpb --lanzador <versión> <carpeta de salida>",
                "Usage: armar-mcpb <apolo-mcp> <version> <macos|windows|linux> <output folder>\n       armar-mcpb --lanzador <version> <output folder>"
            )
        );
        return ExitCode::FAILURE;
    };
    let Some(sistema) = Sistema::leer(sistema) else {
        eprintln!(
            "{}",
            tr!(
                "Sistema desconocido: {sistema}",
                "Unknown system: {sistema}"
            )
        );
        return ExitCode::FAILURE;
    };
    let version = version.trim_start_matches('v');
    escribir(
        std::fs::read(binario).and_then(|b| armar(version, sistema, &b)),
        salida,
        &nombre_fichero(version, sistema),
    )
}

/// Guarda el paquete en la carpeta con su nombre y dice dónde.
fn escribir(paquete: std::io::Result<Vec<u8>>, salida: &str, nombre: &str) -> ExitCode {
    let r = paquete.and_then(|p| {
        let destino = PathBuf::from(salida).join(nombre);
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
