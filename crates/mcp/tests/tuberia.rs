//! El binario `apolo-mcp` de verdad, por sus tuberías (ADR 0023): lo que un
//! agente ve. Sobre todo, que en la salida estándar no hay nada más que
//! mensajes JSON-RPC aunque avifenc y cjxl, que imprimen con printf, trabajen
//! a la vez.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

#[test]
fn una_sesion_entera_por_las_tuberias() {
    let dir = std::env::temp_dir().join(format!("apolo-mcp-tuberia-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus");
    std::fs::copy(corpus.join("foto.webp"), dir.join("foto.webp")).unwrap();
    let foto = dir.join("foto.webp").display().to_string();

    let mut hijo = Command::new(env!("CARGO_BIN_EXE_apolo-mcp"))
        .env("APOLO_CONFIG", dir.join("config"))
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut entrada = hijo.stdin.take().unwrap();
    let mut enviar = |m: Value| {
        writeln!(entrada, "{m}").unwrap();
        entrada.flush().unwrap();
    };
    enviar(
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "prueba", "version": "1"}}}),
    );
    enviar(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
    enviar(json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}));
    // AVIF y JXL a la vez, con progreso: los dos codificadores escriben por
    // su cuenta mientras el protocolo responde.
    for (id, formato) in [(3, "avif"), (4, "jxl")] {
        enviar(
            json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {
            "name": "optimizar_imagen", "_meta": {"progressToken": id},
            "arguments": {"ruta": foto, "formato": formato, "calidad": 60}}}),
        );
    }
    enviar(
        json!({"jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": {
        "name": "analizar_imagen", "arguments": {"ruta": dir.join("no-hay.png")}}}),
    );
    enviar(
        json!({"jsonrpc": "2.0", "id": 6, "method": "tools/call", "params": {
        "name": "optimizar_imagen", "arguments": {"ruta": foto, "formato": "gif"}}}),
    );

    let salida = BufReader::new(hijo.stdout.take().unwrap());
    let mut respuestas = std::collections::BTreeMap::new();
    let mut progresos = 0;
    let limite = Instant::now() + Duration::from_secs(300);
    let mut lineas = salida.lines();
    while respuestas.len() < 6 {
        assert!(Instant::now() < limite, "sin respuesta a tiempo");
        let linea = lineas.next().expect("se cerró la salida").unwrap();
        let m: Value = serde_json::from_str(&linea)
            .unwrap_or_else(|e| panic!("una línea de stdout que no es JSON-RPC ({e}): {linea:?}"));
        assert_eq!(m["jsonrpc"], "2.0", "{linea}");
        if m["method"] == "notifications/progress" {
            progresos += 1;
            continue;
        }
        respuestas.insert(m["id"].as_i64().unwrap(), m);
    }

    let init = &respuestas[&1]["result"];
    assert_eq!(init["serverInfo"]["name"], "apolo");
    assert_eq!(init["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));
    assert!(
        init["instructions"]
            .as_str()
            .unwrap()
            .contains("nunca sobrescribe")
    );

    let herramientas = respuestas[&2]["result"]["tools"].as_array().unwrap();
    assert_eq!(herramientas.len(), 10);
    for h in herramientas {
        let nombre = h["name"].as_str().unwrap();
        assert!(
            nombre.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
            "{nombre}"
        );
        let a = &h["annotations"];
        for clave in [
            "title",
            "readOnlyHint",
            "destructiveHint",
            "idempotentHint",
            "openWorldHint",
        ] {
            assert!(!a[clave].is_null(), "{nombre} sin {clave}");
        }
        assert_eq!(
            a["destructiveHint"], false,
            "{nombre}: Apolo nunca destruye"
        );
        assert_eq!(h["inputSchema"]["type"], "object", "{nombre}");
    }

    for id in [3, 4] {
        let r = &respuestas[&id]["result"];
        assert_eq!(r["isError"], false, "{r}");
        let escrita = r["structuredContent"]["salida"].as_str().unwrap();
        assert!(Path::new(escrita).exists(), "{escrita}");
        // Claude Code le da al modelo structuredContent y no el texto: el
        // resumen tiene que ir también ahí.
        assert_eq!(r["structuredContent"]["resumen"], r["content"][0]["text"]);
    }
    assert!(progresos > 0, "con progressToken llegan avisos de progreso");

    let no_existe = &respuestas[&5]["result"];
    assert_eq!(no_existe["isError"], true);
    assert_eq!(no_existe["structuredContent"]["error"], "no_existe");
    assert!(
        no_existe["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("PISTAS:")
    );
    // Un argumento que no vale también es un error de la herramienta, no del
    // protocolo: el modelo lo lee y se corrige.
    assert!(
        respuestas[&6]["result"]["isError"] == true || !respuestas[&6]["error"].is_null(),
        "{}",
        respuestas[&6]
    );

    // Cerrar la entrada termina el servidor.
    drop(entrada);
    let limite = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(estado) = hijo.try_wait().unwrap() {
            assert!(estado.success(), "{estado}");
            break;
        }
        assert!(Instant::now() < limite, "no termina al cerrar la entrada");
        std::thread::sleep(Duration::from_millis(50));
    }
    // Y nada más salió por stdout.
    for linea in lineas {
        let linea = linea.unwrap();
        serde_json::from_str::<Value>(&linea).unwrap_or_else(|e| panic!("{e}: {linea:?}"));
    }
}
