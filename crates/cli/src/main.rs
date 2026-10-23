//! `apolo`, la línea de comandos de Apolo.
//!
//! Misma lógica y mismos presets que la ventana. `apolo webp`, `apolo jpeg`,
//! `apolo png`, `apolo qoi`, `apolo avif` y `apolo jxl` aceptan las opciones de
//! cwebp, cjpeg, oxipng, qoiconv, avifenc y cjxl tal cual y dan el mismo
//! fichero (ADR 0002, 0020 y 0021), más las propias de Apolo con el prefijo
//! `-apolo_`.

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use apolo_nucleo::idioma::{self, Idioma};
use apolo_nucleo::{tr, trs};
use clap::{Parser, Subcommand};

mod avifjxl;
mod comun;
mod jpeg;
mod lote;
mod png;
mod qoi;
mod webp;

// Los textos de la ayuda van con `trs!` y no con `///`: clap los lee al
// montar la orden, después de poner el idioma (ADR 0025).
#[derive(Parser)]
#[command(
    name = "apolo",
    about = trs!(
        "Optimiza imágenes con los motores de Squoosh",
        "Optimizes images with Squoosh's engines"
    ),
    disable_version_flag = true
)]
struct Orden {
    #[arg(
        short = 'V',
        long = "version",
        help = trs!(
            "Enseña la versión de Apolo y la de cada motor",
            "Shows the version of Apolo and of each engine"
        )
    )]
    version: bool,

    // Solo para que salga en la ayuda: `main` la quita de los argumentos antes
    // de que los lea clap (ver `idioma_de_los_argumentos`).
    #[allow(dead_code)]
    #[arg(
        long = "idioma",
        global = true,
        value_name = "es|en",
        value_parser = ["es", "en"],
        hide_possible_values = true,
        display_order = 1000,
        help = trs!(
            "Idioma de los mensajes: es o en (por defecto, el del sistema)",
            "Language of the messages: es or en (by default, the system's)"
        )
    )]
    idioma: Option<String>,

    #[command(subcommand)]
    accion: Option<Accion>,
}

#[derive(Subcommand)]
enum Accion {
    #[command(about = trs!(
        "Lista los motores que lleva dentro esta compilación",
        "Lists the engines built into this build"
    ))]
    Motores,

    #[command(about = trs!(
        "Lista los presets guardados y dónde están",
        "Lists the saved presets and where they are"
    ))]
    Presets,

    #[command(
        about = trs!(
            "Mide cuánto se aleja una imagen de su original: SSIMULACRA 2, PSNR y SSIM",
            "Measures how far an image is from its original: SSIMULACRA 2, PSNR and SSIM"
        ),
        long_about = trs!(
            "Mide cuánto se aleja una imagen de su original: SSIMULACRA 2, PSNR y SSIM

SSIMULACRA 2 va de −∞ a 100: 90 o más no se distingue del original, 70 es alta calidad, 50 se nota. Es la misma nota que la herramienta ssimulacra2 de libjxl.",
            "Measures how far an image is from its original: SSIMULACRA 2, PSNR and SSIM

SSIMULACRA 2 goes from −∞ to 100: 90 or more can't be told apart from the original, 70 is high quality, 50 is noticeable. It's the same score as the ssimulacra2 tool from libjxl."
        )
    )]
    Medir {
        #[arg(
            value_name = "ORIGINAL",
            help = trs!("La imagen original", "The original image")
        )]
        original: PathBuf,
        #[arg(
            value_name = trs!("RESULTADO", "RESULT"),
            help = trs!(
                "La imagen a medir, del mismo tamaño",
                "The image to measure, of the same size"
            )
        )]
        resultado: PathBuf,
    },

    #[command(
        about = trs!(
            "Convierte carpetas e imágenes enteras, en paralelo, a uno o varios formatos",
            "Converts whole folders and images, in parallel, to one or more formats"
        ),
        long_about = trs!(
            "Convierte carpetas e imágenes enteras, en paralelo, a uno o varios formatos

Las subcarpetas se repiten dentro de la salida, y nunca se sobrescribe
nada: si un fichero existe, el nuevo lleva un número (foto-2.webp).
Con un solo formato, las opciones de su herramienta (cwebp, cjpeg,
oxipng, avifenc o cjxl) van detrás de «--», encima del preset.

Ejemplos:
  apolo lote fotos/ --preset \"Fotos web\" -- -q 80
  apolo lote fotos/ --formato webp,jpeg --mas-ligero",
            "Converts whole folders and images, in parallel, to one or more formats

Subfolders are repeated inside the output, and nothing is ever
overwritten: if a file exists, the new one gets a number (photo-2.webp).
With a single format, the options of its tool (cwebp, cjpeg, oxipng,
avifenc or cjxl) go after “--”, on top of the preset.

Examples:
  apolo lote photos/ --preset \"Web photos\" -- -q 80
  apolo lote photos/ --formato webp,jpeg --mas-ligero"
        )
    )]
    Lote {
        #[arg(
            required = true,
            value_name = trs!("ENTRADAS", "INPUTS"),
            help = trs!("Carpetas o imágenes", "Folders or images")
        )]
        entradas: Vec<PathBuf>,
        #[arg(
            short,
            long,
            value_name = trs!("SALIDA", "OUTPUT"),
            help = trs!(
                "Dónde dejarlas (por defecto, junto a la carpeta de entrada: «-webp», «-jpg»… o «-apolo»; con imágenes sueltas, una subcarpeta «webp», «apolo»… junto a ellas)",
                "Where to put them (by default, next to the input folder: “-webp”, “-jpg”… or “-apolo”; with loose images, a subfolder “webp”, “apolo”… next to them)"
            )
        )]
        salida: Option<PathBuf>,
        #[arg(
            short,
            long,
            value_name = "PRESET",
            help = trs!(
                "Un preset guardado (apolo presets los lista); se puede repetir",
                "A saved preset (apolo presets lists them); can be repeated"
            )
        )]
        preset: Vec<String>,
        #[arg(
            short,
            long,
            value_name = trs!("FORMATO", "FORMAT"),
            help = trs!(
                "Formatos con sus opciones por defecto: webp, jpeg, png, qoi, avif, jxl (separados por comas)",
                "Formats with their default options: webp, jpeg, png, qoi, avif, jxl (comma-separated)"
            )
        )]
        formato: Vec<String>,
        #[arg(
            long,
            help = trs!(
                "De cada imagen, guardar solo el formato que menos pese",
                "For each image, save only the lightest format"
            )
        )]
        mas_ligero: bool,
        #[arg(
            long,
            help = trs!(
                "Medir la nota (SSIMULACRA 2) de cada fichero; el lote tarda más",
                "Measure the score (SSIMULACRA 2) of each file; the batch takes longer"
            )
        )]
        medir: bool,
        #[arg(
            long,
            value_name = trs!("NOTA", "SCORE"),
            help = trs!(
                "Buscar, en cada imagen, la calidad más baja que da esta nota SSIMULACRA 2",
                "Find, for each image, the lowest quality that gives this SSIMULACRA 2 score"
            )
        )]
        objetivo: Option<f32>,
        #[arg(
            long,
            value_name = "PX",
            conflicts_with = "porcentaje",
            help = trs!(
                "Reducir cada imagen a este lado mayor, en píxeles (nunca agranda), encima del preset",
                "Shrink each image to this longest side, in pixels (never enlarges), over the preset"
            )
        )]
        lado_mayor: Option<u32>,
        #[arg(
            long,
            value_name = "%",
            help = trs!(
                "Reducir cada imagen a este tanto por ciento (de más de 0 a 100), encima del preset",
                "Shrink each image to this percentage (above 0 to 100), over the preset"
            )
        )]
        porcentaje: Option<f64>,
        #[arg(
            short = 'j',
            long,
            value_name = trs!("HILOS", "THREADS"),
            help = trs!(
                "Cuántas a la vez (por defecto, una por núcleo, hasta ocho)",
                "How many at once (by default, one per core, up to eight)"
            )
        )]
        hilos: Option<usize>,
        #[arg(
            short,
            long,
            help = trs!("Sin progreso: solo el resumen", "No progress: only the summary")
        )]
        quiet: bool,
        #[arg(
            last = true,
            allow_hyphen_values = true,
            value_name = trs!("HERRAMIENTA", "TOOL"),
            help = trs!(
                "Opciones de la herramienta, detrás de «--»",
                "Options of the tool, after “--”"
            )
        )]
        herramienta: Vec<String>,
    },

    #[command(
        disable_help_flag = true,
        about = trs!(
            "Codifica a AVIF: es avifenc de libavif 1.4.2 (apolo avif --help)",
            "Encodes to AVIF: it's avifenc from libavif 1.4.2 (apolo avif --help)"
        )
    )]
    Avif {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(
        disable_help_flag = true,
        about = trs!(
            "Codifica a JPEG XL: es cjxl de libjxl 0.12.0 (apolo jxl --help)",
            "Encodes to JPEG XL: it's cjxl from libjxl 0.12.0 (apolo jxl --help)"
        )
    )]
    Jxl {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(
        disable_help_flag = true,
        about = trs!(
            "Codifica a JPEG con MozJPEG y las opciones de cjpeg (apolo jpeg -help)",
            "Encodes to JPEG with MozJPEG and the options of cjpeg (apolo jpeg -help)"
        )
    )]
    Jpeg {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(
        disable_help_flag = true,
        about = trs!(
            "Optimiza un PNG con las opciones de oxipng (apolo png --help)",
            "Optimizes a PNG with the options of oxipng (apolo png --help)"
        )
    )]
    Png {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(about = trs!(
        "Pasa una imagen a QOI, como qoiconv: apolo qoi entrada salida.qoi",
        "Converts an image to QOI, like qoiconv: apolo qoi input output.qoi"
    ))]
    Qoi {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    #[command(
        disable_help_flag = true,
        about = trs!(
            "Codifica a WebP con las opciones de cwebp (apolo webp -longhelp)",
            "Encodes to WebP with the options of cwebp (apolo webp -longhelp)"
        )
    )]
    Webp {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Quita `--idioma es|en` (o `--idioma=es|en`) de los argumentos, esté donde
/// esté, y dice qué idioma pedía. Se quita antes de que lea clap porque
/// `apolo webp`, `jpeg`, `png`, `qoi`, `avif` y `jxl` pasan sus argumentos
/// tal cual a la herramienta, que no la conoce. Detrás de `--` ya no se mira:
/// todo es de la herramienta.
fn idioma_de_los_argumentos(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(Vec<OsString>, Option<Idioma>), String> {
    let mut resto = Vec::new();
    let mut pedido = None;
    let mut args = args.into_iter();
    let leer = |v: Option<&str>| -> Result<Idioma, String> {
        let Some(v) = v else {
            return Err(trs!(
                "--idioma necesita un idioma: es o en",
                "--idioma needs a language: es or en"
            )
            .into());
        };
        match v {
            "es" => Ok(Idioma::Es),
            "en" => Ok(Idioma::En),
            otro => Err(tr!("--idioma {otro}: es o en", "--idioma {otro}: es or en")),
        }
    };
    while let Some(a) = args.next() {
        match a.to_str() {
            Some("--") => {
                resto.push(a);
                resto.extend(args);
                break;
            }
            Some("--idioma") => {
                let v = args.next();
                pedido = Some(leer(v.as_ref().and_then(|v| v.to_str()))?);
            }
            Some(t) if t.starts_with("--idioma=") => {
                pedido = Some(leer(Some(&t["--idioma=".len()..]))?);
            }
            _ => resto.push(a),
        }
    }
    Ok((resto, pedido))
}

fn main() -> ExitCode {
    apolo_nucleo::memoria::preparar();
    // El idioma, antes que clap: la ayuda sale en él (ADR 0025).
    idioma::poner(idioma::del_sistema());
    let args = match idioma_de_los_argumentos(std::env::args_os()) {
        Ok((args, pedido)) => {
            if let Some(i) = pedido {
                idioma::poner(i);
            }
            args
        }
        Err(e) => {
            eprintln!("Error: {e}");
            return ExitCode::from(2);
        }
    };
    let orden = Orden::parse_from(args);

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
                eprintln!(
                    "{}",
                    trs!(
                        "No se encuentra la carpeta de configuración",
                        "Can't find the settings folder"
                    )
                );
                return ExitCode::FAILURE;
            };
            println!(
                "{}",
                tr!("Presets en {}:", "Presets in {}:", carpeta.display())
            );
            let lista = apolo_nucleo::presets::listar(&carpeta);
            if lista.is_empty() {
                println!(
                    "{}",
                    trs!(
                        "  (ninguno: se guardan desde el Estudio, o a mano como JSON)",
                        "  (none: they're saved from the Studio, or by hand as JSON)"
                    )
                );
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
            lado_mayor,
            porcentaje,
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
            lado_mayor,
            porcentaje,
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
        let datos = std::fs::read(r).map_err(|e| {
            tr!(
                "no se puede leer «{}»: {e}",
                "can't read “{}”: {e}",
                r.display()
            )
        })?;
        let img = comun::leer(&datos)?;
        apolo_nucleo::vista::rgba(&img).map_err(|e| e.to_string())
    };
    let r = (|| -> Result<apolo_nucleo::medir::Medidas, String> {
        let (w, h, a) = leer(original)?;
        let (w2, h2, b) = leer(resultado)?;
        if (w, h) != (w2, h2) {
            return Err(tr!(
                "no miden lo mismo: {w} × {h} y {w2} × {h2}",
                "they aren't the same size: {w} × {h} and {w2} × {h2}"
            ));
        }
        apolo_nucleo::medir::medir(&a, &b, w, h).map_err(|e| e.to_string())
    })();
    match r {
        Ok(m) => {
            match m.ssimulacra2 {
                Some(n) => println!("SSIMULACRA 2  {}", comun::decimal(format!("{n:.2}"))),
                None => println!(
                    "SSIMULACRA 2  {}",
                    trs!(
                        "(la imagen es menor de 8 × 8)",
                        "(the image is smaller than 8 × 8)"
                    )
                ),
            }
            match m.psnr {
                Some(p) => println!("PSNR          {} dB", comun::decimal(format!("{p:.2}"))),
                None => println!("PSNR          ∞ {}", trs!("(idénticas)", "(identical)")),
            }
            println!("SSIM          {}", comun::decimal(format!("{:.4}", m.ssim)));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn quitar(args: &[&str]) -> Result<(Vec<String>, Option<Idioma>), String> {
        idioma_de_los_argumentos(args.iter().map(OsString::from)).map(|(v, i)| {
            let v = v.into_iter().map(|a| a.into_string().unwrap()).collect();
            (v, i)
        })
    }

    #[test]
    fn quita_idioma_de_donde_este() {
        let (v, i) = quitar(&["apolo", "webp", "--idioma", "en", "a.png", "-o", "a.webp"]).unwrap();
        assert_eq!(v, ["apolo", "webp", "a.png", "-o", "a.webp"]);
        assert_eq!(i, Some(Idioma::En));
        let (v, i) = quitar(&["apolo", "--idioma=es", "jpeg", "a.png"]).unwrap();
        assert_eq!(v, ["apolo", "jpeg", "a.png"]);
        assert_eq!(i, Some(Idioma::Es));
        let (v, i) = quitar(&["apolo", "motores"]).unwrap();
        assert_eq!(v, ["apolo", "motores"]);
        assert_eq!(i, None);
    }

    #[test]
    fn detras_de_dos_guiones_es_de_la_herramienta() {
        let (v, i) = quitar(&["apolo", "lote", "f", "--", "--idioma", "en"]).unwrap();
        assert_eq!(v, ["apolo", "lote", "f", "--", "--idioma", "en"]);
        assert_eq!(i, None);
    }

    #[test]
    fn un_idioma_que_no_hay_es_un_error() {
        assert!(quitar(&["apolo", "--idioma", "fr"]).is_err());
        assert!(quitar(&["apolo", "--idioma"]).is_err());
    }

    #[test]
    fn la_orden_de_clap_es_valida() {
        use clap::CommandFactory;
        Orden::command().debug_assert();
    }
}
