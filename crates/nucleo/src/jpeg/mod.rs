//! JPEG con MozJPEG: el mismo fichero que `cjpeg` 4.1.5 (ADR 0020).
//!
//! Tres piezas, como en WebP:
//! - [`EntradaJpeg`]: la imagen **como la lee cjpeg**, que no es como la lee
//!   cwebp. Conserva el gris, descarta la transparencia sin componerla, no
//!   corrige la gamma de un PNG y copia sus marcadores (el perfil ICC de un
//!   PNG; todos los APPn y COM de un JPEG);
//! - [`OpcionesJpeg`]: las opciones, que se leen y escriben como la orden de
//!   cjpeg;
//! - [`cjpeg::ejecutar`]: el `main` de cjpeg trasladado.

pub mod cjpeg;
pub mod opciones;
mod tablas;

pub use opciones::{Afinado, ColorJpeg, Dct, Escaneo, OpcionesJpeg, Reinicio};

use crate::entrada::{self, Formato, Imagen, Lectura, jpeg as ejpeg, png as epng};
use crate::{Resultado, vista};

/// El perfil sRGB mínimo que cjpeg incrusta cuando un PNG trae el trozo
/// `sRGB` (rdpng.c, `tiny_srgb`): JPEG no tiene forma de decirlo sin perfil.
/// Sacado del fuente con un script, no copiado a mano: a mano se perdieron
/// cuatro ceros y la prueba de equivalencia lo cazó. Son 536 bytes.
const SRGB_MINIMO: &[u8] = &[
    0, 0, 2, 24, 108, 99, 109, 115, 2, 16, 0, 0, 109, 110, 116, 114, 82, 71, 66, 32, 88, 89, 90,
    32, 7, 220, 0, 1, 0, 25, 0, 3, 0, 41, 0, 57, 97, 99, 115, 112, 65, 80, 80, 76, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 246, 214, 0, 1, 0, 0, 0, 0, 211,
    45, 108, 99, 109, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 100, 101, 115, 99, 0,
    0, 0, 252, 0, 0, 0, 106, 99, 112, 114, 116, 0, 0, 1, 104, 0, 0, 0, 11, 119, 116, 112, 116, 0,
    0, 1, 116, 0, 0, 0, 20, 98, 107, 112, 116, 0, 0, 1, 136, 0, 0, 0, 20, 114, 88, 89, 90, 0, 0, 1,
    156, 0, 0, 0, 20, 103, 88, 89, 90, 0, 0, 1, 176, 0, 0, 0, 20, 98, 88, 89, 90, 0, 0, 1, 196, 0,
    0, 0, 20, 114, 84, 82, 67, 0, 0, 1, 216, 0, 0, 0, 64, 98, 84, 82, 67, 0, 0, 1, 216, 0, 0, 0,
    64, 103, 84, 82, 67, 0, 0, 1, 216, 0, 0, 0, 64, 100, 101, 115, 99, 0, 0, 0, 0, 0, 0, 0, 13,
    115, 82, 71, 66, 32, 77, 111, 122, 74, 80, 69, 71, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 116, 101, 120, 116, 0, 0, 0, 0, 80, 68, 0, 0, 88, 89, 90, 32, 0, 0, 0, 0, 0, 0,
    246, 214, 0, 1, 0, 0, 0, 0, 211, 45, 88, 89, 90, 32, 0, 0, 0, 0, 0, 0, 3, 22, 0, 0, 3, 51, 0,
    0, 2, 164, 88, 89, 90, 32, 0, 0, 0, 0, 0, 0, 111, 162, 0, 0, 56, 245, 0, 0, 3, 144, 88, 89, 90,
    32, 0, 0, 0, 0, 0, 0, 98, 153, 0, 0, 183, 133, 0, 0, 24, 218, 88, 89, 90, 32, 0, 0, 0, 0, 0, 0,
    36, 160, 0, 0, 15, 132, 0, 0, 182, 207, 99, 117, 114, 118, 0, 0, 0, 0, 0, 0, 0, 26, 0, 0, 0,
    203, 1, 201, 3, 99, 5, 146, 8, 107, 11, 246, 16, 63, 21, 81, 27, 52, 33, 241, 41, 144, 50, 24,
    59, 146, 70, 5, 81, 119, 93, 237, 107, 112, 122, 5, 137, 177, 154, 124, 172, 105, 191, 125,
    211, 195, 233, 48, 255, 255,
];

/// Una imagen lista para cjpeg.
#[derive(Debug, Clone)]
pub struct EntradaJpeg {
    pub ancho: u32,
    pub alto: u32,
    /// Un canal (gris) o tres (RGB).
    pub gris: bool,
    /// Las filas seguidas, 1 o 3 bytes por píxel.
    pub filas: Vec<u8>,
    /// Marcadores que cjpeg copia a la salida: (código, datos).
    pub marcadores: Vec<(u8, Vec<u8>)>,
    /// Si cjpeg sabe leer este fichero. Si no, Apolo lo convierte igual, pero
    /// la orden cjpeg no daría este resultado.
    pub de_cjpeg: bool,
}

impl EntradaJpeg {
    /// De píxeles RGBA (una imagen procesada, o de un formato que cjpeg no
    /// lee): la transparencia se descarta, como hace cjpeg con un PNG.
    pub fn desde_rgba(ancho: u32, alto: u32, rgba: &[u8]) -> EntradaJpeg {
        EntradaJpeg {
            ancho,
            alto,
            gris: false,
            filas: entrada::quitar_alfa(rgba),
            marcadores: Vec::new(),
            de_cjpeg: false,
        }
    }
}

/// Lee un fichero como cjpeg (`select_file_type`: por el primer byte).
/// Lo que cjpeg no lee (TIFF, WebP, HEIC, QOI…) se lee con los lectores de
/// Apolo y queda marcado con `de_cjpeg: false`.
pub fn leer(datos: &[u8]) -> Resultado<EntradaJpeg> {
    match datos.first() {
        Some(0x89) => {
            let p = epng::leer_para_cjpeg(datos)?;
            let perfil = if p.srgb {
                Some(SRGB_MINIMO.to_vec())
            } else {
                p.icc
            };
            let mut marcadores = Vec::new();
            // rdpng.c lo mete en un solo APP2; si no cabe, avisa y lo deja.
            if let Some(perfil) = perfil.filter(|p| !p.is_empty() && p.len() < 65535 - 14) {
                let mut d = b"ICC_PROFILE\0\x01\x01".to_vec();
                d.extend_from_slice(&perfil);
                marcadores.push((0xE2, d));
            }
            Ok(EntradaJpeg {
                ancho: p.ancho,
                alto: p.alto,
                gris: p.gris,
                filas: p.filas,
                marcadores,
                de_cjpeg: true,
            })
        }
        Some(0xFF) => {
            let j = ejpeg::leer_para_cjpeg(datos)?;
            Ok(EntradaJpeg {
                ancho: j.ancho,
                alto: j.alto,
                gris: j.gris,
                filas: j.filas,
                marcadores: j.marcadores,
                de_cjpeg: true,
            })
        }
        _ => {
            let img = entrada::leer(
                datos,
                Lectura {
                    conservar_alfa: true,
                    metadatos: false,
                },
            )?;
            let mut e = de_imagen(&img)?;
            // rdppm.c: P2 y P5 son gris, P3 y P6 color (aunque sus píxeles
            // sean grises). Lo dice la cabecera, no el contenido.
            if img.formato == Formato::Pnm && matches!(datos.get(1), Some(b'2' | b'5')) {
                e.gris = true;
                e.filas = e.filas.iter().step_by(3).copied().collect();
            }
            Ok(e)
        }
    }
}

/// Desde una imagen ya leída por Apolo, en color. PNM, BMP y GIF los lee
/// cjpeg; el resto no.
pub fn de_imagen(img: &Imagen) -> Resultado<EntradaJpeg> {
    let (ancho, alto, rgba) = vista::rgba(img)?;
    let mut e = EntradaJpeg::desde_rgba(ancho, alto, &rgba);
    e.de_cjpeg = matches!(img.formato, Formato::Pnm | Formato::Bmp | Formato::Gif);
    Ok(e)
}
