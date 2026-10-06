//! El generador de `frontend/src/tokens.css`.

use crate::{Rgb, Tema, temas};

/// Ruta del fichero generado, relativa a la raíz del repositorio.
pub const RUTA: &str = "frontend/src/tokens.css";

fn variables(t: &Tema) -> Vec<(&'static str, Rgb)> {
    vec![
        ("lienzo", t.lienzo),
        ("suave", t.suave),
        ("tarjeta", t.tarjeta),
        ("elevada", t.elevada),
        ("filete", t.filete),
        ("filete-fuerte", t.filete_fuerte),
        ("tinta", t.tinta),
        ("cuerpo", t.cuerpo),
        ("apagado", t.apagado),
        ("relleno", t.relleno),
        ("relleno-vivo", t.relleno_vivo),
        ("sobre-acento", t.sobre_acento),
        ("acento", t.acento),
        ("barra", t.barra),
        ("campo", t.campo),
        ("boton", t.boton),
        ("boton-encima", t.boton_encima),
        ("exito", t.exito),
        ("aviso", t.aviso),
        ("error", t.error),
        ("damero-a", t.damero_a),
        ("damero-b", t.damero_b),
    ]
}

fn bloque(selector: &str, t: &Tema, sangria: &str) -> String {
    let mut s = format!("{sangria}{selector} {{\n");
    s += &format!(
        "{sangria}  color-scheme: {};\n",
        if t.oscuro { "dark" } else { "light" }
    );
    for (nombre, color) in variables(t) {
        s += &format!("{sangria}  --{nombre}: {color};\n");
    }
    s += &format!("{sangria}}}\n");
    s
}

/// El CSS completo. El modo sigue al sistema; `data-tema` en `<html>` lo fuerza.
pub fn css() -> String {
    let [claro, oscuro] = temas();
    let mut s = String::from(
        "/* Generado por `make tokens` desde crates/tema. No se edita a mano (ADR 0008). */\n\n",
    );
    s += &bloque(":root", &claro, "");
    s += "\n@media (prefers-color-scheme: dark) {\n";
    s += &bloque(":root:not([data-tema=\"claro\"])", &oscuro, "  ");
    s += "}\n\n";
    s += &bloque(":root[data-tema=\"oscuro\"]", &oscuro, "");
    s
}
