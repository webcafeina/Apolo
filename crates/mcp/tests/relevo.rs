//! El relevo con los binarios de verdad (ADR 0023): la copia del `.mcpb` le
//! cede el turno a la instalada, y `acerca_de_apolo` dice quién contesta. En
//! Linux, con un HOME de prueba: la instalada va en `~/.local/bin`.
#![cfg(target_os = "linux")]

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

/// Arranca `binario` como lo haría Claude Desktop con el paquete, y devuelve
/// lo que dice `acerca_de_apolo`.
fn preguntar(binario: &Path, casa: &Path) -> Value {
    let mut hijo = Command::new(binario)
        .env("HOME", casa)
        .env("APOLO_MCP_DESDE_PAQUETE", "1")
        .env("APOLO_CONFIG", casa.join("config"))
        .env("APOLO_IDIOMA", "es")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut entrada = hijo.stdin.take().unwrap();
    for m in [
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "prueba", "version": "1"}}}),
        json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": "acerca_de_apolo", "arguments": {}}}),
    ] {
        writeln!(entrada, "{m}").unwrap();
    }
    let mut lineas = BufReader::new(hijo.stdout.take().unwrap()).lines();
    let r = loop {
        let m: Value = serde_json::from_str(&lineas.next().unwrap().unwrap()).unwrap();
        if m["id"] == 2 {
            break m["result"]["structuredContent"].clone();
        }
    };
    drop(entrada);
    assert!(hijo.wait().unwrap().success());
    r
}

#[test]
fn la_copia_del_paquete_cede_el_turno_a_la_instalada() {
    let casa = std::env::temp_dir().join(format!("apolo-mcp-relevo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&casa);
    let paquete = casa.join("extension/server/apolo-mcp");
    let instalada = casa.join(".local/bin/apolo-mcp");
    for d in [&paquete, &instalada] {
        std::fs::create_dir_all(d.parent().unwrap()).unwrap();
    }
    std::fs::copy(env!("CARGO_BIN_EXE_apolo-mcp"), &paquete).unwrap();

    // Sin Apolo instalado, contesta la copia del paquete, y lo dice.
    let sola = preguntar(&paquete, &casa);
    assert_eq!(sola["ejecutable"], "~/extension/server/apolo-mcp");
    assert_eq!(sola["copia_del_paquete_sin_aplicacion"], true);
    assert!(sola["relevado_desde"].is_null());

    // Con Apolo instalado, contesta la instalada.
    std::fs::copy(env!("CARGO_BIN_EXE_apolo-mcp"), &instalada).unwrap();
    let relevada = preguntar(&paquete, &casa);
    assert_eq!(relevada["ejecutable"], "~/.local/bin/apolo-mcp");
    assert_eq!(relevada["relevado_desde"], env!("CARGO_PKG_VERSION"));
    assert_eq!(relevada["copia_del_paquete_sin_aplicacion"], false);
    assert_eq!(relevada["version"], env!("CARGO_PKG_VERSION"));
    let _ = std::fs::remove_dir_all(&casa);
}
