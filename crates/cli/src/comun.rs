//! Lo que comparten `apolo jpeg`, `apolo png` y `apolo qoi`.

use apolo_nucleo::entrada::{self, Imagen, Lectura};
use apolo_nucleo::presets::{self, PresetGuardado};

/// Quita `-apolo_preset <nombre>` de los argumentos y carga ese preset.
pub fn preset(args: &[String]) -> Result<(Option<PresetGuardado>, Vec<String>), String> {
    let mut resto = Vec::with_capacity(args.len());
    let mut preset = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-apolo_preset" {
            let nombre = args.get(i + 1).ok_or("-apolo_preset necesita un nombre")?;
            let carpeta =
                presets::carpeta().ok_or("no se encuentra la carpeta de configuración")?;
            preset = Some(presets::cargar(&carpeta, nombre).ok_or_else(|| {
                format!("no hay ningún preset «{nombre}» en {}", carpeta.display())
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
            "Aviso: {} no cambian el fichero y se ignoran en Apolo.",
            ignoradas.join(", ")
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
            let v = args
                .get(i + 1)
                .ok_or("-apolo_objetivo necesita una nota (de 0 a 100)")?;
            let n: f32 = v
                .replace(',', ".")
                .parse()
                .ok()
                .filter(|n| (0.0..=100.0).contains(n))
                .ok_or_else(|| format!("-apolo_objetivo {v}: la nota va de 0 a 100"))?;
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
        let nota = format!("{:.1}", h.nota).replace('.', ",");
        if h.alcanzada {
            eprintln!(
                "Calidad {} para una nota de {o} (SSIMULACRA 2 {nota}, {} pruebas)",
                h.calidad, h.pruebas
            );
        } else {
            eprintln!("Ni con calidad 100 se llega a {o}: se queda en 100 (SSIMULACRA 2 {nota})");
        }
    }
}
