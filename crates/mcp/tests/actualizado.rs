//! El aviso de actualización (actualizado.rs, ADR 0023): con el servidor en
//! marcha, se reemplaza su fichero por otra versión, como hace el
//! actualizador, y la respuesta siguiente lo dice. En Linux: el reemplazo con
//! el fichero en uso es un `rename`, como en macOS.
#![cfg(target_os = "linux")]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

#[test]
fn avisa_si_la_aplicacion_se_actualiza_con_el_servidor_en_marcha() {
    let dir = std::env::temp_dir().join(format!("apolo-mcp-actualizado-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let binario = dir.join("apolo-mcp");
    std::fs::copy(env!("CARGO_BIN_EXE_apolo-mcp"), &binario).unwrap();

    let mut hijo = Command::new(&binario)
        .env("APOLO_CONFIG", dir.join("config"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut entrada = hijo.stdin.take().unwrap();
    let mut lineas = BufReader::new(hijo.stdout.take().unwrap()).lines();
    let mut pedir = |id: i64, m: Value| -> Value {
        writeln!(entrada, "{m}").unwrap();
        loop {
            let r: Value = serde_json::from_str(&lineas.next().unwrap().unwrap()).unwrap();
            if r["id"] == id {
                return r["result"].clone();
            }
        }
    };
    pedir(
        1,
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "prueba", "version": "1"}}}),
    );
    let acerca = |id: i64| json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": "acerca_de_apolo", "arguments": {}}});

    let antes = pedir(2, acerca(2));
    assert!(
        antes["structuredContent"]["aviso_actualizacion"].is_null(),
        "{antes}"
    );

    // La «versión nueva»: un fichero que dice ser la 9.9.9, puesto encima con
    // un rename, como deja las cosas un actualizador.
    let nuevo = dir.join("nuevo");
    std::fs::write(&nuevo, "#!/bin/sh\necho 'apolo-mcp 9.9.9' >&2\n").unwrap();
    std::fs::set_permissions(&nuevo, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::rename(&nuevo, &binario).unwrap();

    let despues = pedir(3, acerca(3));
    let aviso = despues["structuredContent"]["aviso_actualizacion"]
        .as_str()
        .unwrap_or_default();
    assert!(
        aviso.contains("9.9.9") && aviso.contains(env!("CARGO_PKG_VERSION")),
        "{despues}"
    );
    assert!(aviso.contains("Claude Desktop"), "{aviso}");
    let texto = despues["content"][0]["text"].as_str().unwrap();
    assert!(
        texto.contains("AVISO: Apolo se ha actualizado a la 9.9.9"),
        "{texto}"
    );

    drop(entrada);
    let _ = hijo.wait();
    let _ = std::fs::remove_dir_all(&dir);
}
