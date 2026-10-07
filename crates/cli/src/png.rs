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
        println!("{AYUDA}");
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
        return Err("falta el fichero de entrada".into());
    }
    if o.entradas.len() > 1 && (o.salida.is_some() || o.a_stdout) {
        return Err("con --out o --stdout, una sola entrada".into());
    }
    for entrada in &o.entradas {
        let datos =
            std::fs::read(entrada).map_err(|e| format!("no se puede leer «{entrada}»: {e}"))?;
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
            eprintln!("Aviso: «{entrada}» no es un PNG; oxipng no lo abriría.");
        }
        let r = salida::codificar(&datos, &img, &ajuste, None).map_err(|e| e.to_string())?;
        if o.a_stdout {
            std::io::stdout()
                .write_all(&r.datos)
                .map_err(|e| e.to_string())?;
        } else {
            let destino = o.salida.clone().unwrap_or_else(|| entrada.clone());
            std::fs::write(&destino, &r.datos)
                .map_err(|e| format!("no se puede escribir «{destino}»: {e}"))?;
            eprintln!("{entrada}: {} → {} bytes", datos.len(), r.datos.len());
        }
    }
    Ok(())
}

const AYUDA: &str = "\
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
--dry-run y --max-raw-size.";
