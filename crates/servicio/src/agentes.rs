//! Ajustes › Agentes (ADR 0023): dónde está el `apolo-mcp` de esta Apolo y
//! cómo se añade a cada agente. Lo usan la ventana y `apolo-dev`.
//!
//! La ruta es la del binario que va junto a la aplicación (el sidecar de
//! Tauri), que el actualizador reemplaza con cada versión: un agente que apunta
//! ahí se actualiza solo, sin reinstalar nada.

use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::json;

/// Lo que enseña la sección: la versión, la ruta y lo que se copia.
#[derive(Debug, Clone, Serialize)]
pub struct Agentes {
    pub version: &'static str,
    /// La ruta del servidor, si hay una que sirva para siempre.
    pub ruta: Option<String>,
    /// Por qué no hay ruta: `appimage` (cambia en cada arranque), `traslocada`
    /// (macOS la ejecuta desde una copia: hay que moverla a Aplicaciones) o
    /// `no_encontrado` (una compilación sin el servidor).
    pub aviso: Option<&'static str>,
    /// `claude mcp add …`, para Claude Code.
    pub claude_code: Option<String>,
    /// El bloque `mcpServers`, para Claude Desktop a mano y Cursor.
    pub json_mcp_servers: Option<String>,
    /// El de VS Code, que lee `servers` y no `mcpServers`.
    pub json_vscode: Option<String>,
}

/// El nombre del binario en este sistema.
pub fn nombre_binario(windows: bool) -> &'static str {
    if windows {
        "apolo-mcp.exe"
    } else {
        "apolo-mcp"
    }
}

/// Dónde está `apolo-mcp` para el ejecutable `exe` de la aplicación: a su
/// lado (Contents/MacOS en el Mac, la carpeta de instalación en Windows,
/// /usr/bin con el .deb). `appimage` es si corre desde una AppImage.
pub fn ruta_mcp(
    exe: &Path,
    windows: bool,
    appimage: bool,
    existe: impl Fn(&Path) -> bool,
) -> Result<PathBuf, &'static str> {
    if appimage {
        // Se monta en una carpeta distinta cada vez: una ruta de hoy no vale
        // mañana.
        return Err("appimage");
    }
    if exe.to_string_lossy().contains("/AppTranslocation/") {
        return Err("traslocada");
    }
    let ruta = exe.with_file_name(nombre_binario(windows));
    if existe(&ruta) {
        Ok(ruta)
    } else {
        Err("no_encontrado")
    }
}

/// Los fragmentos para cada agente, con la ruta dentro.
pub fn agentes(ruta: Result<PathBuf, &'static str>) -> Agentes {
    let (ruta, aviso) = match ruta {
        Ok(r) => (Some(r.display().to_string()), None),
        Err(a) => (None, Some(a)),
    };
    let bonito = |v: serde_json::Value| serde_json::to_string_pretty(&v).unwrap_or_default();
    Agentes {
        version: env!("CARGO_PKG_VERSION"),
        claude_code: ruta
            .as_ref()
            .map(|r| format!("claude mcp add --scope user apolo -- \"{r}\"")),
        json_mcp_servers: ruta
            .as_ref()
            .map(|r| bonito(json!({ "mcpServers": { "apolo": { "command": r } } }))),
        json_vscode: ruta
            .as_ref()
            .map(|r| bonito(json!({ "servers": { "apolo": { "type": "stdio", "command": r } } }))),
        ruta,
        aviso,
    }
}

/// Lo de esta Apolo, mirando el disco.
pub fn de_esta() -> Agentes {
    let ruta = std::env::current_exe()
        .map_err(|_| "no_encontrado")
        .and_then(|exe| {
            ruta_mcp(
                &exe,
                cfg!(windows),
                std::env::var_os("APPIMAGE").is_some(),
                |p| p.is_file(),
            )
        });
    agentes(ruta)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_ruta_va_junto_a_la_aplicacion() {
        let si = |_: &Path| true;
        assert_eq!(
            ruta_mcp(
                Path::new("/Applications/Apolo.app/Contents/MacOS/apolo-app"),
                false,
                false,
                si
            ),
            Ok(PathBuf::from(
                "/Applications/Apolo.app/Contents/MacOS/apolo-mcp"
            ))
        );
        assert_eq!(
            ruta_mcp(Path::new("/usr/bin/apolo-app"), false, false, si),
            Ok(PathBuf::from("/usr/bin/apolo-mcp"))
        );
        assert_eq!(
            ruta_mcp(
                Path::new("/tmp/.mount_x/usr/bin/apolo-app"),
                false,
                true,
                si
            ),
            Err("appimage")
        );
        assert_eq!(
            ruta_mcp(
                Path::new(
                    "/private/var/folders/x/AppTranslocation/y/d/Apolo.app/Contents/MacOS/apolo-app"
                ),
                false,
                false,
                si
            ),
            Err("traslocada")
        );
        assert_eq!(
            ruta_mcp(Path::new("/x/apolo-app"), false, false, |_| false),
            Err("no_encontrado")
        );
    }

    #[test]
    fn los_fragmentos_nombran_la_misma_ruta() {
        let r = r"C:\Users\Ana\AppData\Local\Apolo\apolo-mcp.exe";
        let a = agentes(Ok(PathBuf::from(r)));
        assert_eq!(
            a.claude_code.unwrap(),
            format!("claude mcp add --scope user apolo -- \"{r}\"")
        );
        let m: serde_json::Value = serde_json::from_str(&a.json_mcp_servers.unwrap()).unwrap();
        assert_eq!(m["mcpServers"]["apolo"]["command"], r);
        let v: serde_json::Value = serde_json::from_str(&a.json_vscode.unwrap()).unwrap();
        assert_eq!(v["servers"]["apolo"]["command"], r);
        assert_eq!(v["servers"]["apolo"]["type"], "stdio");
        let sin = agentes(Err("appimage"));
        assert!(sin.ruta.is_none() && sin.claude_code.is_none());
        assert_eq!(sin.aviso, Some("appimage"));
    }
}
