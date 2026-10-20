//! `apolo-mcp` en inglés (ADR 0025), con `APOLO_IDIOMA=en`: las
//! instrucciones, la lista de herramientas, los errores con sus pistas y las
//! respuestas. Los códigos de error y los nombres no cambian.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

use serde_json::{Value, json};

#[test]
fn en_ingles_por_las_tuberias() {
    let dir = std::env::temp_dir().join(format!("apolo-mcp-ingles-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut hijo = Command::new(env!("CARGO_BIN_EXE_apolo-mcp"))
        .env("APOLO_CONFIG", dir.join("config"))
        .env("APOLO_IDIOMA", "en")
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut entrada = hijo.stdin.take().unwrap();
    let mut lineas = BufReader::new(hijo.stdout.take().unwrap()).lines();
    let mut pedir = |id: i64, metodo: &str, params: Value| -> Value {
        writeln!(
            entrada,
            "{}",
            json!({"jsonrpc": "2.0", "id": id, "method": metodo, "params": params})
        )
        .unwrap();
        loop {
            let r: Value = serde_json::from_str(&lineas.next().unwrap().unwrap()).unwrap();
            if r["id"] == id {
                return r["result"].clone();
            }
        }
    };

    let init = pedir(
        1,
        "initialize",
        json!({"protocolVersion": "2025-06-18", "capabilities": {},
               "clientInfo": {"name": "prueba", "version": "1"}}),
    );
    let instrucciones = init["instructions"].as_str().unwrap();
    assert!(
        instrucciones.starts_with(&format!("This is Apolo {}", env!("CARGO_PKG_VERSION"))),
        "{instrucciones}"
    );
    assert!(
        instrucciones.contains("never overwrites"),
        "{instrucciones}"
    );

    let lista = pedir(2, "tools/list", json!({}));
    let herramientas = lista["tools"].as_array().unwrap();
    assert_eq!(herramientas.len(), 11);
    let optimizar = herramientas
        .iter()
        .find(|h| h["name"] == "optimizar_imagen")
        .unwrap();
    assert_eq!(optimizar["annotations"]["title"], "Optimize an image");
    assert!(
        optimizar["description"]
            .as_str()
            .unwrap()
            .starts_with("Optimizes an image"),
        "{optimizar}"
    );
    // Los argumentos se llaman igual; su descripción, en inglés.
    assert_eq!(
        optimizar["inputSchema"]["properties"]["ruta"]["description"],
        "Absolute path of the image (~ is accepted)."
    );
    let todo = lista.to_string();
    for espanol in ["«", "Ruta absoluta", "Por defecto", "píxeles"] {
        assert!(!todo.contains(espanol), "queda «{espanol}» en tools/list");
    }

    let error = pedir(
        3,
        "tools/call",
        json!({"name": "analizar_imagen", "arguments": {"ruta": dir.join("no-hay.png")}}),
    );
    assert_eq!(error["isError"], true);
    assert_eq!(error["structuredContent"]["error"], "no_existe");
    let texto = error["content"][0]["text"].as_str().unwrap();
    assert!(texto.contains("doesn't exist"), "{texto}");
    assert!(texto.contains("\nHINTS: "), "{texto}");

    let acerca = pedir(
        4,
        "tools/call",
        json!({"name": "acerca_de_apolo", "arguments": {}}),
    );
    let texto = acerca["content"][0]["text"].as_str().unwrap();
    assert!(texto.contains("\nEngines: "), "{texto}");
    assert!(texto.contains("updating doesn't touch them"), "{texto}");
    assert!(!texto.contains("Motores"), "{texto}");

    drop(entrada);
    let _ = hijo.wait();
    let _ = std::fs::remove_dir_all(&dir);
}
