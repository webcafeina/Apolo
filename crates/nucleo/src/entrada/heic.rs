//! HEIC, las fotos del iPhone, con libheif (crates/heic).
//!
//! cwebp no lee HEIC, así que aquí no hay con qué compararse: la orden cwebp
//! que enseña la interfaz sirve de referencia, pero no se puede ejecutar tal
//! cual (servicio: `Motivo::FormatoSinCwebp`).
//!
//! libheif entrega los píxeles **ya derechos**: aplica al decodificar los giros
//! que guarda el propio HEIF. El EXIF del iPhone dice además «orientación 6», y
//! si se dejara, enderezar (ADR 0012) giraría la foto otra vez. Por eso aquí se
//! deja en 1: lo que dice el EXIF ya está hecho.

use super::{Formato, Imagen, Lectura, Pixeles, quitar_alfa};
use crate::error::{Error, Resultado};
use crate::metadatos::Metadatos;
use crate::orientacion;

pub(super) fn leer(datos: &[u8], lectura: Lectura) -> Resultado<Imagen> {
    let h = apolo_heic::leer(datos).map_err(|e| Error::lectura("HEIC", e))?;
    let pixeles = match (h.alfa, lectura.conservar_alfa) {
        (true, true) => Pixeles::Rgba(h.pixeles),
        (true, false) => Pixeles::Rgb(quitar_alfa(&h.pixeles)),
        (false, _) => Pixeles::Rgb(h.pixeles),
    };
    let metadatos = if lectura.metadatos {
        Metadatos {
            exif: h.exif.as_deref().map(orientacion::normalizar),
            icc: h.icc,
            xmp: h.xmp,
        }
    } else {
        Metadatos::default()
    };
    Ok(Imagen {
        ancho: h.ancho,
        alto: h.alto,
        formato: Formato::Heic,
        pixeles,
        metadatos,
    })
}
