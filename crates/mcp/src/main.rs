//! apolo-mcp: el servidor MCP de Apolo por la entrada y la salida estándar
//! (ADR 0023). Lo arranca el agente (Claude Desktop, Claude Code, Cursor…),
//! no la persona.

// Sin consola en Windows: el agente lo arranca con tuberías, y una ventana
// negra que se abre y se cierra en cada sesión no ayuda (como Esfinge).
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::process::ExitCode;

use apolo_mcp::ServidorApolo;
use rmcp::ServiceExt;

fn main() -> ExitCode {
    // Lo primero de todo: la salida estándar es del protocolo. Se guarda una
    // copia para él y el fd 1 pasa a la salida de errores, así lo que escriba
    // cualquier biblioteca (avifenc imprime con printf) no rompe los mensajes.
    let salida = match apolo_avifjxl::apartar_salida() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("apolo-mcp: no se pudo preparar la salida: {e}");
            return ExitCode::FAILURE;
        }
    };
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        eprintln!("apolo-mcp {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    // La copia del .mcpb le cede el turno a la aplicación instalada, que es la
    // que se actualiza (relevo.rs). Si no hay o no arranca, sigue esta.
    if let Some(otro) = apolo_mcp::relevo::ceder_si_toca()
        && let Ok(copia) = salida.try_clone()
        && let Some(codigo) = apolo_mcp::relevo::relevar(&otro, copia)
    {
        return ExitCode::from(codigo.clamp(0, 255) as u8);
    }
    // Qué fichero se ha cargado, para avisar si la aplicación lo reemplaza
    // mientras sigue en marcha (actualizado.rs).
    apolo_mcp::actualizado::anotar();
    if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        eprintln!(
            "apolo-mcp {}: este programa lo arranca un agente de IA (Claude, Cursor…), no se usa a mano.\nCómo añadirlo: Apolo › Ajustes › Agentes. Esperando mensajes MCP por la entrada estándar…",
            env!("CARGO_PKG_VERSION")
        );
    }
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("apolo-mcp: {e}");
            return ExitCode::FAILURE;
        }
    };
    rt.block_on(async move {
        let transporte = (tokio::io::stdin(), tokio::fs::File::from_std(salida));
        match ServidorApolo::nuevo().serve(transporte).await {
            Ok(servicio) => {
                let _ = servicio.waiting().await;
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("apolo-mcp: {e}");
                ExitCode::FAILURE
            }
        }
    })
}
