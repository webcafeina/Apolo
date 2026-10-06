use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No se reconoce el formato de la imagen")]
    FormatoDesconocido,

    #[error("No se pudo leer la imagen {formato}: {detalle}")]
    Lectura {
        formato: &'static str,
        detalle: String,
    },

    #[error("La configuración no es válida: {0}")]
    Configuracion(String),

    #[error("No se pudo codificar: {0}")]
    Codificacion(String),

    #[error("Cancelado")]
    Cancelado,

    #[error("Falta memoria")]
    Memoria,
}

pub type Resultado<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn lectura(formato: &'static str, detalle: impl ToString) -> Self {
        Error::Lectura {
            formato,
            detalle: detalle.to_string(),
        }
    }
}
