//! QOI: el mismo fichero que `qoiconv`, la herramienta del autor del formato
//! (ADR 0020). QOI no tiene opciones: la codificación es una sola.
//!
//! `qoiconv` solo lee PNG, con stb_image, y elige los canales con
//! `stbi_info`: 3 si el PNG es RGB (aunque traiga `tRNS`, que entonces se
//! pierde) o de paleta sin transparencia; 4 en cualquier otro caso, el gris
//! incluido. stb_image no corrige la gamma y de los 16 bits se queda con el
//! byte alto. El espacio de color se escribe como sRGB.
//!
//! De otros formatos, Apolo usa 4 canales si hay transparencia y 3 si no;
//! eso ya no lo da ninguna orden de qoiconv.

use std::io::Cursor;

use png::{ColorType, Transformations};

use crate::{Error, Resultado};

/// Píxeles listos para QOI: 3 o 4 bytes por píxel.
pub struct EntradaQoi {
    pub ancho: u32,
    pub alto: u32,
    pub pixeles: Vec<u8>,
    pub canales: u8,
    /// Si qoiconv lee este fichero (si es un PNG).
    pub de_qoiconv: bool,
}

/// Un PNG como lo lee qoiconv.
pub fn leer_png(datos: &[u8]) -> Resultado<EntradaQoi> {
    let f = |e: png::DecodingError| Error::lectura("PNG", e);
    let mut d = png::Decoder::new(Cursor::new(datos));
    d.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    d.set_ignore_text_chunk(true);
    let mut lector = d.read_info().map_err(f)?;
    let tam = lector
        .output_buffer_size()
        .ok_or_else(|| Error::lectura("PNG", "la imagen es demasiado grande"))?;
    let mut buf = vec![0; tam];
    let marco = lector.next_frame(&mut buf).map_err(f)?;
    buf.truncate(marco.buffer_size());
    let info = lector.info();
    let (ancho, alto) = (info.width, info.height);
    let original = info.color_type;
    // stb_image solo ve el tRNS de una paleta si llega antes de IDAT, que es
    // donde manda la norma.
    let trns_paleta = original == ColorType::Indexed && info.trns.is_some();
    let (salida, _) = lector.output_color_type();

    let canales = match original {
        ColorType::Rgb => 3,
        ColorType::Indexed if !trns_paleta => 3,
        _ => 4,
    };
    let pixeles = match (salida, canales) {
        (ColorType::Rgb, 3) => buf,
        (ColorType::Rgba, 3) => crate::entrada::quitar_alfa(&buf),
        (ColorType::Rgba, 4) => buf,
        (ColorType::Rgb, 4) => buf
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        (ColorType::Grayscale, 4) => buf.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        (ColorType::GrayscaleAlpha, 4) => buf
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        _ => return Err(Error::lectura("PNG", "tipo de color inesperado")),
    };
    Ok(EntradaQoi {
        ancho,
        alto,
        pixeles,
        canales,
        de_qoiconv: true,
    })
}

/// De píxeles RGBA (cualquier otro formato, o una imagen procesada).
pub fn desde_rgba(ancho: u32, alto: u32, rgba: &[u8]) -> EntradaQoi {
    let alfa = rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 255);
    EntradaQoi {
        ancho,
        alto,
        pixeles: if alfa {
            rgba.to_vec()
        } else {
            crate::entrada::quitar_alfa(rgba)
        },
        canales: if alfa { 4 } else { 3 },
        de_qoiconv: false,
    }
}

/// Codifica en QOI: `qoi_encode` de qoi.h, trasladado tal cual. El crate
/// `qoi` da ficheros válidos pero no los mismos: apunta los colores en la
/// tabla de índices también cuando los encuentra, y la referencia solo
/// cuando no (ADR 0020).
pub fn codificar(e: &EntradaQoi) -> Resultado<Vec<u8>> {
    let canales = e.canales as usize;
    let n = e.ancho as usize * e.alto as usize;
    if e.ancho == 0 || e.alto == 0 || !(3..=4).contains(&canales) || e.pixeles.len() != n * canales
    {
        return Err(Error::Codificacion(
            "QOI: dimensiones o canales no válidos".into(),
        ));
    }
    // QOI_PIXELS_MAX: 400 millones de píxeles.
    if e.alto as u64 >= 400_000_000 / e.ancho as u64 {
        return Err(Error::Codificacion(
            "QOI: la imagen es demasiado grande".into(),
        ));
    }
    const INDEX: u8 = 0x00;
    const DIFF: u8 = 0x40;
    const LUMA: u8 = 0x80;
    const RUN: u8 = 0xc0;
    const RGB: u8 = 0xfe;
    const RGBA: u8 = 0xff;
    let hash = |p: [u8; 4]| {
        (p[0] as usize * 3 + p[1] as usize * 5 + p[2] as usize * 7 + p[3] as usize * 11) % 64
    };

    let mut b = Vec::with_capacity(n * (canales + 1) + 14 + 8);
    b.extend_from_slice(b"qoif");
    b.extend_from_slice(&e.ancho.to_be_bytes());
    b.extend_from_slice(&e.alto.to_be_bytes());
    b.push(e.canales);
    b.push(0); // QOI_SRGB

    let mut indice = [[0u8; 4]; 64];
    let mut previo = [0u8, 0, 0, 255];
    let mut px = previo;
    let mut racha: u8 = 0;
    let fin = n * canales - canales;
    for pos in (0..n * canales).step_by(canales) {
        let d = &e.pixeles[pos..pos + canales];
        px[0] = d[0];
        px[1] = d[1];
        px[2] = d[2];
        if canales == 4 {
            px[3] = d[3];
        }
        if px == previo {
            racha += 1;
            if racha == 62 || pos == fin {
                b.push(RUN | (racha - 1));
                racha = 0;
            }
        } else {
            if racha > 0 {
                b.push(RUN | (racha - 1));
                racha = 0;
            }
            let h = hash(px);
            if indice[h] == px {
                b.push(INDEX | h as u8);
            } else {
                indice[h] = px;
                if px[3] == previo[3] {
                    let vr = px[0].wrapping_sub(previo[0]) as i8;
                    let vg = px[1].wrapping_sub(previo[1]) as i8;
                    let vb = px[2].wrapping_sub(previo[2]) as i8;
                    let vg_r = vr.wrapping_sub(vg);
                    let vg_b = vb.wrapping_sub(vg);
                    // vr > -3 && vr < 2: de -2 a 1 (y no -3..2, que incluye el -3).
                    if (-2..2).contains(&vr) && (-2..2).contains(&vg) && (-2..2).contains(&vb) {
                        b.push(
                            DIFF | ((vr + 2) as u8) << 4 | ((vg + 2) as u8) << 2 | (vb + 2) as u8,
                        );
                    } else if (-8..8).contains(&vg_r)
                        && (-32..32).contains(&vg)
                        && (-8..8).contains(&vg_b)
                    {
                        b.push(LUMA | (vg + 32) as u8);
                        b.push(((vg_r + 8) as u8) << 4 | (vg_b + 8) as u8);
                    } else {
                        b.extend_from_slice(&[RGB, px[0], px[1], px[2]]);
                    }
                } else {
                    b.extend_from_slice(&[RGBA, px[0], px[1], px[2], px[3]]);
                }
            }
        }
        previo = px;
    }
    b.extend_from_slice(&[0, 0, 0, 0, 0, 0, 0, 1]);
    Ok(b)
}

/// La orden para enseñar.
pub fn texto(entrada: &str, salida: &str) -> String {
    format!(
        "qoiconv {} {}",
        crate::cwebp::citar(entrada),
        crate::cwebp::citar(salida)
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn otro_lector_lo_lee_igual() {
        // Rachas, índices, diferencias pequeñas y grandes, y alfa.
        let mut rgba = Vec::new();
        for i in 0..4096u32 {
            let v = (i / 7) as u8;
            rgba.extend_from_slice(&[
                v,
                v.wrapping_mul(3),
                (i % 13) as u8 * 20,
                if i % 500 < 3 { 128 } else { 255 },
            ]);
        }
        let e = desde_rgba(64, 64, &rgba);
        assert_eq!(e.canales, 4);
        let q = codificar(&e).unwrap();
        let leida = image::load_from_memory_with_format(&q, image::ImageFormat::Qoi)
            .unwrap()
            .into_rgba8()
            .into_raw();
        assert_eq!(leida, rgba);
    }
}
