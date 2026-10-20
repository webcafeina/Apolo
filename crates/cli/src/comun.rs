//! Lo que comparten `apolo jpeg`, `apolo png` y `apolo qoi`.

use apolo_nucleo::entrada::{self, Imagen, Lectura};
use apolo_nucleo::idioma::{self, Idioma};
use apolo_nucleo::presets::{self, PresetGuardado};
use apolo_nucleo::{tr, trs};

/// Un número ya escrito con punto, con la coma decimal en español: «1,5».
/// En inglés se queda el punto.
pub fn decimal(s: String) -> String {
    match idioma::actual() {
        Idioma::Es => s.replace('.', ","),
        Idioma::En => s,
    }
}

/// Quita `-apolo_preset <nombre>` de los argumentos y carga ese preset.
pub fn preset(args: &[String]) -> Result<(Option<PresetGuardado>, Vec<String>), String> {
    let mut resto = Vec::with_capacity(args.len());
    let mut preset = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-apolo_preset" {
            let nombre = args.get(i + 1).ok_or(trs!(
                "-apolo_preset necesita un nombre",
                "-apolo_preset needs a name"
            ))?;
            let carpeta = presets::carpeta().ok_or(trs!(
                "no se encuentra la carpeta de configuración",
                "can't find the settings folder"
            ))?;
            preset = Some(presets::cargar(&carpeta, nombre).ok_or_else(|| {
                tr!(
                    "no hay ningún preset «{nombre}» en {}",
                    "there's no preset “{nombre}” in {}",
                    carpeta.display()
                )
            })?);
            i += 2;
            continue;
        }
        resto.push(args[i].clone());
        i += 1;
    }
    Ok((preset, resto))
}

/// Lee una imagen como el Estudio: con metadatos, o sin ellos si están rotos.
pub fn leer(datos: &[u8]) -> Result<Imagen, String> {
    entrada::leer(datos, Lectura::default())
        .or_else(|_| {
            entrada::leer(
                datos,
                Lectura {
                    conservar_alfa: true,
                    metadatos: false,
                },
            )
        })
        .map_err(|e| e.to_string())
}

pub fn avisar_ignoradas(ignoradas: &[String]) {
    if !ignoradas.is_empty() {
        eprintln!(
            "{}",
            tr!(
                "Aviso: {} no cambian el fichero y se ignoran en Apolo.",
                "Warning: {} don't change the file and are ignored in Apolo.",
                ignoradas.join(", ")
            )
        );
    }
}

/// Quita `-apolo_objetivo <nota>` de los argumentos (ADR 0022): la nota
/// SSIMULACRA 2 que tiene que alcanzar la calidad que busque Apolo.
pub fn objetivo(args: &[String]) -> Result<(Option<f32>, Vec<String>), String> {
    let mut resto = Vec::with_capacity(args.len());
    let mut nota = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-apolo_objetivo" {
            let v = args.get(i + 1).ok_or(trs!(
                "-apolo_objetivo necesita una nota (de 0 a 100)",
                "-apolo_objetivo needs a score (0 to 100)"
            ))?;
            let n: f32 = v
                .replace(',', ".")
                .parse()
                .ok()
                .filter(|n| (0.0..=100.0).contains(n))
                .ok_or_else(|| {
                    tr!(
                        "-apolo_objetivo {v}: la nota va de 0 a 100",
                        "-apolo_objetivo {v}: the score goes from 0 to 100"
                    )
                })?;
            nota = Some(n);
            i += 2;
            continue;
        }
        resto.push(args[i].clone());
        i += 1;
    }
    Ok((nota, resto))
}

/// Cuenta en la salida de errores la calidad que encontró la búsqueda.
pub fn informar_hallada(h: Option<apolo_nucleo::salida::Hallada>, objetivo: Option<f32>) {
    if let (Some(h), Some(o)) = (h, objetivo) {
        let nota = decimal(format!("{:.1}", h.nota));
        if h.alcanzada {
            eprintln!(
                "{}",
                tr!(
                    "Calidad {} para una nota de {o} (SSIMULACRA 2 {nota}, {} pruebas)",
                    "Quality {} for a score of {o} (SSIMULACRA 2 {nota}, {} tries)",
                    h.calidad,
                    h.pruebas
                )
            );
        } else {
            eprintln!(
                "{}",
                tr!(
                    "Ni con calidad 100 se llega a {o}: se queda en 100 (SSIMULACRA 2 {nota})",
                    "Not even quality 100 reaches {o}: it stays at 100 (SSIMULACRA 2 {nota})"
                )
            );
        }
    }
}
