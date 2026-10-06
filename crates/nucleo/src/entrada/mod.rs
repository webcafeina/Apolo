//! Leer imágenes.
//!
//! Para los formatos que `cwebp` acepta (PNG, JPEG, TIFF, WebP y PNM), cada
//! lector reproduce lo que hace el suyo en `imageio/` de libwebp 1.6.0: qué
//! transformaciones aplica, si entrega RGB o RGBA y de dónde saca los
//! metadatos. Si los píxeles de partida no son exactamente los mismos, la
//! salida tampoco lo es (ADR 0002). Lo que no es de cwebp (GIF, BMP, QOI) va
//! por el crate `image`.

mod jpeg;
mod otros;
mod png;
mod tiff;
mod webp;

use crate::error::{Error, Resultado};
use crate::metadatos::Metadatos;

/// Formato de la imagen de partida.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Formato {
    Png,
    Jpeg,
    Tiff,
    WebP,
    Pnm,
    Gif,
    Bmp,
    Qoi,
    /// YUV 4:2:0 crudo, con el tamaño dado aparte (`cwebp -s`).
    Yuv,
}

impl Formato {
    pub fn nombre(self) -> &'static str {
        match self {
            Formato::Png => "PNG",
            Formato::Jpeg => "JPEG",
            Formato::Tiff => "TIFF",
            Formato::WebP => "WebP",
            Formato::Pnm => "PNM",
            Formato::Gif => "GIF",
            Formato::Bmp => "BMP",
            Formato::Qoi => "QOI",
            Formato::Yuv => "YUV",
        }
    }

    /// Lo deduce de los primeros bytes, como `WebPGuessImageType`, más los
    /// formatos que añade Apolo.
    pub fn adivinar(datos: &[u8]) -> Option<Formato> {
        if datos.len() < 12 {
            return None;
        }
        let be32 =
            |i: usize| u32::from_be_bytes([datos[i], datos[i + 1], datos[i + 2], datos[i + 3]]);
        let (m1, m2) = (be32(0), be32(8));
        Some(match m1 {
            0x8950_4E47 => Formato::Png,
            0xFFD8_FF00..=0xFFD8_FFFF => Formato::Jpeg,
            0x4949_2A00 | 0x4D4D_002A => Formato::Tiff,
            0x5249_4646 if m2 == 0x5745_4250 => Formato::WebP,
            _ if datos[0] == b'P' && (b'5'..=b'7').contains(&datos[1]) => Formato::Pnm,
            _ if datos.starts_with(b"GIF8") => Formato::Gif,
            _ if datos.starts_with(b"BM") => Formato::Bmp,
            _ if datos.starts_with(b"qoif") => Formato::Qoi,
            _ => return None,
        })
    }
}

/// Los píxeles tal como los entrega el lector, sin pasar todavía a libwebp.
#[derive(Clone)]
pub enum Pixeles {
    /// RGB empaquetado, 3 bytes por píxel.
    Rgb(Vec<u8>),
    /// RGBA empaquetado, 4 bytes por píxel, alfa sin premultiplicar.
    Rgba(Vec<u8>),
    /// RGBA que libwebp tiene que importar como RGBX (alfa ignorado).
    Rgbx(Vec<u8>),
    /// Un WebP sin decodificar: cwebp lo decodifica directamente al espacio
    /// que vaya a usar el codificador (YUV o ARGB), y eso hay que repetirlo.
    WebP(Vec<u8>),
    /// YUV 4:2:0 crudo: planos Y, U y V seguidos.
    Yuv(Vec<u8>),
}

impl std::fmt::Debug for Pixeles {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (tipo, n) = match self {
            Pixeles::Rgb(v) => ("Rgb", v.len()),
            Pixeles::Rgba(v) => ("Rgba", v.len()),
            Pixeles::Rgbx(v) => ("Rgbx", v.len()),
            Pixeles::WebP(v) => ("WebP", v.len()),
            Pixeles::Yuv(v) => ("Yuv", v.len()),
        };
        write!(f, "{tipo}({n} bytes)")
    }
}

/// Una imagen leída.
#[derive(Debug, Clone)]
pub struct Imagen {
    pub ancho: u32,
    pub alto: u32,
    pub formato: Formato,
    pub pixeles: Pixeles,
    pub metadatos: Metadatos,
}

/// Cómo leer.
#[derive(Debug, Clone, Copy)]
pub struct Lectura {
    /// Conservar la transparencia (`cwebp` sin `-noalpha`).
    pub conservar_alfa: bool,
    /// Extraer EXIF, ICC y XMP. cwebp solo los lee si se le piden con
    /// `-metadata`, y un fallo al leerlos hace fallar la lectura entera.
    pub metadatos: bool,
}

impl Default for Lectura {
    fn default() -> Self {
        Lectura {
            conservar_alfa: true,
            metadatos: true,
        }
    }
}

/// Lee una imagen de memoria, deduciendo el formato.
pub fn leer(datos: &[u8], lectura: Lectura) -> Resultado<Imagen> {
    let formato = Formato::adivinar(datos).ok_or(Error::FormatoDesconocido)?;
    match formato {
        Formato::Png => png::leer(datos, lectura),
        Formato::Jpeg => jpeg::leer(datos, lectura),
        Formato::Tiff => tiff::leer(datos, lectura),
        Formato::WebP => webp::leer(datos, lectura),
        Formato::Pnm | Formato::Gif | Formato::Bmp | Formato::Qoi => {
            otros::leer(datos, formato, lectura)
        }
        Formato::Yuv => unreachable!("el YUV crudo no se adivina"),
    }
}

/// YUV 4:2:0 crudo de `ancho`×`alto` (`cwebp -s`).
pub fn leer_yuv(datos: &[u8], ancho: u32, alto: u32) -> Resultado<Imagen> {
    let (w, h) = (ancho as usize, alto as usize);
    let esperado = w * h + 2 * w.div_ceil(2) * h.div_ceil(2);
    if datos.len() != esperado {
        return Err(Error::lectura(
            "YUV",
            format!(
                "tiene {} bytes y para {ancho}×{alto} hacen falta {esperado}",
                datos.len()
            ),
        ));
    }
    Ok(Imagen {
        ancho,
        alto,
        formato: Formato::Yuv,
        pixeles: Pixeles::Yuv(datos.to_vec()),
        metadatos: Metadatos::default(),
    })
}

/// Quita el alfa de un RGBA empaquetado, sin mezclar: lo que hace
/// `png_set_strip_alpha`.
pub(crate) fn quitar_alfa(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect()
}

/// Gris a RGB, repitiendo el valor: `png_set_gray_to_rgb`.
pub(crate) fn gris_a_rgb(gris: &[u8]) -> Vec<u8> {
    gris.iter().flat_map(|&g| [g, g, g]).collect()
}

/// Gris con alfa a RGBA.
pub(crate) fn gris_alfa_a_rgba(ga: &[u8]) -> Vec<u8> {
    ga.as_chunks::<2>()
        .0
        .iter()
        .flat_map(|p| [p[0], p[0], p[0], p[1]])
        .collect()
}
