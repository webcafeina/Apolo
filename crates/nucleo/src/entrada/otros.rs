//! PNM, GIF, BMP y QOI, con el crate `image`.
//!
//! cwebp lee PNM (P5–P7) con su propio lector; con valores máximos de 255 da
//! los mismos píxeles, con otros no está comprobado. GIF (solo el primer
//! fotograma), BMP y QOI no los lee cwebp: aquí no hay con qué compararse.

use std::io::Cursor;

use image::{DynamicImage, ImageFormat, ImageReader};

use super::{Formato, Imagen, Lectura, Pixeles, quitar_alfa};
use crate::error::{Error, Resultado};
use crate::metadatos::Metadatos;

pub(super) fn leer(datos: &[u8], formato: Formato, lectura: Lectura) -> Resultado<Imagen> {
    let f = formato.nombre();
    let tipo = match formato {
        Formato::Pnm => ImageFormat::Pnm,
        Formato::Gif => ImageFormat::Gif,
        Formato::Bmp => ImageFormat::Bmp,
        Formato::Qoi => ImageFormat::Qoi,
        _ => unreachable!(),
    };
    let mut lector = ImageReader::new(Cursor::new(datos));
    lector.set_format(tipo);
    let img = lector.decode().map_err(|e| Error::lectura(f, e))?;
    let (ancho, alto) = (img.width(), img.height());
    let tiene_alfa = img.color().has_alpha();
    let pixeles = match (tiene_alfa, lectura.conservar_alfa, img) {
        (true, true, img) => Pixeles::Rgba(img.into_rgba8().into_raw()),
        (true, false, img) => Pixeles::Rgb(quitar_alfa(&img.into_rgba8().into_raw())),
        (false, _, DynamicImage::ImageRgb8(b)) => Pixeles::Rgb(b.into_raw()),
        (false, _, img) => Pixeles::Rgb(img.into_rgb8().into_raw()),
    };
    Ok(Imagen {
        ancho,
        alto,
        formato,
        pixeles,
        metadatos: Metadatos::default(),
    })
}
