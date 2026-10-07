//! `apolo avif` y `apolo jxl`: son avifenc y cjxl, compilados dentro (ADR
//! 0021). Los argumentos les llegan tal cual, así que aceptan todas sus
//! opciones, su ayuda y sus errores.
//!
//! Apolo solo se pone en medio con sus opciones (`-apolo_preset`,
//! `-apolo_enderezar`) o cuando la entrada es de un formato que la
//! herramienta no abre (WebP, HEIC, TIFF…): entonces la lee él y le pasa un
//! PNG con sus píxeles.

use std::process::ExitCode;

use apolo_avifjxl::Herramienta;
use apolo_nucleo::entrada::Formato;
use apolo_nucleo::formatos::{avif, jxl};
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida};

use crate::comun;

pub fn ejecutar(formato: FormatoSalida, args: &[String]) -> ExitCode {
    let herramienta = if formato == FormatoSalida::Avif {
        Herramienta::Avifenc
    } else {
        Herramienta::Cjxl
    };
    let ayuda = args.iter().any(|a| a == "-h" || a == "--help");
    if ayuda || !hace_falta_apolo(formato, args) {
        let codigo = apolo_avifjxl::ejecutar(herramienta, args);
        if ayuda {
            println!("{AYUDA}");
        }
        return ExitCode::from(codigo.clamp(0, 255) as u8);
    }
    match correr(formato, args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Si hay opciones de Apolo, o una entrada que la herramienta no lee.
fn hace_falta_apolo(formato: FormatoSalida, args: &[String]) -> bool {
    if args.iter().any(|a| a.starts_with("-apolo_")) {
        return true;
    }
    let ficheros = match formato {
        FormatoSalida::Avif => avif::leer_orden(args).map(|o| o.ficheros),
        _ => jxl::leer_orden(args).map(|o| o.ficheros),
    };
    // Si Apolo no entiende la orden, que la juzgue la herramienta.
    let Some(entrada) = ficheros.ok().and_then(|f| f.first().cloned()) else {
        return false;
    };
    let Some(f) = std::fs::read(&entrada)
        .ok()
        .and_then(|d| Formato::adivinar(&d))
    else {
        return false;
    };
    let lee = match formato {
        FormatoSalida::Avif => matches!(f, Formato::Png | Formato::Jpeg),
        _ => matches!(
            f,
            Formato::Png | Formato::Jpeg | Formato::Pnm | Formato::Gif | Formato::Jxl
        ),
    };
    !lee
}

fn correr(formato: FormatoSalida, args: &[String]) -> Result<(), String> {
    let (preset, resto) = comun::preset(args)?;
    let mut ajuste = Ajuste {
        formato,
        ..Default::default()
    };
    if let Some(p) = &preset {
        ajuste.avif = p.ajuste.avif.clone();
        ajuste.jxl = p.ajuste.jxl.clone();
        ajuste.proceso = p.ajuste.proceso;
    }
    let ficheros = match formato {
        FormatoSalida::Avif => {
            let o = avif::leer_orden_desde(&ajuste.avif, &resto).map_err(|e| e.to_string())?;
            comun::avisar_ignoradas(&o.ignoradas);
            ajuste.avif = o.opciones;
            o.ficheros
        }
        _ => {
            let o = jxl::leer_orden_desde(&ajuste.jxl, &resto).map_err(|e| e.to_string())?;
            comun::avisar_ignoradas(&o.ignoradas);
            ajuste.jxl = o.opciones;
            o.ficheros
        }
    };
    let [entrada, destino] = ficheros.as_slice() else {
        return Err("hacen falta la entrada y la salida".into());
    };
    let datos = std::fs::read(entrada).map_err(|e| format!("no se puede leer «{entrada}»: {e}"))?;
    let img = comun::leer(&datos)?;
    if !formato.herramienta_lee(img.formato) {
        eprintln!(
            "Aviso: «{entrada}» es {}; {} no lo abriría: Apolo le pasa un PNG con sus píxeles.",
            img.formato.nombre(),
            formato.herramienta()
        );
    }
    let r = salida::codificar(&datos, &img, &ajuste, None).map_err(|e| e.to_string())?;
    std::fs::write(destino, &r.datos)
        .map_err(|e| format!("no se puede escribir «{destino}»: {e}"))?;
    eprintln!("{entrada}: {} → {} bytes", datos.len(), r.datos.len());
    Ok(())
}

const AYUDA: &str = "
Propias de Apolo:

  -apolo_preset NOMBRE   partir de un preset guardado (apolo presets)
  -apolo_enderezar       girar según la orientación EXIF

Con una entrada que la herramienta no abre (WebP, HEIC, TIFF…), Apolo la lee
y le pasa un PNG con sus píxeles.";
