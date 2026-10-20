//! Avisar de que Apolo se ha actualizado mientras el servidor sigue en marcha
//! (ADR 0023).
//!
//! Claude Desktop arranca el servidor al abrirse y lo mantiene vivo mientras
//! está abierto: actualizar Apolo reemplaza el fichero, pero el proceso que ya
//! corre sigue con el código de antes, y una conversación nueva reutiliza ese
//! proceso. Desde aquí no se puede arreglar (quien relanza el servidor es el
//! cliente), pero sí avisar: si el ejecutable ha cambiado en el disco y es de
//! otra versión, cada respuesta lo dice, y el agente se lo cuenta a la persona.

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// Lo que identifica al fichero: fecha de modificación y tamaño.
type Firma = (Option<SystemTime>, u64);

struct Estado {
    ruta: PathBuf,
    al_arrancar: Firma,
    /// La última firma vista y la versión que daba, para no relanzar
    /// `--version` en cada llamada.
    ultima: Mutex<Option<(Firma, Option<String>)>>,
}

static ESTADO: OnceLock<Option<Estado>> = OnceLock::new();

fn firma(ruta: &PathBuf) -> Option<Firma> {
    let m = std::fs::metadata(ruta).ok()?;
    Some((m.modified().ok(), m.len()))
}

fn estado() -> Option<&'static Estado> {
    ESTADO
        .get_or_init(|| {
            let ruta = std::env::current_exe().ok()?;
            let al_arrancar = firma(&ruta)?;
            Some(Estado {
                ruta,
                al_arrancar,
                ultima: Mutex::new(None),
            })
        })
        .as_ref()
}

/// Se llama al arrancar, para que la firma sea la del fichero que se cargó.
pub fn anotar() {
    let _ = estado();
}

/// La versión del fichero en disco, si no es la que está en marcha.
pub fn version_nueva() -> Option<String> {
    let e = estado()?;
    let ahora = firma(&e.ruta)?; // A medio actualizar puede no estar: nada que decir.
    if ahora == e.al_arrancar {
        return None;
    }
    let mut ultima = e.ultima.lock().unwrap_or_else(|x| x.into_inner());
    if let Some((f, v)) = ultima.as_ref()
        && *f == ahora
    {
        return v.clone();
    }
    // «apolo-mcp 0.8.2» por la salida de errores (main.rs).
    let v = std::process::Command::new(&e.ruta)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .output()
        .ok()
        .and_then(|s| {
            let t = String::from_utf8_lossy(&s.stderr).into_owned()
                + &String::from_utf8_lossy(&s.stdout);
            t.split_whitespace()
                .skip_while(|p| *p != "apolo-mcp")
                .nth(1)
                .map(str::to_string)
        })
        .filter(|v| v != env!("CARGO_PKG_VERSION"));
    *ultima = Some((ahora, v.clone()));
    v
}

/// La línea que se añade a las respuestas.
pub fn aviso() -> Option<String> {
    version_nueva().map(|v| {
        let esta = env!("CARGO_PKG_VERSION");
        apolo_nucleo::tr!(
            "AVISO: Apolo se ha actualizado a la {v}, pero este agente sigue con la {esta} hasta que se reinicie. Díselo a la persona: en Claude Desktop, cerrar y volver a abrir Claude Desktop (o apagar y encender la extensión en Ajustes › Extensiones); en Claude Code y otros, empezar una sesión nueva.",
            "NOTICE: Apolo has been updated to {v}, but this agent keeps running {esta} until it restarts. Tell the person: in Claude Desktop, quit and reopen Claude Desktop (or turn the extension off and on in Settings › Extensions); in Claude Code and others, start a new session."
        )
    })
}
