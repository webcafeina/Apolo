//! El generador de `frontend/src/tokens.css` y de `web/tokens.css`: la
//! ventana y la web (entrega 8d) usan la misma paleta, medida en
//! `tests/contraste.rs`.

use crate::{Rgb, Tema, claro, macos_claro, macos_oscuro, oscuro, windows_claro, windows_oscuro};

/// Ruta del fichero generado, relativa a la raíz del repositorio.
pub const RUTA: &str = "frontend/src/tokens.css";

/// El mismo CSS para la web (`web/`, ADR 0024).
pub const RUTA_WEB: &str = "web/tokens.css";

/// Los dos ficheros que se generan con el mismo contenido.
pub const RUTAS: [&str; 2] = [RUTA, RUTA_WEB];

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
        ("oro-trazo", t.oro_trazo),
        ("barra", t.barra),
        ("barra-encima", t.barra_encima),
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
    for (nombre, (c, alfa)) in [("anillo", t.anillo), ("relleno-tenue", t.relleno_tenue)] {
        s += &format!(
            "{sangria}  --{nombre}: rgb({} {} {} / {alfa}%);\n",
            c.r, c.g, c.b
        );
    }
    // La flecha de un desplegable, en el color de `apagado` de la paleta (un
    // SVG no puede leer una variable CSS).
    s += &format!(
        "{sangria}  --flecha-select: url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 12 12'%3E%3Cpath d='M2.5 4.5 6 8l3.5-3.5' fill='none' stroke='%23{:02x}{:02x}{:02x}' stroke-width='1.2' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E\");\n",
        t.apagado.r, t.apagado.g, t.apagado.b
    );
    s += &format!("{sangria}}}\n");
    s
}

/// El CSS completo. El modo sigue al sistema; `data-tema` en `<html>` lo fuerza.
/// Después, la misma pareja para Windows (ADR 0027) y para macOS (ADR 0028) bajo
/// `data-sistema`: son selectores más específicos que los de siempre, así que
/// mandan en su sistema sea cual sea el modo.
pub fn css() -> String {
    let mut s = String::from(
        "/* Generado por `make tokens` desde crates/tema. No se edita a mano (ADR 0008). */\n\n",
    );
    for (sistema, claro, oscuro) in [
        ("", claro(), oscuro()),
        (
            "[data-sistema=\"windows\"]",
            windows_claro(),
            windows_oscuro(),
        ),
        ("[data-sistema=\"macos\"]", macos_claro(), macos_oscuro()),
    ] {
        if !sistema.is_empty() {
            s += "\n";
        }
        s += &bloque(&format!(":root{sistema}"), &claro, "");
        s += "\n@media (prefers-color-scheme: dark) {\n";
        s += &bloque(
            &format!(":root{sistema}:not([data-tema=\"claro\"])"),
            &oscuro,
            "  ",
        );
        s += "}\n\n";
        s += &bloque(
            &format!(":root{sistema}[data-tema=\"oscuro\"]"),
            &oscuro,
            "",
        );
    }
    s
}
