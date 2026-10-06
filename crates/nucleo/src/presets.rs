//! Presets con nombre: un fichero JSON por preset (ADR 0013).
//!
//! Viven en la carpeta de configuración del usuario, `…/Apolo/presets/`, que
//! es la misma para la ventana y para la CLI: lo que se guarda en el Estudio
//! se usa con `apolo webp -apolo_preset <nombre>`, y se comparte copiando el
//! fichero.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::webp::OpcionesWebp;

/// Un preset guardado.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PresetGuardado {
    pub nombre: String,
    /// El formato de salida. Por ahora solo `webp`; con la entrega 4 llegan
    /// los demás, cada uno con su bloque de opciones.
    #[serde(default = "webp")]
    pub formato: String,
    #[serde(default)]
    pub webp: OpcionesWebp,
}

fn webp() -> String {
    "webp".into()
}

/// `…/Apolo/presets`, en la carpeta de configuración de cada sistema
/// (`~/.config` en Linux, `~/Library/Application Support` en macOS,
/// `%APPDATA%` en Windows).
pub fn carpeta() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("Apolo").join("presets"))
}

/// El nombre de fichero de un preset: minúsculas, sin tildes, con guiones.
pub fn fichero(nombre: &str) -> String {
    let mut s = String::new();
    for c in nombre.trim().to_lowercase().chars() {
        let c = match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            s.push(c);
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    let s = s.trim_end_matches('-');
    if s.is_empty() {
        "preset.json".into()
    } else {
        format!("{s}.json")
    }
}

/// Los presets de la carpeta, por nombre. Los ficheros que no se entienden se
/// saltan: un JSON roto no puede dejar sin presets al resto.
pub fn listar(carpeta: &Path) -> Vec<PresetGuardado> {
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        return vec![];
    };
    let mut v: Vec<PresetGuardado> = entradas
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| serde_json::from_slice(&std::fs::read(e.path()).ok()?).ok())
        .collect();
    v.sort_by_key(|p| p.nombre.to_lowercase());
    v
}

/// Busca un preset por nombre (sin distinguir mayúsculas) o por fichero.
pub fn cargar(carpeta: &Path, nombre: &str) -> Option<PresetGuardado> {
    let buscado = fichero(nombre);
    listar(carpeta)
        .into_iter()
        .find(|p| p.nombre.eq_ignore_ascii_case(nombre) || fichero(&p.nombre) == buscado)
}

/// Guarda (o sobrescribe) un preset.
pub fn guardar(carpeta: &Path, preset: &PresetGuardado) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(carpeta)?;
    let ruta = carpeta.join(fichero(&preset.nombre));
    let json = serde_json::to_vec_pretty(preset).map_err(std::io::Error::other)?;
    std::fs::write(&ruta, json)?;
    Ok(ruta)
}

/// Borra un preset. Que no existiera no es un error.
pub fn borrar(carpeta: &Path, nombre: &str) -> std::io::Result<()> {
    match std::fs::remove_file(carpeta.join(fichero(nombre))) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn nombres_de_fichero() {
        assert_eq!(fichero("Fotos para la web"), "fotos-para-la-web.json");
        assert_eq!(
            fichero("  Iconos (sin pérdida)  "),
            "iconos-sin-perdida.json"
        );
        assert_eq!(fichero("Ñandú"), "nandu.json");
        assert_eq!(fichero("¡¡!!"), "preset.json");
    }

    #[test]
    fn guardar_listar_cargar_borrar() {
        let dir = std::env::temp_dir().join(format!("apolo-presets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let mut p = PresetGuardado {
            nombre: "Web".into(),
            formato: "webp".into(),
            webp: OpcionesWebp::default(),
        };
        p.webp.calidad = 82.0;
        guardar(&dir, &p).unwrap();
        std::fs::write(dir.join("roto.json"), b"{no es json").unwrap();
        assert_eq!(listar(&dir), vec![p.clone()]);
        assert_eq!(cargar(&dir, "web").unwrap().webp.calidad, 82.0);
        borrar(&dir, "Web").unwrap();
        borrar(&dir, "Web").unwrap();
        assert!(cargar(&dir, "web").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
