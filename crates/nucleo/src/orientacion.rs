//! Enderezar según la orientación EXIF (ADR 0012).
//!
//! cwebp no lo hace, así que es una opción de Apolo, apagada por defecto. Con
//! ella, los píxeles se giran o voltean **antes** de recortar y redimensionar,
//! y el EXIF que se copie a la salida lleva Orientation = 1, para que nadie la
//! vuelva a girar al abrirla.

use crate::entrada::{Imagen, Pixeles};
use crate::error::Resultado;
use crate::vista;

const ORIENTACION: u16 = 0x0112;
const CORTO: u16 = 3;

/// Dónde está el valor de la etiqueta Orientation dentro del EXIF, y si el
/// TIFF es big-endian. Admite el EXIF crudo (empieza por `II*\0` o `MM\0*`)
/// y el que trae delante la firma `Exif\0\0`.
fn localizar(exif: &[u8]) -> Option<(usize, bool)> {
    let base = if exif.starts_with(b"Exif\0\0") { 6 } else { 0 };
    let t = exif.get(base..)?;
    let be = match t.get(..4)? {
        b"MM\0*" => true,
        b"II*\0" => false,
        _ => return None,
    };
    let u16_ = |i: usize| -> Option<u16> {
        let b = t.get(i..i + 2)?;
        Some(if be {
            u16::from_be_bytes([b[0], b[1]])
        } else {
            u16::from_le_bytes([b[0], b[1]])
        })
    };
    let u32_ = |i: usize| -> Option<u32> {
        let b = t.get(i..i + 4)?;
        let a = [b[0], b[1], b[2], b[3]];
        Some(if be {
            u32::from_be_bytes(a)
        } else {
            u32::from_le_bytes(a)
        })
    };
    let ifd0 = u32_(4)? as usize;
    let entradas = u16_(ifd0)? as usize;
    for k in 0..entradas {
        let e = ifd0 + 2 + 12 * k;
        if u16_(e)? == ORIENTACION && u16_(e + 2)? == CORTO && u32_(e + 4)? >= 1 {
            return Some((base + e + 8, be));
        }
    }
    None
}

/// La orientación EXIF (1–8), o 1 si no hay o no se entiende.
pub fn leer(exif: Option<&[u8]>) -> u8 {
    let Some(exif) = exif else { return 1 };
    let Some((i, be)) = localizar(exif) else {
        return 1;
    };
    let v = if be {
        u16::from_be_bytes([exif[i], exif[i + 1]])
    } else {
        u16::from_le_bytes([exif[i], exif[i + 1]])
    };
    if (1..=8).contains(&v) { v as u8 } else { 1 }
}

/// El mismo EXIF con Orientation = 1.
pub fn normalizar(exif: &[u8]) -> Vec<u8> {
    let mut v = exif.to_vec();
    if let Some((i, be)) = localizar(exif) {
        let uno = if be {
            1u16.to_be_bytes()
        } else {
            1u16.to_le_bytes()
        };
        v[i..i + 2].copy_from_slice(&uno);
    }
    v
}

/// Aplica la orientación a un búfer de `canales` bytes por píxel.
/// Devuelve el nuevo ancho, alto y búfer.
pub fn transformar(
    ancho: u32,
    alto: u32,
    canales: usize,
    datos: &[u8],
    orientacion: u8,
) -> (u32, u32, Vec<u8>) {
    let (w, h) = (ancho as usize, alto as usize);
    if !(2..=8).contains(&orientacion) {
        return (ancho, alto, datos.to_vec());
    }
    let girada = orientacion >= 5;
    let (nw, nh) = if girada { (h, w) } else { (w, h) };
    let mut s = vec![0u8; datos.len()];
    for y in 0..nh {
        for x in 0..nw {
            // De dónde sale el píxel (x, y) de la imagen derecha.
            let (sx, sy) = match orientacion {
                2 => (w - 1 - x, y),
                3 => (w - 1 - x, h - 1 - y),
                4 => (x, h - 1 - y),
                5 => (y, x),
                6 => (y, h - 1 - x),
                7 => (w - 1 - y, h - 1 - x),
                _ => (w - 1 - y, x), // 8
            };
            let o = (sy * w + sx) * canales;
            let d = (y * nw + x) * canales;
            s[d..d + canales].copy_from_slice(&datos[o..o + canales]);
        }
    }
    (nw as u32, nh as u32, s)
}

/// Cómo volver a meter los píxeles girados en su variante de `Pixeles`.
type Envolver = fn(Vec<u8>) -> Pixeles;

/// La imagen enderezada, con el EXIF ya a 1. Si no hay nada que hacer, `None`.
pub fn enderezar(img: &Imagen) -> Resultado<Option<Imagen>> {
    let o = leer(img.metadatos.exif.as_deref());
    if o == 1 {
        return Ok(None);
    }
    let (canales, datos, envolver): (usize, std::borrow::Cow<[u8]>, Envolver) = match &img.pixeles {
        Pixeles::Rgb(v) => (3, v.into(), Pixeles::Rgb),
        Pixeles::Rgba(v) => (4, v.into(), Pixeles::Rgba),
        Pixeles::Rgbx(v) => (4, v.into(), Pixeles::Rgbx),
        // WebP y YUV no se pueden girar sin decodificar: se pasan a RGBA. Con
        // esta opción ya no hay promesa de fichero idéntico a cwebp.
        Pixeles::WebP(_) | Pixeles::Yuv(_) => {
            let (_, _, rgba) = vista::rgba(img)?;
            (4, rgba.into(), Pixeles::Rgba)
        }
    };
    let (ancho, alto, girados) = transformar(img.ancho, img.alto, canales, &datos, o);
    let mut metadatos = img.metadatos.clone();
    metadatos.exif = metadatos.exif.as_deref().map(normalizar);
    Ok(Some(Imagen {
        ancho,
        alto,
        formato: img.formato,
        pixeles: envolver(girados),
        metadatos,
    }))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Un EXIF mínimo con una sola entrada: Orientation.
    pub(crate) fn exif(orientacion: u16, be: bool) -> Vec<u8> {
        let mut v = Vec::new();
        let p16 =
            |v: &mut Vec<u8>, x: u16| v.extend(if be { x.to_be_bytes() } else { x.to_le_bytes() });
        let p32 =
            |v: &mut Vec<u8>, x: u32| v.extend(if be { x.to_be_bytes() } else { x.to_le_bytes() });
        v.extend(if be { b"MM\0*" } else { b"II*\0" });
        p32(&mut v, 8);
        p16(&mut v, 1);
        p16(&mut v, ORIENTACION);
        p16(&mut v, CORTO);
        p32(&mut v, 1);
        p16(&mut v, orientacion);
        p16(&mut v, 0);
        p32(&mut v, 0);
        v
    }

    #[test]
    fn lee_y_normaliza() {
        for be in [true, false] {
            for o in 1..=8 {
                let e = exif(o, be);
                assert_eq!(leer(Some(&e)), o as u8);
                assert_eq!(leer(Some(&normalizar(&e))), 1);
                let mut con_firma = b"Exif\0\0".to_vec();
                con_firma.extend(&e);
                assert_eq!(leer(Some(&con_firma)), o as u8);
            }
        }
        assert_eq!(leer(None), 1);
        assert_eq!(leer(Some(b"basura")), 1);
    }

    /// Una imagen de 3×2 con un valor distinto por píxel:
    /// ```text
    /// 1 2 3
    /// 4 5 6
    /// ```
    /// Para cada orientación, lo que guarda la cámara según la definición de
    /// EXIF (p. ej. 6: «la fila 0 es el lado derecho visual; la columna 0, el
    /// borde de arriba»), y al enderezar tiene que salir siempre la de arriba.
    #[test]
    fn las_ocho_orientaciones() {
        let derecha = [1u8, 2, 3, 4, 5, 6];
        // Cómo guarda la cámara la imagen derecha para cada orientación.
        let guardada: [(u8, u32, u32, &[u8]); 8] = [
            (1, 3, 2, &[1, 2, 3, 4, 5, 6]),
            (2, 3, 2, &[3, 2, 1, 6, 5, 4]),
            (3, 3, 2, &[6, 5, 4, 3, 2, 1]),
            (4, 3, 2, &[4, 5, 6, 1, 2, 3]),
            (5, 2, 3, &[1, 4, 2, 5, 3, 6]),
            (6, 2, 3, &[3, 6, 2, 5, 1, 4]),
            (7, 2, 3, &[6, 3, 5, 2, 4, 1]),
            (8, 2, 3, &[4, 1, 5, 2, 6, 3]),
        ];
        for (o, w, h, datos) in guardada {
            let (nw, nh, s) = transformar(w, h, 1, datos, o);
            assert_eq!((nw, nh), (3, 2), "orientación {o}");
            assert_eq!(s, derecha, "orientación {o}");
        }
    }
}
