//! Los errores de las herramientas (ADR 0023), como en Cronos: un código
//! estable para la máquina, un mensaje en español y una pista de qué
//! herramienta o argumento lo arregla.

use apolo_nucleo::Error;
use apolo_servicio::Fallo;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ErrorMcp {
    /// El código estable: `no_existe`, `nota_no_admitida`, `cancelado`…
    pub error: &'static str,
    pub mensaje: String,
    /// Qué herramienta o argumento lo arregla.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pista: Option<String>,
}

pub type Hecho<T> = Result<T, ErrorMcp>;

impl ErrorMcp {
    pub fn nuevo(error: &'static str, mensaje: impl Into<String>) -> Self {
        ErrorMcp {
            error,
            mensaje: mensaje.into(),
            pista: None,
        }
    }

    pub fn con_pista(mut self, pista: impl Into<String>) -> Self {
        self.pista = Some(pista.into());
        self
    }

    /// El texto para el modelo: el mensaje y, debajo, las pistas.
    pub fn texto(&self) -> String {
        match &self.pista {
            Some(p) => format!("{}\nPISTAS: {p}", self.mensaje),
            None => self.mensaje.clone(),
        }
    }

    pub fn cancelado() -> Self {
        ErrorMcp::nuevo("cancelado", "Cancelado: no se ha escrito nada.")
    }
}

pub const FORMATOS_QUE_LEE: &str =
    "PNG, JPEG, HEIC, AVIF, JPEG XL, WebP, TIFF, GIF, BMP, QOI y PNM";

impl From<Error> for ErrorMcp {
    fn from(e: Error) -> Self {
        match e {
            Error::Cancelado => ErrorMcp::cancelado(),
            Error::FormatoDesconocido => ErrorMcp::nuevo("no_es_imagen", e.to_string())
                .con_pista(format!("Apolo lee {FORMATOS_QUE_LEE}.")),
            Error::Lectura { .. } => ErrorMcp::nuevo("no_se_lee", e.to_string())
                .con_pista("El fichero puede estar dañado; prueba con analizar_imagen."),
            Error::Configuracion(_) => ErrorMcp::nuevo("configuracion", e.to_string()),
            Error::Codificacion(_) => ErrorMcp::nuevo("codificacion", e.to_string()),
            Error::Memoria => ErrorMcp::nuevo("memoria", e.to_string())
                .con_pista("Prueba con una imagen más pequeña, o reduce con ancho o alto."),
            Error::Fichero(_) => ErrorMcp::nuevo("fichero", e.to_string()),
        }
    }
}

impl From<Fallo> for ErrorMcp {
    fn from(f: Fallo) -> Self {
        if f.cancelado {
            ErrorMcp::cancelado()
        } else {
            ErrorMcp::nuevo("fallo", f.mensaje)
        }
    }
}

impl From<std::io::Error> for ErrorMcp {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => ErrorMcp::nuevo("sin_permiso", e.to_string())
                .con_pista("Elige otra carpeta con el argumento salida."),
            _ => ErrorMcp::nuevo("fichero", e.to_string()),
        }
    }
}
