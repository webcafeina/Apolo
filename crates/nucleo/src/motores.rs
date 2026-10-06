//! Qué motores lleva dentro esta compilación, y en qué versión.
//!
//! Lo enseñan «Acerca de» y `apolo --version`. La versión de libwebp no es un
//! detalle: es la que decide los bytes de salida (ADR 0002).

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Motor {
    pub nombre: &'static str,
    pub version: String,
}

/// Los motores enlazados, en el orden en que se enseñan.
pub fn motores() -> Vec<Motor> {
    vec![Motor {
        nombre: "libwebp",
        version: version_libwebp(),
    }]
}

/// La versión del codificador de libwebp, como «1.5.0».
///
/// libwebp la da empaquetada en un entero: 0xMMmmpp.
pub fn version_libwebp() -> String {
    // SAFETY: función pura de libwebp, sin argumentos ni estado.
    let v = unsafe { libwebp_sys::WebPGetEncoderVersion() };
    desempaquetar(v)
}

fn desempaquetar(v: i32) -> String {
    format!("{}.{}.{}", (v >> 16) & 0xff, (v >> 8) & 0xff, v & 0xff)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn desempaqueta_la_version() {
        assert_eq!(desempaquetar(0x010500), "1.5.0");
        assert_eq!(desempaquetar(0x000603), "0.6.3");
    }

    #[test]
    fn libwebp_esta_enlazada() {
        let v = version_libwebp();
        assert!(v.starts_with("1."), "libwebp debería ser 1.x y es {v}");
    }
}
