//! El idioma de los textos de Apolo: español o inglés (ADR 0025).
//!
//! Cada texto visible lleva sus dos versiones en el mismo sitio, con [`tr!`]
//! (con formato, da un `String`) o [`trs!`] (fijo, da un `&'static str`):
//!
//! ```
//! use apolo_nucleo::{tr, trs};
//! let ruta = "a.png";
//! let m = tr!("No se puede leer «{ruta}»", "Can't read “{ruta}”");
//! let t = trs!("Cancelado", "Cancelled");
//! ```
//!
//! El compilador exige las dos, y que usen los mismos datos: no hay dos
//! catálogos que se puedan desincronizar.
//!
//! Hay un idioma para todo el proceso. Quien arranca lo pone con [`poner`]: la
//! CLI con `--idioma` o [`del_sistema`], `apolo-mcp` con `APOLO_IDIOMA` o el del
//! sistema, la ventana con el de sus ajustes. Si nadie lo pone, español: así
//! las pruebas no dependen del `LANG` de la máquina (en CI es `C.UTF-8`). Un
//! sistema en otro idioma (francés, alemán…) usa el inglés.

use std::cell::Cell;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Idioma {
    Es,
    En,
}

impl Idioma {
    /// «es» o «en».
    pub fn codigo(self) -> &'static str {
        match self {
            Idioma::Es => "es",
            Idioma::En => "en",
        }
    }

    /// De un código de idioma o de una locale: «es», «es_ES.UTF-8», «en-US»…
    /// `None` si no es ni español ni inglés.
    pub fn de_codigo(texto: &str) -> Option<Idioma> {
        let t = texto.trim().to_ascii_lowercase();
        let base = t.split(['_', '-', '.', '@']).next().unwrap_or("");
        match base {
            "es" => Some(Idioma::Es),
            "en" => Some(Idioma::En),
            _ => None,
        }
    }
}

static ACTUAL: AtomicU8 = AtomicU8::new(1);

thread_local! {
    /// Un idioma solo para este hilo, para las pruebas (ver [`en_este_hilo`]).
    static DEL_HILO: Cell<u8> = const { Cell::new(0) };
}

fn a_byte(i: Idioma) -> u8 {
    match i {
        Idioma::Es => 1,
        Idioma::En => 2,
    }
}

/// El idioma de este proceso: el que se puso, o español.
pub fn actual() -> Idioma {
    let hilo = DEL_HILO.with(Cell::get);
    let b = if hilo != 0 {
        hilo
    } else {
        ACTUAL.load(Ordering::Relaxed)
    };
    if b == 2 { Idioma::En } else { Idioma::Es }
}

/// Hace `f` con el idioma `i` solo en este hilo: para probar los textos de un
/// idioma sin cambiar los de las pruebas que corren a la vez.
pub fn en_este_hilo<T>(i: Idioma, f: impl FnOnce() -> T) -> T {
    let antes = DEL_HILO.with(|c| c.replace(a_byte(i)));
    let r = f();
    DEL_HILO.with(|c| c.set(antes));
    r
}

/// Pone el idioma de todo el proceso.
pub fn poner(i: Idioma) {
    ACTUAL.store(a_byte(i), Ordering::Relaxed);
}

/// El del sistema: `LC_ALL`, `LC_MESSAGES` o `LANG` si están puestas (en una
/// terminal mandan ellas), y si no, el del sistema operativo. Español si es
/// español; si no, inglés.
pub fn del_sistema() -> Idioma {
    for v in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(valor) = std::env::var(v) {
            let valor = valor.trim();
            // «C» y «POSIX» no dicen ningún idioma: se mira el siguiente.
            if valor.is_empty() || valor == "C" || valor == "POSIX" || valor.starts_with("C.") {
                continue;
            }
            return Idioma::de_codigo(valor).unwrap_or(Idioma::En);
        }
    }
    sys_locale::get_locale()
        .and_then(|l| Idioma::de_codigo(&l))
        .unwrap_or(Idioma::En)
}

/// Un texto con formato en los dos idiomas: `tr!("Hola, {nombre}", "Hi, {nombre}")`.
/// Da un `String`. Los datos se escriben como en `format!`.
#[macro_export]
macro_rules! tr {
    ($es:literal, $en:literal $(,)?) => {
        match $crate::idioma::actual() {
            $crate::idioma::Idioma::Es => format!($es),
            $crate::idioma::Idioma::En => format!($en),
        }
    };
    ($es:literal, $en:literal, $($datos:tt)+) => {
        match $crate::idioma::actual() {
            $crate::idioma::Idioma::Es => format!($es, $($datos)+),
            $crate::idioma::Idioma::En => format!($en, $($datos)+),
        }
    };
}

/// Un texto fijo en los dos idiomas: `trs!("Cancelado", "Cancelled")`. Da un
/// `&'static str`.
#[macro_export]
macro_rules! trs {
    ($es:literal, $en:literal $(,)?) => {
        match $crate::idioma::actual() {
            $crate::idioma::Idioma::Es => $es,
            $crate::idioma::Idioma::En => $en,
        }
    };
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn reconoce_los_codigos() {
        for (t, i) in [
            ("es", Some(Idioma::Es)),
            ("es_ES.UTF-8", Some(Idioma::Es)),
            ("es-MX", Some(Idioma::Es)),
            ("EN", Some(Idioma::En)),
            ("en_GB.UTF-8@euro", Some(Idioma::En)),
            ("fr_FR.UTF-8", None),
            ("", None),
        ] {
            assert_eq!(Idioma::de_codigo(t), i, "{t}");
        }
    }

    #[test]
    fn las_macros_dan_el_del_idioma_puesto() {
        let n = 3;
        en_este_hilo(Idioma::En, || {
            assert_eq!(tr!("{n} fotos", "{n} photos"), "3 photos");
            assert_eq!(trs!("Cancelado", "Cancelled"), "Cancelled");
        });
        en_este_hilo(Idioma::Es, || {
            assert_eq!(tr!("{} fotos", "{} photos", n), "3 fotos");
            assert_eq!(trs!("Cancelado", "Cancelled"), "Cancelado");
        });
    }
}
