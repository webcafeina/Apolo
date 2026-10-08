//! `apolo`, la línea de comandos de Apolo.
//!
//! Misma lógica y mismos presets que la ventana. `apolo webp`, `apolo jpeg`,
//! `apolo png`, `apolo qoi`, `apolo avif` y `apolo jxl` aceptan las opciones de
//! cwebp, cjpeg, oxipng, qoiconv, avifenc y cjxl tal cual y dan el mismo
//! fichero (ADR 0002, 0020 y 0021), más las propias de Apolo con el prefijo
//! `-apolo_`.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

mod avifjxl;
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

    /// Mide cuánto se aleja una imagen de su original: SSIMULACRA 2, PSNR y SSIM
    ///
    /// SSIMULACRA 2 va de −∞ a 100: 90 o más no se distingue del original, 70
    /// es alta calidad, 50 se nota. Es la misma nota que la herramienta
    /// ssimulacra2 de libjxl.
    Medir {
        /// La imagen original
        original: PathBuf,
        /// La imagen a medir, del mismo tamaño
        resultado: PathBuf,
    },

    /// Convierte carpetas e imágenes enteras, en paralelo, a uno o varios formatos
    ///
    /// Las subcarpetas se repiten dentro de la salida, y nunca se sobrescribe
    /// nada: si un fichero existe, el nuevo lleva un número (foto-2.webp).
    /// Con un solo formato, las opciones de su herramienta (cwebp, cjpeg,
    /// oxipng, avifenc o cjxl) van detrás de «--», encima del preset.
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
        /// Formatos con sus opciones por defecto: webp, jpeg, png, qoi, avif, jxl (separados por comas)
        #[arg(short, long)]
        formato: Vec<String>,
        /// De cada imagen, guardar solo el formato que menos pese
        #[arg(long)]
        mas_ligero: bool,
        /// Medir la nota (SSIMULACRA 2) de cada fichero; el lote tarda más
        #[arg(long)]
        medir: bool,
        /// Buscar, en cada imagen, la calidad más baja que da esta nota SSIMULACRA 2
        #[arg(long, value_name = "NOTA")]
        objetivo: Option<f32>,
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

    /// Codifica a AVIF: es avifenc de libavif 1.4.2 (apolo avif --help)
    #[command(disable_help_flag = true)]
    Avif {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Codifica a JPEG XL: es cjxl de libjxl 0.12.0 (apolo jxl --help)
    #[command(disable_help_flag = true)]
    Jxl {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
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
                    apolo_nucleo::salida::FormatoSalida::Jpeg => "jpeg",
                    otro => otro.extension(),
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
        Some(Accion::Medir {
            original,
            resultado,
        }) => medir(&original, &resultado),
        Some(Accion::Lote {
            entradas,
            salida,
            preset,
            formato,
            mas_ligero,
            medir,
            objetivo,
            hilos,
            quiet,
            herramienta,
        }) => lote::ejecutar(lote::Peticion {
            entradas,
            salida,
            presets: preset,
            formatos: formato,
            mas_ligero,
            medir,
            objetivo,
            hilos,
            herramienta,
            silencio: quiet,
        }),
        Some(Accion::Jpeg { args }) => jpeg::ejecutar(&args),
        Some(Accion::Png { args }) => png::ejecutar(&args),
        Some(Accion::Qoi { args }) => qoi::ejecutar(&args),
        Some(Accion::Avif { args }) => {
            avifjxl::ejecutar(apolo_nucleo::salida::FormatoSalida::Avif, &args)
        }
        Some(Accion::Jxl { args }) => {
            avifjxl::ejecutar(apolo_nucleo::salida::FormatoSalida::Jxl, &args)
        }
        None => {
            use clap::CommandFactory;
            Orden::command().print_help().ok();
            println!();
            ExitCode::SUCCESS
        }
    }
}

fn medir(original: &std::path::Path, resultado: &std::path::Path) -> ExitCode {
    let leer = |r: &std::path::Path| -> Result<(u32, u32, Vec<u8>), String> {
        let datos =
            std::fs::read(r).map_err(|e| format!("no se puede leer «{}»: {e}", r.display()))?;
        let img = comun::leer(&datos)?;
        apolo_nucleo::vista::rgba(&img).map_err(|e| e.to_string())
    };
    let r = (|| -> Result<apolo_nucleo::medir::Medidas, String> {
        let (w, h, a) = leer(original)?;
        let (w2, h2, b) = leer(resultado)?;
        if (w, h) != (w2, h2) {
            return Err(format!("no miden lo mismo: {w} × {h} y {w2} × {h2}"));
        }
        apolo_nucleo::medir::medir(&a, &b, w, h).map_err(|e| e.to_string())
    })();
    match r {
        Ok(m) => {
            let coma = |s: String| s.replace('.', ",");
            match m.ssimulacra2 {
                Some(n) => println!("SSIMULACRA 2  {}", coma(format!("{n:.2}"))),
                None => println!("SSIMULACRA 2  (la imagen es menor de 8 × 8)"),
            }
            match m.psnr {
                Some(p) => println!("PSNR          {} dB", coma(format!("{p:.2}"))),
                None => println!("PSNR          ∞ (idénticas)"),
            }
            println!("SSIM          {}", coma(format!("{:.4}", m.ssim)));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
