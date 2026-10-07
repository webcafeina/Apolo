//! AVIF y JPEG XL: los decodifican avifdec y djxl, compilados dentro de Apolo
//! (ADR 0021), a un PNG con su perfil y sus metadatos, que se lee como
//! cualquier PNG. cwebp no lee ninguno de los dos.

use super::{Formato, Imagen, Lectura};
use crate::error::{Error, Resultado};

pub(super) fn leer(datos: &[u8], formato: Formato, lectura: Lectura) -> Resultado<Imagen> {
    let png = match formato {
        Formato::Avif => apolo_avifjxl::avif_a_png(datos, false),
        _ => apolo_avifjxl::jxl_a_png(datos, false),
    }
    .map_err(|e| Error::lectura(formato.nombre(), e))?;
    let mut img = super::png::leer(&png, lectura)?;
    img.formato = formato;
    Ok(img)
}
