//! Lo que devuelve una herramienta (ADR 0023), como en Cronos: un texto
//! compacto para el modelo, una línea por cosa, y los mismos datos en JSON
//! (`structuredContent`). A veces, una imagen.

use serde::Serialize;

/// Más de esto en el texto no sirve: se corta y se dice.
pub const LIMITE_TEXTO: usize = 20_000;

#[derive(Debug, Clone)]
pub struct Respuesta {
    pub texto: String,
    pub datos: serde_json::Value,
    pub imagen: Option<Imagen>,
}

#[derive(Debug, Clone)]
pub struct Imagen {
    pub mime: &'static str,
    pub base64: String,
}

impl Respuesta {
    pub fn nueva(texto: impl Into<String>, datos: &impl Serialize) -> Respuesta {
        let mut datos = serde_json::to_value(datos).unwrap_or(serde_json::Value::Null);
        let mut texto: String = texto.into();
        if texto.chars().count() > LIMITE_TEXTO {
            texto = texto.chars().take(LIMITE_TEXTO).collect();
            texto.push_str("\n… (recortada: pide menos, o usa el cursor desde)");
            if let serde_json::Value::Object(m) = &mut datos {
                m.insert("recortada".into(), true.into());
            }
        }
        Respuesta {
            texto,
            datos,
            imagen: None,
        }
    }
}

/// Un peso legible: «812 B», «48 KB», «1,2 MB».
pub fn peso(n: u64) -> String {
    let coma = |s: String| s.replace('.', ",");
    if n < 1024 {
        format!("{n} B")
    } else if n < 1024 * 1024 {
        coma(format!("{:.0} KB", n as f64 / 1024.0))
    } else {
        coma(format!("{:.1} MB", n as f64 / 1024.0 / 1024.0))
    }
}

/// El ahorro en porcentaje, con signo: «−61 %» o «+4 %».
pub fn ahorro(antes: u64, despues: u64) -> String {
    let p = ahorro_pct(antes, despues);
    if p >= 0.0 {
        format!("−{:.0} %", p)
    } else {
        format!("+{:.0} %", -p)
    }
}

/// Cuánto se ahorra, en tanto por ciento (negativo si crece).
pub fn ahorro_pct(antes: u64, despues: u64) -> f64 {
    if antes == 0 {
        return 0.0;
    }
    (1.0 - despues as f64 / antes as f64) * 100.0
}

/// Un número con coma decimal.
pub fn coma(n: f64, decimales: usize) -> String {
    format!("{n:.decimales$}").replace('.', ",")
}

/// Lo que significa una nota SSIMULACRA 2 (la escala de su autor).
pub fn banda(nota: f64) -> &'static str {
    if nota >= 90.0 {
        "no se distingue del original"
    } else if nota >= 80.0 {
        "muy alta: no se distingue lado a lado"
    } else if nota >= 70.0 {
        "alta"
    } else if nota >= 50.0 {
        "media: se nota comparando"
    } else if nota >= 30.0 {
        "baja"
    } else {
        "muy baja"
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn pesos_y_ahorros() {
        assert_eq!(peso(812), "812 B");
        assert_eq!(peso(48 * 1024), "48 KB");
        assert_eq!(peso(1_258_291), "1,2 MB");
        assert_eq!(ahorro(1000, 390), "−61 %");
        assert_eq!(ahorro(1000, 1040), "+4 %");
    }

    #[test]
    fn el_texto_largo_se_recorta_y_lo_dice() {
        let r = Respuesta::nueva("x".repeat(LIMITE_TEXTO + 10), &serde_json::json!({}));
        assert!(r.texto.contains("recortada"));
        assert_eq!(r.datos["recortada"], true);
    }
}
