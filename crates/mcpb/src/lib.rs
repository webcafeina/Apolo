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

use apolo_nucleo::{tr, trs};
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
/// servidor: no hay una lista aparte que se quede atrás. En el idioma del
/// proceso (ADR 0025): el de la ventana al crearlo desde Ajustes, y en inglés
/// los de la publicación (`APOLO_IDIOMA=en`).
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
        "description": trs!(
            "Optimiza imágenes en tu ordenador con los motores de cwebp, MozJPEG, oxipng, avifenc y cjxl, sin sobrescribir nunca nada.",
            "Optimizes images on your computer with the engines of cwebp, MozJPEG, oxipng, avifenc and cjxl, never overwriting anything."
        ),
        "long_description": trs!(
            "Deja que el agente analice, optimice, compare formatos a igual calidad percibida (SSIMULACRA 2), mida la pérdida, mire las imágenes y convierta carpetas enteras con Apolo, en este ordenador. Siempre escribe ficheros nuevos: nunca sobrescribe ni borra. Para las imágenes de tu ordenador, el agente debe usar Apolo y no su propio entorno en la nube; Apolo no ve las imágenes adjuntas en la conversación, así que se le dice dónde está el fichero. Si Apolo está instalado, la extensión usa el suyo, que se actualiza con la aplicación.",
            "Lets the agent analyze, optimize, compare formats at equal perceived quality (SSIMULACRA 2), measure the loss, look at images and convert whole folders with Apolo, on this computer. It always writes new files: it never overwrites or deletes. For the images on your computer, the agent should use Apolo and not its own cloud environment; Apolo doesn't see images attached to the conversation, so you tell it where the file is. If Apolo is installed, the extension uses its copy, which updates with the app."
        ),
        "author": { "name": "Webcafeína", "url": "https://webcafeina.com" },
        // La web, que es donde se explica y se descarga (ADR 0034); el código, en el repositorio.
        "homepage": trs!("https://apolo.webcafeina.com/", "https://apolo.webcafeina.com/en/"),
        "documentation": trs!("https://apolo.webcafeina.com/agentes/", "https://apolo.webcafeina.com/en/agents/"),
        "support": "https://github.com/webcafeina/Apolo/issues",
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
        "keywords": [
            trs!("imágenes", "images"),
            "webp",
            "avif",
            "jpeg xl",
            "png",
            "jpeg",
            trs!("optimizar", "optimize"),
            trs!("comprimir", "compress"),
            trs!("convertir", "convert")
        ],
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

/// El lanzador del plugin de Claude (ADR 0036): busca el `apolo-mcp` instalado
/// y le pasa la conversación, o explica cómo instalar Apolo si no lo encuentra.
const LANZADOR: &str = include_str!("../../../empaquetado/claude/apolo/servidor/apolo-mcp.mjs");

/// El nombre del paquete con el lanzador: `Apolo-1.0.2-node.mcpb`.
pub fn nombre_fichero_lanzador(version: &str) -> String {
    format!("Apolo-{version}-node.mcpb")
}

/// El manifiesto del paquete con el lanzador, para Smithery (ADR 0036): uno
/// solo para todos los sistemas, de tipo node, porque Smithery admite un
/// paquete por versión y no aplica `platform_overrides`. No lleva `apolo-mcp`:
/// usa el de la aplicación instalada.
pub fn manifiesto_lanzador(version: &str) -> Value {
    let mut m = manifiesto(version, Sistema::de_este());
    m["long_description"] = json!(trs!(
        "Deja que el agente analice, optimice, compare formatos a igual calidad percibida (SSIMULACRA 2), mida la pérdida, mire las imágenes y convierta carpetas enteras con Apolo, en este ordenador. Siempre escribe ficheros nuevos: nunca sobrescribe ni borra. Necesita la aplicación Apolo, gratis (https://apolo.webcafeina.com/): este paquete solo lleva un lanzador que busca el apolo-mcp que va dentro de ella, y que se actualiza con la aplicación. Si no está instalada, el agente explica cómo instalarla.",
        "Lets the agent analyze, optimize, compare formats at equal perceived quality (SSIMULACRA 2), measure the loss, look at images and convert whole folders with Apolo, on this computer. It always writes new files: it never overwrites or deletes. Needs the free Apolo app (https://apolo.webcafeina.com/en/): this package only holds a launcher that finds the apolo-mcp shipped inside it, which updates with the app. If it isn't installed, the agent explains how to install it."
    ));
    m["server"] = json!({
        "type": "node",
        "entry_point": "server/apolo-mcp.mjs",
        "mcp_config": {
            "command": "node",
            "args": ["${__dirname}/server/apolo-mcp.mjs"]
        }
    });
    m["compatibility"] = json!({
        "platforms": ["darwin", "win32", "linux"],
        "runtimes": { "node": ">=18.0.0" }
    });
    m
}

/// El paquete con el lanzador: el manifiesto, el icono y el guion, unos KB.
pub fn armar_lanzador(version: &str) -> std::io::Result<Vec<u8>> {
    let mut z = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let normal = SimpleFileOptions::default().unix_permissions(0o644);
    let manifiesto = serde_json::to_vec_pretty(&manifiesto_lanzador(version))?;
    for (nombre, datos) in [
        ("manifest.json", &manifiesto[..]),
        ("icon.png", ICONO),
        ("server/apolo-mcp.mjs", LANZADOR.as_bytes()),
    ] {
        z.start_file(nombre, normal)
            .map_err(std::io::Error::other)?;
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
            Some("appimage") => trs!(
                "Desde la AppImage no se puede: su ruta cambia en cada arranque. Instala el .deb, o baja el .mcpb de la página de descargas.",
                "It can't be done from the AppImage: its path changes on every launch. Install the .deb, or download the .mcpb from the downloads page."
            )
            .into(),
            Some("traslocada") => trs!(
                "Mueve Apolo a Aplicaciones y vuelve a abrirla: ahora macOS la ejecuta desde una copia temporal.",
                "Move Apolo to Applications and open it again: right now macOS runs it from a temporary copy."
            )
            .into(),
            _ => trs!(
                "Esta Apolo no lleva apolo-mcp al lado.",
                "This Apolo doesn't have apolo-mcp next to it."
            )
            .into(),
        });
    };
    let version = env!("CARGO_PKG_VERSION");
    let servidor = std::fs::read(&ruta)
        .map_err(|e| tr!("No se puede leer {ruta}: {e}", "Can't read {ruta}: {e}"))?;
    let sistema = Sistema::de_este();
    let paquete = armar(version, sistema, &servidor).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(carpeta).map_err(|e| e.to_string())?;
    apolo_nucleo::lote::escribir_sin_pisar(
        &carpeta.join(nombre_fichero(version, sistema)),
        &paquete,
    )
    .map_err(|e| {
        tr!(
            "No se pudo guardar el paquete: {e}",
            "Couldn't save the package: {e}"
        )
    })
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
    fn el_paquete_del_lanzador_es_uno_para_todos_y_sin_binario() {
        let p = armar_lanzador("1.2.3").unwrap();
        let mut z = zip::ZipArchive::new(Cursor::new(p)).unwrap();
        let mut nombres: Vec<String> = z.file_names().map(|n| n.unwrap().to_string()).collect();
        nombres.sort();
        assert_eq!(
            nombres,
            ["icon.png", "manifest.json", "server/apolo-mcp.mjs"]
        );
        let mut m = String::new();
        z.by_name("manifest.json")
            .unwrap()
            .read_to_string(&mut m)
            .unwrap();
        let m: Value = serde_json::from_str(&m).unwrap();
        assert_eq!(m["version"], "1.2.3");
        assert_eq!(m["server"]["type"], "node");
        assert_eq!(
            m["server"]["mcp_config"]["args"],
            json!(["${__dirname}/server/apolo-mcp.mjs"])
        );
        assert_eq!(
            m["compatibility"]["platforms"],
            json!(["darwin", "win32", "linux"])
        );
        // Sin el relevo del paquete con binario: aquí no hay copia propia a la que ceder.
        assert!(m["server"]["mcp_config"].get("env").is_none());
        assert_eq!(m["tools"], manifiesto("1.2.3", Sistema::Mac)["tools"]);
        let mut guion = String::new();
        z.by_name("server/apolo-mcp.mjs")
            .unwrap()
            .read_to_string(&mut guion)
            .unwrap();
        assert_eq!(guion, LANZADOR);
        assert_eq!(nombre_fichero_lanzador("1.2.3"), "Apolo-1.2.3-node.mcpb");
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

    #[test]
    fn el_manifiesto_va_en_el_idioma_del_proceso() {
        use apolo_nucleo::idioma::{Idioma, en_este_hilo};
        let es = en_este_hilo(Idioma::Es, || manifiesto("0.0.0", Sistema::Mac));
        let en = en_este_hilo(Idioma::En, || manifiesto("0.0.0", Sistema::Mac));
        assert!(es["description"].as_str().unwrap().contains("Optimiza"));
        assert!(en["description"].as_str().unwrap().starts_with("Optimizes"));
        assert_eq!(en["keywords"][0], "images");
        // Sin nombrar a Squoosh (la fase comercial lo quitó de todo lo público).
        assert!(!es.to_string().to_lowercase().contains("squoosh"));
        assert!(!en.to_string().to_lowercase().contains("squoosh"));
        // Las herramientas, con la misma traducción que tools/list.
        let d = |m: &Value, i: usize| m["tools"][i]["description"].as_str().unwrap().to_string();
        for i in 0..es["tools"].as_array().unwrap().len() {
            assert_ne!(d(&es, i), d(&en, i));
            assert_eq!(
                Some(d(&en, i).as_str()),
                apolo_mcp::traduccion::al_ingles(&d(&es, i))
            );
        }
    }
}
