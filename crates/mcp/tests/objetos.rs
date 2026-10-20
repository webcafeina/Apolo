//! Los argumentos que son objetos (`recorte`, `zona`), como los manda cada
//! cliente (prueba guiada de la v0.9.0): Claude Desktop los mandaba como texto
//! con el JSON dentro, y Apolo los rechazaba. Ahora el esquema no usa `$ref` y
//! se aceptan de las dos formas.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{Value, json};

#[test]
fn el_recorte_vale_como_objeto_y_como_texto() {
    let dir = std::env::temp_dir().join(format!("apolo-mcp-objetos-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus");
    std::fs::copy(corpus.join("foto.webp"), dir.join("foto.webp")).unwrap();
    let foto = dir.join("foto.webp").display().to_string();

    let mut hijo = Command::new(env!("CARGO_BIN_EXE_apolo-mcp"))
        .env("APOLO_CONFIG", dir.join("config"))
        .env("APOLO_IDIOMA", "es")
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

    // El esquema: un objeto escrito en el sitio, sin $ref.
    let lista = pedir(
        2,
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    );
    for (herramienta, campo) in [
        ("optimizar_imagen", "recorte"),
        ("ver_imagen", "zona"),
        ("empezar_lote", "recorte"),
    ] {
        let h = lista["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["name"] == herramienta)
            .unwrap();
        let p = &h["inputSchema"]["properties"][campo];
        assert_eq!(p["type"], "object", "{herramienta}.{campo}: {p}");
        assert!(
            !p.to_string().contains("$ref"),
            "{herramienta}.{campo}: {p}"
        );
    }

    // Como objeto y como texto con el JSON dentro: los dos recortan.
    let rect = json!({"x": 0, "y": 0, "ancho": 64, "alto": 32});
    for (id, recorte) in [(3, rect.clone()), (4, Value::String(rect.to_string()))] {
        let r = pedir(
            id,
            json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {
            "name": "optimizar_imagen", "arguments": {"ruta": foto, "formato": "webp", "calidad": 70, "recorte": recorte}}}),
        );
        assert_eq!(r["isError"], false, "{r}");
        assert_eq!(
            (
                r["structuredContent"]["ancho"].as_u64(),
                r["structuredContent"]["alto"].as_u64()
            ),
            (Some(64), Some(32)),
            "{r}"
        );
    }
    // Y la zona de ver_imagen, como texto.
    let v = pedir(
        5,
        json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {
        "name": "ver_imagen", "arguments": {"ruta": foto, "zona": "{\"x\":10,\"y\":10,\"ancho\":40,\"alto\":20}"}}}),
    );
    assert_eq!(v["structuredContent"]["ancho"], 40, "{v}");

    drop(entrada);
    let _ = hijo.wait();
    let _ = std::fs::remove_dir_all(&dir);
}
