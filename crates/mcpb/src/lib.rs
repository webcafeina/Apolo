//! El paquete `.mcpb` de Claude Desktop (ADR 0023): un zip con el manifiesto,
//! el icono y `apolo-mcp` dentro. Se instala con un clic, sin tocar ficheros
//! de configuración.
//!
//! Lo arman igual la publicación (`armar-mcpb`, uno por sistema) y la propia
//! aplicación desde Ajustes › Agentes, con el `apolo-mcp` que lleva dentro: así
//! nadie tiene que acertar en la página de descargas con su sistema y versión.
//!
//! El servidor del paquete no se actualiza con Apolo, así que su manifiesto
//! pone `APOLO_MCP_DESDE_PAQUETE`: al arrancar le cede el turno al instalado
//! (`apolo_mcp::relevo`). Por eso basta con instalarlo una vez.

use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use zip::write::SimpleFileOptions;

/// El icono de la aplicación, 512×512.
const ICONO: &[u8] = include_bytes!("../../../src-tauri/icons/icon.png");

/// El sistema para el que va el paquete, con el nombre que usa el manifiesto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sistema {
    Mac,
    Windows,
    Linux,
}

impl Sistema {
    pub fn de_este() -> Sistema {
        match std::env::consts::OS {
            "macos" => Sistema::Mac,
            "windows" => Sistema::Windows,
            _ => Sistema::Linux,
        }
    }

    pub fn leer(s: &str) -> Option<Sistema> {
        match s {
            "macos" | "darwin" => Some(Sistema::Mac),
            "windows" | "win32" => Some(Sistema::Windows),
            "linux" => Some(Sistema::Linux),
            _ => None,
        }
    }

    fn manifiesto(self) -> &'static str {
        match self {
            Sistema::Mac => "darwin",
            Sistema::Windows => "win32",
            Sistema::Linux => "linux",
        }
    }

    /// Para el nombre del fichero: `Apolo-0.8.0-macos.mcpb`.
    pub fn nombre(self) -> &'static str {
        match self {
            Sistema::Mac => "macos",
            Sistema::Windows => "windows",
            Sistema::Linux => "linux",
        }
    }
}

/// El nombre del paquete para esta versión y sistema.
pub fn nombre_fichero(version: &str, sistema: Sistema) -> String {
    format!("Apolo-{version}-{}.mcpb", sistema.nombre())
}

/// El manifiesto (versión 0.3), con las herramientas sacadas del propio
/// servidor: no hay una lista aparte que se quede atrás.
pub fn manifiesto(version: &str, sistema: Sistema) -> Value {
    let herramientas: Vec<Value> = apolo_mcp::ServidorApolo::herramientas()
        .into_iter()
        .map(|h| {
            let d = h.description.as_deref().unwrap_or_default();
            json!({
                "name": h.name,
                "description": d.split_whitespace().collect::<Vec<_>>().join(" "),
            })
        })
        .collect();
    json!({
        "manifest_version": "0.3",
        "name": "apolo",
        "display_name": "Apolo",
        "version": version,
        "description": "Optimiza imágenes en tu ordenador con los motores de cwebp, MozJPEG, oxipng, avifenc y cjxl, sin sobrescribir nunca nada.",
        "long_description": "Deja que el agente analice, optimice, compare formatos a igual calidad percibida (SSIMULACRA 2), mida la pérdida, mire las imágenes y convierta carpetas enteras con Apolo, en este ordenador. Siempre escribe ficheros nuevos: nunca sobrescribe ni borra. Para las imágenes de tu ordenador, el agente debe usar Apolo y no su propio entorno en la nube; Apolo no ve las imágenes adjuntas en la conversación, así que se le dice dónde está el fichero. Si Apolo está instalado, la extensión usa el suyo, que se actualiza con la aplicación.",
        "author": { "name": "Webcafeína", "url": "https://webcafeina.com" },
        "homepage": "https://github.com/webcafeina/Apolo",
        "repository": { "type": "git", "url": "https://github.com/webcafeina/Apolo" },
        "license": "GPL-3.0-or-later",
        "icon": "icon.png",
        "server": {
            "type": "binary",
            // En Windows el fichero lleva .exe y el manifiesto no: lo pone
            // Claude Desktop (lo mismo en Esfinge).
            "entry_point": "server/apolo-mcp",
            "mcp_config": {
                "command": "${__dirname}/server/apolo-mcp",
                "args": [],
                "env": { apolo_mcp::relevo::DESDE_PAQUETE: "1" }
            }
        },
        "tools": herramientas,
        "keywords": ["imágenes", "webp", "avif", "jpeg xl", "optimizar", "squoosh"],
        "compatibility": { "platforms": [sistema.manifiesto()] }
    })
}

/// El paquete entero, con `servidor` (los bytes de `apolo-mcp` de ese sistema).
pub fn armar(version: &str, sistema: Sistema, servidor: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let normal = SimpleFileOptions::default().unix_permissions(0o644);
    // Sin el bit de ejecución, en macOS y Linux se instala y no arranca.
    // Nivel 1: el servidor pesa decenas de MB, y apretar más tarda mucho
    // (también al crearlo desde Ajustes) para ganar poco.
    let ejecutable = SimpleFileOptions::default()
        .unix_permissions(0o755)
        .compression_level(Some(1));
    let manifiesto = serde_json::to_vec_pretty(&manifiesto(version, sistema))?;
    let dentro = match sistema {
        Sistema::Windows => "server/apolo-mcp.exe",
        _ => "server/apolo-mcp",
    };
    for (nombre, datos, modo) in [
        ("manifest.json", &manifiesto[..], normal),
        ("icon.png", ICONO, normal),
        (dentro, servidor, ejecutable),
    ] {
        z.start_file(nombre, modo).map_err(std::io::Error::other)?;
        z.write_all(datos)?;
    }
    Ok(z.finish().map_err(std::io::Error::other)?.into_inner())
}

/// La carpeta de descargas, o la personal si no hay.
pub fn carpeta_descargas() -> PathBuf {
    dirs::download_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(std::env::temp_dir)
}

/// El paquete de esta Apolo, con el `apolo-mcp` que lleva al lado, escrito en
/// `carpeta` sin pisar nada (`Apolo-0.8.0-macos-2.mcpb` si ya hay uno). Es lo
/// que hace el botón de Ajustes › Agentes.
pub fn crear_en(carpeta: &Path) -> Result<PathBuf, String> {
    let a = apolo_servicio::agentes::de_esta();
    let Some(ruta) = a.ruta else {
        return Err(match a.aviso {
            Some("appimage") => "Desde la AppImage no se puede: su ruta cambia en cada arranque. Instala el .deb, o baja el .mcpb de la página de descargas.".into(),
            Some("traslocada") => "Mueve Apolo a Aplicaciones y vuelve a abrirla: ahora macOS la ejecuta desde una copia temporal.".into(),
            _ => "Esta Apolo no lleva apolo-mcp al lado.".into(),
        });
    };
    let version = env!("CARGO_PKG_VERSION");
    let servidor = std::fs::read(&ruta).map_err(|e| format!("No se puede leer {ruta}: {e}"))?;
    let sistema = Sistema::de_este();
    let paquete = armar(version, sistema, &servidor).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(carpeta).map_err(|e| e.to_string())?;
    apolo_nucleo::lote::escribir_sin_pisar(
        &carpeta.join(nombre_fichero(version, sistema)),
        &paquete,
    )
    .map_err(|e| format!("No se pudo guardar el paquete: {e}"))
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::io::Read;

    #[test]
    fn el_paquete_lleva_manifiesto_icono_y_servidor_ejecutable() {
        let p = armar("1.2.3", Sistema::Linux, b"binario").unwrap();
        let mut z = zip::ZipArchive::new(Cursor::new(p)).unwrap();
        let mut m = String::new();
        z.by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut m)
            .unwrap();
        let m: Value = serde_json::from_str(&m).unwrap();
        assert_eq!(m["version"], "1.2.3");
        assert_eq!(
            m["server"]["mcp_config"]["env"]["APOLO_MCP_DESDE_PAQUETE"],
            "1"
        );
        assert_eq!(m["compatibility"]["platforms"], json!(["linux"]));
        let s = z.by_name("server/apolo-mcp").unwrap();
        assert_eq!(s.unix_mode().unwrap() & 0o777, 0o755);
        drop(s);
        assert_eq!(z.by_name("icon.png").unwrap().size() as usize, ICONO.len());
        let w = armar("1.2.3", Sistema::Windows, b"x").unwrap();
        assert!(
            zip::ZipArchive::new(Cursor::new(w))
                .unwrap()
                .by_name("server/apolo-mcp.exe")
                .is_ok()
        );
    }

    #[test]
    fn las_herramientas_del_manifiesto_son_las_del_servidor() {
        let m = manifiesto("0.0.0", Sistema::Mac);
        let nombres: Vec<&str> = m["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        let del_servidor: Vec<String> = apolo_mcp::ServidorApolo::herramientas()
            .into_iter()
            .map(|h| h.name.to_string())
            .collect();
        assert_eq!(nombres, del_servidor);
        assert!(nombres.contains(&"acerca_de_apolo"));
    }
}
