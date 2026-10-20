//! `apolo png`: acepta las opciones de `oxipng` 10.2.1 y da el mismo fichero
//! (ADR 0020). Más `-apolo_preset` y `-apolo_enderezar`.
//!
//! Como oxipng: sin `--out`, **reescribe la entrada** (si gana algo). Con un
//! fichero que no sea PNG, Apolo hace antes un PNG con sus píxeles.

use std::io::Write;
use std::process::ExitCode;

use apolo_nucleo::entrada::Formato;
use apolo_nucleo::formatos::png;
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida};

use apolo_nucleo::{tr, trs};

use crate::comun;

pub fn ejecutar(args: &[String]) -> ExitCode {
    match correr(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn correr(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{}", ayuda());
        return Ok(());
    }
    let (preset, resto) = comun::preset(args)?;
    let base = preset
        .as_ref()
        .map(|p| p.ajuste.png.clone())
        .unwrap_or_default();
    let o = png::leer_orden_desde(&base, &resto).map_err(|e| e.to_string())?;
    comun::avisar_ignoradas(&o.ignoradas);
    if o.entradas.is_empty() {
        return Err(trs!("falta el fichero de entrada", "the input file is missing").into());
    }
    if o.entradas.len() > 1 && (o.salida.is_some() || o.a_stdout) {
        return Err(trs!(
            "con --out o --stdout, una sola entrada",
            "with --out or --stdout, a single input"
        )
        .into());
    }
    for entrada in &o.entradas {
        let datos = std::fs::read(entrada).map_err(|e| {
            tr!(
                "no se puede leer «{entrada}»: {e}",
                "can't read “{entrada}”: {e}"
            )
        })?;
        let img = comun::leer(&datos)?;
        let mut ajuste = Ajuste {
            formato: FormatoSalida::Png,
            png: o.opciones.clone(),
            ..Default::default()
        };
        if let Some(p) = &preset {
            ajuste.proceso = p.ajuste.proceso;
        }
        if img.formato != Formato::Png {
            eprintln!(
                "{}",
                tr!(
                    "Aviso: «{entrada}» no es un PNG; oxipng no lo abriría.",
                    "Warning: “{entrada}” isn't a PNG; oxipng wouldn't open it."
                )
            );
        }
        let r = salida::codificar(&datos, &img, &ajuste, None).map_err(|e| e.to_string())?;
        if o.a_stdout {
            std::io::stdout()
                .write_all(&r.datos)
                .map_err(|e| e.to_string())?;
        } else {
            let destino = o.salida.clone().unwrap_or_else(|| entrada.clone());
            std::fs::write(&destino, &r.datos).map_err(|e| {
                tr!(
                    "no se puede escribir «{destino}»: {e}",
                    "can't write “{destino}”: {e}"
                )
            })?;
            eprintln!("{entrada}: {} → {} bytes", datos.len(), r.datos.len());
        }
    }
    Ok(())
}

fn ayuda() -> &'static str {
    trs!(
        "\
Uso: apolo png [opciones de oxipng] entrada.png [--out salida.png]

Las opciones son las de oxipng 10.2.1, y el fichero es el mismo, byte a byte.
Como oxipng, sin --out reescribe la entrada. Las más usadas:

  -o N          nivel, de 0 a 6 o max (2 por defecto)
  -a            cambiar el color de lo transparente si comprime más
  -i on|off|keep  entrelazado (off por defecto)
  -s, --strip safe|all  quitar los metadatos que no cambian la imagen, o todos
  -z            Zopfli: más lento y algo más pequeño (--zi N, --ziwi N)
  --nx, --nb, --nc, --np, --ng, --nz, -f LISTA, --fast, --zc N,
  --scale16, --force, --fix, --brute-level N, --brute-lines N

Propias de Apolo:

  -apolo_preset NOMBRE   partir de un preset guardado (apolo presets)
  -apolo_enderezar       girar según la orientación EXIF

No están: --keep, las listas de trozos en --strip, --timeout, --dir,
--dry-run y --max-raw-size.",
        "\
Usage: apolo png [oxipng options] input.png [--out output.png]

The options are those of oxipng 10.2.1, and the file is the same, byte for byte.
Like oxipng, without --out it overwrites the input. The most used ones:

  -o N          optimization level, 0 to 6 or max (default: 2)
  -a            alter the color of transparent pixels if it compresses better
  -i on|off|keep  interlacing (default: off)
  -s, --strip safe|all  strip metadata that doesn't affect the image, or all
  -z            Zopfli: much slower, slightly smaller (--zi N, --ziwi N)
  --nx, --nb, --nc, --np, --ng, --nz, -f LIST, --fast, --zc N,
  --scale16, --force, --fix, --brute-level N, --brute-lines N

Apolo's own:

  -apolo_preset NAME     start from a saved preset (apolo presets)
  -apolo_enderezar       auto-rotate according to the EXIF orientation

Not available: --keep, chunk lists in --strip, --timeout, --dir,
--dry-run and --max-raw-size."
    )
}
