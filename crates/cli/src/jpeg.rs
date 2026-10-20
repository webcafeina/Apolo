//! `apolo jpeg`: acepta las opciones de `cjpeg` (MozJPEG 4.1.5) tal cual y da
//! el mismo fichero (ADR 0020). Más `-apolo_preset` y `-apolo_enderezar`.
//!
//! Como cjpeg: `apolo jpeg [opciones] -outfile salida.jpg entrada`. Sin
//! `-outfile`, el JPEG sale por la salida normal.

use std::io::Write;
use std::process::ExitCode;

use apolo_nucleo::jpeg::{self, cjpeg, opciones};
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
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "-h" || a == "-help" || a == "--help")
    {
        println!("{}", ayuda());
        return Ok(());
    }
    let (objetivo, args) = comun::objetivo(args)?;
    let (preset, resto) = comun::preset(&args)?;
    let base = preset
        .as_ref()
        .map(|p| p.ajuste.jpeg.clone())
        .unwrap_or_default();
    let o = opciones::leer_orden_desde(&base, &resto)?;
    if o.version {
        eprintln!(
            "mozjpeg version 4.1.5 (apolo {})",
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    comun::avisar_ignoradas(&o.ignoradas);
    let entrada = o.entrada.clone().ok_or(trs!(
        "falta el fichero de entrada",
        "the input file is missing"
    ))?;
    let datos = std::fs::read(&entrada).map_err(|e| {
        tr!(
            "no se puede leer «{entrada}»: {e}",
            "can't read “{entrada}”: {e}"
        )
    })?;

    let mut ajuste = Ajuste {
        formato: FormatoSalida::Jpeg,
        jpeg: o.opciones.clone(),
        ..Default::default()
    };
    if let Some(p) = &preset {
        ajuste.proceso = p.ajuste.proceso;
    }
    ajuste.objetivo = objetivo;
    let icc = match &o.icc {
        Some(r) => Some(
            std::fs::read(r)
                .map_err(|e| tr!("no se puede leer «{r}»: {e}", "can't read “{r}”: {e}"))?,
        ),
        None => None,
    };
    let resultado = if ajuste.proceso.vacio() && !ajuste.enderezar() && objetivo.is_none() {
        // El camino de cjpeg tal cual, con -icc y -strict.
        let e = jpeg::leer(&datos).map_err(|e| e.to_string())?;
        if !e.de_cjpeg {
            eprintln!(
                "{}",
                trs!(
                    "Aviso: cjpeg no lee este formato; Apolo lo convierte igual.",
                    "Warning: cjpeg doesn't read this format; Apolo converts it anyway."
                )
            );
        }
        let extra = cjpeg::Extra {
            icc,
            estricto: o.estricto,
        };
        cjpeg::ejecutar(&e, &o.opciones.orden(), &extra).map_err(|e| e.to_string())?
    } else {
        if icc.is_some() || o.estricto {
            return Err(trs!(
                "-icc y -strict no se pueden combinar con el proceso, -apolo_enderezar ni -apolo_objetivo",
                "-icc and -strict can't be combined with processing, -apolo_enderezar or -apolo_objetivo"
            )
            .into());
        }
        let img = comun::leer(&datos)?;
        let r = salida::codificar(&datos, &img, &ajuste, None).map_err(|e| e.to_string())?;
        comun::informar_hallada(r.hallada, objetivo);
        r.datos
    };
    match &o.salida {
        Some(s) => std::fs::write(s, &resultado)
            .map_err(|e| tr!("no se puede escribir «{s}»: {e}", "can't write “{s}”: {e}"))?,
        None => std::io::stdout()
            .write_all(&resultado)
            .map_err(|e| e.to_string())?,
    }
    Ok(())
}

fn ayuda() -> &'static str {
    trs!(
        "\
Uso: apolo jpeg [opciones de cjpeg] -outfile salida.jpg entrada

Las opciones son las de cjpeg de MozJPEG 4.1.5, y el fichero es el mismo, byte
a byte. Las más usadas:

  -quality N[,...]   calidad, de 0 a 100 (75 por defecto)
  -grayscale         en gris
  -baseline          secuencial (sin progresivo)
  -progressive       progresivo (lo de MozJPEG)
  -tune-psnr, -tune-ssim, -tune-ms-ssim, -tune-hvs-psnr
  -quant-table N     tabla de cuantización (0 a 8; 3 por defecto)
  -sample HxV[,...]  submuestreo del color
  -revert            los valores de libjpeg, sin los de MozJPEG
  -notrellis, -fastcrush, -noovershoot, -nojfif, -dct int|fast|float,
  -restart N[B], -smooth N, -qslots N[,...], -icc FICHERO, -strict

Propias de Apolo:

  -apolo_preset NOMBRE   partir de un preset guardado (apolo presets)
  -apolo_enderezar       girar según la orientación EXIF
  -apolo_objetivo NOTA   buscar la calidad más baja que da esa nota SSIMULACRA 2

No están: -arithmetic (tampoco en el cjpeg oficial), -qtables y -scans.",
        "\
Usage: apolo jpeg [cjpeg switches] -outfile output.jpg input

The switches are those of cjpeg from MozJPEG 4.1.5, and the file is the same,
byte for byte. The most used ones:

  -quality N[,...]   compression quality (0..100; 5-95 is most useful range,
                     default is 75)
  -grayscale         create monochrome JPEG file
  -baseline          create baseline JPEG file (disable progressive coding)
  -progressive       create progressive JPEG file (enabled by default)
  -tune-psnr, -tune-ssim, -tune-ms-ssim, -tune-hvs-psnr
  -quant-table N     use predefined quantization table N (0-8, default 3)
  -sample HxV[,...]  set component sampling factors
  -revert            revert to standard defaults (instead of mozjpeg defaults)
  -notrellis, -fastcrush, -noovershoot, -nojfif, -dct int|fast|float,
  -restart N[B], -smooth N, -qslots N[,...], -icc FILE, -strict

Apolo's own:

  -apolo_preset NAME     start from a saved preset (apolo presets)
  -apolo_enderezar       auto-rotate according to the EXIF orientation
  -apolo_objetivo SCORE  find the lowest quality that gives that SSIMULACRA 2 score

Not available: -arithmetic (not in the official cjpeg either), -qtables and -scans."
    )
}
