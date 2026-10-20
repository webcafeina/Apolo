use std::fmt;

use crate::{tr, trs};

// El texto de cada error, en el idioma del proceso (ADR 0025): por eso el
// `Display` está escrito a mano y no con `#[error]` de thiserror, que no
// admite macros.
#[derive(Debug)]
pub enum Error {
    FormatoDesconocido,

    Lectura {
        formato: &'static str,
        detalle: String,
    },

    Configuracion(String),

    Codificacion(String),

    Cancelado,

    Memoria,

    /// Leer o escribir un fichero, con la ruta ya en el texto.
    Fichero(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::FormatoDesconocido => f.write_str(trs!(
                "No se reconoce el formato de la imagen",
                "Unrecognized image format"
            )),
            Error::Lectura { formato, detalle } => f.write_str(&tr!(
                "No se pudo leer la imagen {formato}: {detalle}",
                "Couldn't read the {formato} image: {detalle}"
            )),
            Error::Configuracion(d) => f.write_str(&tr!(
                "La configuración no es válida: {d}",
                "Invalid configuration: {d}"
            )),
            Error::Codificacion(d) => {
                f.write_str(&tr!("No se pudo codificar: {d}", "Couldn't encode: {d}"))
            }
            Error::Cancelado => f.write_str(trs!("Cancelado", "Cancelled")),
            Error::Memoria => f.write_str(trs!("Falta memoria", "Out of memory")),
            Error::Fichero(d) => f.write_str(d),
        }
    }
}

impl std::error::Error for Error {}

pub type Resultado<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn lectura(formato: &'static str, detalle: impl ToString) -> Self {
        Error::Lectura {
            formato,
            detalle: detalle.to_string(),
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::idioma::{Idioma, en_este_hilo};

    #[test]
    fn los_errores_en_los_dos_idiomas() {
        let lectura = || Error::lectura("PNG", "x").to_string();
        assert_eq!(
            Error::FormatoDesconocido.to_string(),
            "No se reconoce el formato de la imagen"
        );
        assert_eq!(lectura(), "No se pudo leer la imagen PNG: x");
        en_este_hilo(Idioma::En, || {
            assert_eq!(
                Error::FormatoDesconocido.to_string(),
                "Unrecognized image format"
            );
            assert_eq!(lectura(), "Couldn't read the PNG image: x");
            assert_eq!(Error::Cancelado.to_string(), "Cancelled");
        });
    }
}
