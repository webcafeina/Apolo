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
