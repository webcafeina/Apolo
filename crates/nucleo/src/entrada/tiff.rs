//! TIFF, como `imageio/tiffdec.c`.
//!
//! cwebp lee con `TIFFReadRGBAImageOriented`, que da siempre RGBA de 8 bits
//! orientado arriba-izquierda y **con el alfa premultiplicado**: libtiff
//! premultiplica el alfa normal («no asociado») al leerlo así. Luego cwebp
//! deshace la premultiplicación, pero solo cuando el fichero dice que el alfa
//! ya venía asociado. Resultado: un TIFF con alfa normal llega a libwebp con
//! el color oscurecido donde hay transparencia. Es una rareza de cwebp, y se
//! repite porque la promesa es el mismo fichero (docs/trampas.md).
//!
//! Aquí se cubre lo que se ve en la práctica: 8 y 16 bits por canal,
//! gris, RGB y RGBA, sin orientación rara. Lo demás pasa por el crate `tiff`
//! sin garantía de que los píxeles coincidan con los de libtiff (deuda.md).

use std::io::Cursor;

use tiff::ColorType;
use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

use super::{Formato, Imagen, Lectura, Pixeles};
use crate::error::{Error, Resultado};
use crate::metadatos::Metadatos;

const F: &str = "TIFF";
const ALFA_ASOCIADO: u16 = 1;
const ALFA_NO_ASOCIADO: u16 = 2;

pub(super) fn leer(datos: &[u8], lectura: Lectura) -> Resultado<Imagen> {
    let e = |x: tiff::TiffError| Error::lectura(F, x);
    let mut d = Decoder::new(Cursor::new(datos)).map_err(e)?;
    let (ancho, alto) = d.dimensions().map_err(e)?;
    let tipo = d.colortype().map_err(e)?;
    let extra = d
        .find_tag(Tag::ExtraSamples)
        .map_err(e)?
        .and_then(|v| v.into_u16_vec().ok())
        .unwrap_or_default();

    let metadatos = if lectura.metadatos {
        Metadatos {
            icc: d
                .find_tag(Tag::Unknown(34675))
                .map_err(e)?
                .and_then(|v| v.into_u8_vec().ok()),
            xmp: d
                .find_tag(Tag::Unknown(700))
                .map_err(e)?
                .and_then(|v| v.into_u8_vec().ok()),
            exif: None,
        }
    } else {
        Metadatos::default()
    };

    let muestras = match d.read_image().map_err(e)? {
        DecodingResult::U8(v) => v,
        // BuildMapBitdepth16To8 de libtiff.
        DecodingResult::U16(v) => v
            .into_iter()
            .map(|x| ((x as u32 + 128) / 257) as u8)
            .collect(),
        _ => return Err(Error::lectura(F, "profundidad no admitida")),
    };
    let mut rgba: Vec<u8> = match tipo {
        ColorType::Gray(_) => muestras.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        ColorType::GrayA(_) => muestras
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        ColorType::RGB(_) => muestras
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        ColorType::RGBA(_) => muestras,
        otro => {
            return Err(Error::lectura(
                F,
                format!("tipo de color no admitido: {otro:?}"),
            ));
        }
    };
    match extra.first() {
        Some(&ALFA_ASOCIADO) => desmultiplicar(&mut rgba),
        Some(&ALFA_NO_ASOCIADO) => premultiplicar(&mut rgba),
        _ => {}
    }
    // tiffdec importa con RGBA o, sin alfa, con RGBX: siempre cuatro canales.
    let pixeles = if lectura.conservar_alfa {
        Pixeles::Rgba(rgba)
    } else {
        Pixeles::Rgbx(rgba)
    };
    Ok(Imagen {
        ancho,
        alto,
        formato: Formato::Tiff,
        pixeles,
        metadatos,
    })
}

/// La tabla `UaToAa` de libtiff (tif_getimage.c).
fn premultiplicar(rgba: &mut [u8]) {
    for p in rgba.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        for c in &mut p[..3] {
            *c = ((*c as u32 * a + 127) / 255) as u8;
        }
    }
}

/// `MultARGBRow` de tiffdec: punto fijo de 24 bits.
fn desmultiplicar(rgba: &mut [u8]) {
    const MFIX: u32 = 24;
    const MITAD: u32 = (1 << MFIX) >> 1;
    for p in rgba.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        if a == 0 {
            p[0] = 0;
            p[1] = 0;
            p[2] = 0;
        } else if a < 255 {
            let escala = (255u32 << MFIX) / a;
            for c in &mut p[..3] {
                let v = ((*c as u64 * escala as u64 + MITAD as u64) >> MFIX) as u32;
                *c = v.min(255) as u8;
            }
        }
    }
}
