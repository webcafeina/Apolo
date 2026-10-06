//! `apolo`, la línea de comandos de Apolo.
//!
//! Misma lógica y mismos presets que la ventana. `apolo webp` acepta las
//! opciones de `cwebp` tal cual (ADR 0002), más las propias de Apolo con el
//! prefijo `-apolo_`.

use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod webp;

#[derive(Parser)]
#[command(
    name = "apolo",
    about = "Optimiza imágenes con los motores de Squoosh",
    disable_version_flag = true
)]
struct Orden {
    /// Enseña la versión de Apolo y la de cada motor
    #[arg(short = 'V', long = "version")]
    version: bool,

    #[command(subcommand)]
    accion: Option<Accion>,
}

#[derive(Subcommand)]
enum Accion {
    /// Lista los motores que lleva dentro esta compilación
    Motores,

    /// Lista los presets guardados y dónde están
    Presets,

    /// Codifica a WebP con las opciones de cwebp (apolo webp -longhelp)
    #[command(disable_help_flag = true)]
    Webp {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> ExitCode {
    let orden = Orden::parse();

    if orden.version {
        println!("apolo {}", env!("CARGO_PKG_VERSION"));
        for m in apolo_nucleo::motores() {
            println!("  {} {}", m.nombre, m.version);
        }
        return ExitCode::SUCCESS;
    }

    match orden.accion {
        Some(Accion::Motores) => {
            for m in apolo_nucleo::motores() {
                println!("{}\t{}", m.nombre, m.version);
            }
            ExitCode::SUCCESS
        }
        Some(Accion::Presets) => {
            let Some(carpeta) = apolo_nucleo::presets::carpeta() else {
                eprintln!("No se encuentra la carpeta de configuración");
                return ExitCode::FAILURE;
            };
            println!("Presets en {}:", carpeta.display());
            let lista = apolo_nucleo::presets::listar(&carpeta);
            if lista.is_empty() {
                println!("  (ninguno: se guardan desde el Estudio, o a mano como JSON)");
            }
            for p in lista {
                println!(
                    "  {:<24} apolo webp -apolo_preset {:?} {}",
                    p.nombre,
                    p.nombre,
                    apolo_nucleo::cwebp::escribir_apolo(&p.webp).join(" ")
                );
            }
            ExitCode::SUCCESS
        }
        Some(Accion::Webp { args }) => webp::ejecutar(&args),
        None => {
            use clap::CommandFactory;
            Orden::command().print_help().ok();
            println!();
            ExitCode::SUCCESS
        }
    }
}
