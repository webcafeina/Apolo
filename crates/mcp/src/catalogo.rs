//! `docs/mcp/catalogo.md`: las herramientas tal como las ve un agente, sacadas
//! de las mismas definiciones que da `tools/list`. Se regenera con
//! `make catalogo-mcp`, y una prueba falla si el fichero se queda atrás.

use serde_json::{Map, Value};

use crate::{INSTRUCCIONES, ServidorApolo};

/// Junta las líneas de un comentario de documentación en un párrafo.
fn parrafo(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// El tipo de un argumento, para leerlo: «texto», «webp | avif…», «lista de…».
fn tipo(p: &Value, defs: &Map<String, Value>) -> String {
    if let Some(r) = p.get("$ref").and_then(Value::as_str) {
        let nombre = r.rsplit('/').next().unwrap_or(r);
        return defs
            .get(nombre)
            .map(|d| tipo(d, defs))
            .unwrap_or_else(|| nombre.into());
    }
    if let Some(v) = p.get("anyOf").and_then(Value::as_array) {
        let t: Vec<String> = v
            .iter()
            .filter(|x| x.get("type") != Some(&Value::from("null")))
            .map(|x| tipo(x, defs))
            .collect();
        return t.join(" o ");
    }
    if let Some(v) = p.get("enum").and_then(Value::as_array) {
        return v
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(" \\| ");
    }
    if let Some(v) = p.get("oneOf").and_then(Value::as_array) {
        return v
            .iter()
            .filter_map(|x| x.get("const").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join(" \\| ");
    }
    let t = match p.get("type") {
        Some(Value::Array(v)) => v
            .iter()
            .filter_map(Value::as_str)
            .find(|t| *t != "null")
            .unwrap_or("?"),
        Some(Value::String(s)) => s,
        _ => "?",
    };
    match t {
        "string" => "texto".into(),
        "integer" => "entero".into(),
        "number" => "número".into(),
        "boolean" => "sí/no".into(),
        "array" => format!(
            "lista de {}",
            p.get("items").map(|i| tipo(i, defs)).unwrap_or_default()
        ),
        "object" => {
            let campos: Vec<&str> = p
                .get("properties")
                .and_then(Value::as_object)
                .map(|o| o.keys().map(String::as_str).collect())
                .unwrap_or_default();
            format!("{{{}}}", campos.join(", "))
        }
        otro => otro.into(),
    }
}

fn anotaciones(a: &Value) -> String {
    let mut v = Vec::new();
    if a["readOnlyHint"] == true {
        v.push("solo lee");
    } else {
        v.push("hace cambios");
    }
    if a["destructiveHint"] == false {
        v.push("no destruye");
    }
    if a["idempotentHint"] == true {
        v.push("idempotente");
    }
    if a["openWorldHint"] == false {
        v.push("todo en este ordenador");
    }
    v.join(" · ")
}

/// El catálogo entero, en Markdown.
pub fn catalogo() -> String {
    let mut s = String::from(
        "# Las herramientas de apolo-mcp\n\n\
         > Generado desde el código (`crates/mcp`) con `make catalogo-mcp`. No se edita a mano: una\n\
         > prueba falla si se queda atrás. Las decisiones, en el [ADR 0023](../adr/0023-mcp.md).\n\n\
         ## Lo que lee el agente al conectarse\n\n",
    );
    s.push_str(INSTRUCCIONES);
    s.push('\n');
    for h in ServidorApolo::herramientas() {
        let h = serde_json::to_value(&h).unwrap_or_default();
        let nombre = h["name"].as_str().unwrap_or_default();
        let a = &h["annotations"];
        s.push_str(&format!(
            "\n## `{nombre}` — {}\n\n*{}*\n\n{}\n",
            a["title"].as_str().unwrap_or_default(),
            anotaciones(a),
            parrafo(h["description"].as_str().unwrap_or_default())
        ));
        let esquema = &h["inputSchema"];
        let defs = esquema
            .get("$defs")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let obligatorios: Vec<&str> = esquema["required"]
            .as_array()
            .map(|v| v.iter().filter_map(Value::as_str).collect())
            .unwrap_or_default();
        let Some(props) = esquema["properties"].as_object().filter(|p| !p.is_empty()) else {
            s.push_str("\nSin argumentos.\n");
            continue;
        };
        s.push_str("\n| Argumento | Tipo | Qué es |\n|---|---|---|\n");
        // Primero los obligatorios, luego por orden alfabético.
        let mut claves: Vec<&String> = props.keys().collect();
        claves.sort_by_key(|k| (!obligatorios.contains(&k.as_str()), k.as_str()));
        for k in claves {
            let p = &props[k];
            let mut t = tipo(p, &defs);
            if let (Some(min), Some(max)) = (p.get("minimum"), p.get("maximum")) {
                t.push_str(&format!(" ({min}–{max})"));
            }
            let mut que = parrafo(
                p.get("description")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
            );
            if obligatorios.contains(&k.as_str()) {
                que = format!("**Obligatorio.** {que}");
            }
            s.push_str(&format!(
                "| `{k}` | {t} | {} |\n",
                que.trim().replace('|', "\\|")
            ));
        }
    }
    s
}
