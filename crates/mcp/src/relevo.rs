//! El relevo (ADR 0023): la copia de `apolo-mcp` que va dentro del `.mcpb` de
//! Claude Desktop le cede el turno a la de la aplicación instalada.
//!
//! El `.mcpb` lleva el servidor dentro, y actualizar Apolo no lo actualiza: el
//! agente seguiría con las herramientas de la versión con la que se instaló el
//! paquete (le pasó a Esfinge, `internal/mcpb/relevo.go`). Así, el paquete se
//! instala una vez y lo que corre es siempre lo de la aplicación, que el
//! actualizador mantiene al día.
//!
//! Solo releva la copia del paquete (su manifiesto pone
//! `APOLO_MCP_DESDE_PAQUETE`), y nunca a sí misma. Todo está escrito para no
//! romperse: sin aplicación, o si el otro no arranca, sigue la copia del
//! paquete, que funciona igual aunque no se actualice.

use std::path::{Path, PathBuf};

/// Lo que pone el manifiesto del `.mcpb` para que esta copia sepa que lo es.
pub const DESDE_PAQUETE: &str = "APOLO_MCP_DESDE_PAQUETE";
/// Lo que el relevo le pasa a la instalada: la versión del paquete que le
/// cedió el turno, para que `acerca_de_apolo` lo cuente.
pub const RELEVADO_DESDE: &str = "APOLO_MCP_RELEVADO_DESDE";

/// Dónde puede estar `apolo-mcp` instalado con Apolo, en orden. Las variables
/// de entorno se pasan para poder probar los tres sistemas desde cualquiera.
pub fn donde_buscar(
    sistema: &str,
    casa: Option<&str>,
    variable: impl Fn(&str) -> Option<String>,
) -> Vec<String> {
    // Unidas con el separador del sistema de destino, no con el de aquí: si no,
    // la tabla de Windows no se podría comprobar desde Linux.
    let unir = |partes: &[&str]| -> String {
        let sep = if sistema == "windows" { "\\" } else { "/" };
        let mut s = String::new();
        for p in partes.iter().filter(|p| !p.is_empty()) {
            if s.is_empty() {
                s = p.trim_end_matches(sep).to_string();
            } else {
                s.push_str(sep);
                s.push_str(p.trim_matches(|c| c == '/' || c == '\\'));
            }
        }
        s
    };
    let mut v = Vec::new();
    match sistema {
        "macos" => {
            v.push(unir(&[
                "/Applications",
                "Apolo.app",
                "Contents",
                "MacOS",
                "apolo-mcp",
            ]));
            if let Some(c) = casa {
                v.push(unir(&[
                    c,
                    "Applications",
                    "Apolo.app",
                    "Contents",
                    "MacOS",
                    "apolo-mcp",
                ]));
            }
        }
        "windows" => {
            // El instalador de Tauri: por usuario en %LOCALAPPDATA%\Apolo (lo
            // que hace por defecto), o para todos en Program Files.
            if let Some(l) = variable("LOCALAPPDATA") {
                v.push(unir(&[&l, "Apolo", "apolo-mcp.exe"]));
            }
            for pf in ["ProgramW6432", "ProgramFiles"] {
                if let Some(p) = variable(pf) {
                    let r = unir(&[&p, "Apolo", "apolo-mcp.exe"]);
                    if !v.contains(&r) {
                        v.push(r);
                    }
                }
            }
        }
        _ => {
            // El .deb lo deja en /usr/bin; el paquete de la CLI, donde se ponga.
            v.push("/usr/bin/apolo-mcp".into());
            v.push("/usr/local/bin/apolo-mcp".into());
            if let Some(c) = casa {
                v.push(unir(&[c, ".local", "bin", "apolo-mcp"]));
            }
        }
    }
    v
}

/// A quién cederle el turno, o `None` para seguir. `yo` es este ejecutable:
/// cederse a uno mismo sería una cadena sin fin de procesos.
pub fn a_quien_ceder(
    sitios: &[String],
    yo: &Path,
    existe: impl Fn(&Path) -> bool,
) -> Option<PathBuf> {
    let yo = limpia(yo);
    sitios
        .iter()
        .map(PathBuf::from)
        .find(|s| existe(s))
        .filter(|s| limpia(s) != yo)
}

fn limpia(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

/// Lanza la instalada con esta misma entrada, la salida del protocolo y la de
/// errores, y espera. `None` si no arrancó: entonces sigue esta copia.
pub fn relevar(otro: &Path, salida: std::fs::File) -> Option<i32> {
    let mut hijo = std::process::Command::new(otro)
        .stdin(std::process::Stdio::inherit())
        .stdout(salida)
        .stderr(std::process::Stdio::inherit())
        .env_remove(DESDE_PAQUETE)
        .env(RELEVADO_DESDE, env!("CARGO_PKG_VERSION"))
        .spawn()
        .ok()?;
    // A partir de aquí manda la instalada; su código de salida es el nuestro.
    Some(hijo.wait().ok().and_then(|e| e.code()).unwrap_or(1))
}

/// Si esta copia es la del paquete y hay una instalada, le cede el turno.
pub fn ceder_si_toca() -> Option<PathBuf> {
    std::env::var_os(DESDE_PAQUETE)?;
    let yo = std::env::current_exe().ok()?;
    let casa = dirs::home_dir().map(|c| c.display().to_string());
    let sitios = donde_buscar(std::env::consts::OS, casa.as_deref(), |v| {
        std::env::var(v).ok()
    });
    a_quien_ceder(&sitios, &yo, se_puede_ejecutar)
}

fn se_puede_ejecutar(p: &Path) -> bool {
    let Ok(m) = std::fs::metadata(p) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        m.is_file() && m.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        m.is_file()
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn variables(v: &str) -> Option<String> {
        match v {
            "LOCALAPPDATA" => Some(r"C:\Users\ana\AppData\Local".into()),
            "ProgramW6432" | "ProgramFiles" => Some(r"C:\Program Files".into()),
            _ => None,
        }
    }

    #[test]
    fn donde_esta_instalado_en_cada_sistema() {
        assert_eq!(
            donde_buscar("macos", Some("/Users/ana"), variables),
            [
                "/Applications/Apolo.app/Contents/MacOS/apolo-mcp",
                "/Users/ana/Applications/Apolo.app/Contents/MacOS/apolo-mcp"
            ]
        );
        assert_eq!(
            donde_buscar("windows", Some(r"C:\Users\ana"), variables),
            [
                r"C:\Users\ana\AppData\Local\Apolo\apolo-mcp.exe",
                r"C:\Program Files\Apolo\apolo-mcp.exe"
            ]
        );
        assert_eq!(
            donde_buscar("linux", Some("/home/ana"), variables),
            [
                "/usr/bin/apolo-mcp",
                "/usr/local/bin/apolo-mcp",
                "/home/ana/.local/bin/apolo-mcp"
            ]
        );
    }

    #[test]
    fn cede_a_la_primera_que_existe_y_nunca_a_si_misma() {
        let sitios = vec!["/no/esta".to_string(), "/usr/bin/apolo-mcp".to_string()];
        let existe = |p: &Path| p == Path::new("/usr/bin/apolo-mcp");
        assert_eq!(
            a_quien_ceder(&sitios, Path::new("/tmp/mcpb/server/apolo-mcp"), existe),
            Some(PathBuf::from("/usr/bin/apolo-mcp"))
        );
        assert_eq!(
            a_quien_ceder(&sitios, Path::new("/usr/bin/apolo-mcp"), existe),
            None
        );
        assert_eq!(a_quien_ceder(&sitios, Path::new("/x"), |_| false), None);
    }
}
