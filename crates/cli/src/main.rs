//! `apolo`, la línea de comandos de Apolo.
//!
//! Misma lógica y mismos presets que la ventana. `apolo webp`, `apolo jpeg`,
//! `apolo png` y `apolo qoi` aceptan las opciones de cwebp, cjpeg, oxipng y
//! qoiconv tal cual y dan el mismo fichero (ADR 0002 y 0020), más las propias
//! de Apolo con el prefijo `-apolo_`.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod comun;
mod jpeg;
mod lote;
mod png;
mod qoi;
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

    /// Convierte carpetas e imágenes enteras, en paralelo, a uno o varios formatos
    ///
    /// Las subcarpetas se repiten dentro de la salida, y nunca se sobrescribe
    /// nada: si un fichero existe, el nuevo lleva un número (foto-2.webp).
    /// Con un solo formato, las opciones de su herramienta (cwebp, cjpeg u
    /// oxipng) van detrás de «--», encima del preset.
    ///
    /// Ejemplos:
    ///   apolo lote fotos/ --preset "Fotos web" -- -q 80
    ///   apolo lote fotos/ --formato webp,jpeg --mas-ligero
    #[command(verbatim_doc_comment)]
    Lote {
        /// Carpetas o imágenes
        #[arg(required = true)]
        entradas: Vec<PathBuf>,
        /// Dónde dejarlas (por defecto, junto a la entrada: «-webp», «-jpg»… o «-apolo»)
        #[arg(short, long)]
        salida: Option<PathBuf>,
        /// Un preset guardado (apolo presets los lista); se puede repetir
        #[arg(short, long)]
        preset: Vec<String>,
        /// Formatos con sus opciones por defecto: webp, jpeg, png, qoi (separados por comas)
        #[arg(short, long)]
        formato: Vec<String>,
        /// De cada imagen, guardar solo el formato que menos pese
        #[arg(long)]
        mas_ligero: bool,
        /// Cuántas a la vez (por defecto, una por núcleo, hasta ocho)
        #[arg(short = 'j', long)]
        hilos: Option<usize>,
        /// Sin progreso: solo el resumen
        #[arg(short, long)]
        quiet: bool,
        /// Opciones de la herramienta, detrás de «--»
        #[arg(last = true, allow_hyphen_values = true)]
        herramienta: Vec<String>,
    },

    /// Codifica a JPEG con MozJPEG y las opciones de cjpeg (apolo jpeg -help)
    #[command(disable_help_flag = true)]
    Jpeg {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Optimiza un PNG con las opciones de oxipng (apolo png --help)
    #[command(disable_help_flag = true)]
    Png {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Pasa una imagen a QOI, como qoiconv: apolo qoi entrada salida.qoi
    Qoi {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

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
                let f = p.ajuste.formato;
                let sub = match f {
                    apolo_nucleo::salida::FormatoSalida::Webp => "webp",
                    apolo_nucleo::salida::FormatoSalida::Jpeg => "jpeg",
                    apolo_nucleo::salida::FormatoSalida::Png => "png",
                    apolo_nucleo::salida::FormatoSalida::Qoi => "qoi",
                };
                println!(
                    "  {:<24} {:<5} apolo {sub} -apolo_preset {:?} {}",
                    p.nombre,
                    f.nombre(),
                    p.nombre,
                    apolo_nucleo::salida::argumentos(&p.ajuste).join(" ")
                );
            }
            ExitCode::SUCCESS
        }
        Some(Accion::Webp { args }) => webp::ejecutar(&args),
        Some(Accion::Lote {
            entradas,
            salida,
            preset,
            formato,
            mas_ligero,
            hilos,
            quiet,
            herramienta,
        }) => lote::ejecutar(lote::Peticion {
            entradas,
            salida,
            presets: preset,
            formatos: formato,
            mas_ligero,
            hilos,
            herramienta,
            silencio: quiet,
        }),
        Some(Accion::Jpeg { args }) => jpeg::ejecutar(&args),
        Some(Accion::Png { args }) => png::ejecutar(&args),
        Some(Accion::Qoi { args }) => qoi::ejecutar(&args),
        None => {
            use clap::CommandFactory;
            Orden::command().print_help().ok();
            println!();
            ExitCode::SUCCESS
        }
    }
}
