//! Píxeles para enseñar: el original y el resultado, en RGBA de 8 bits.
//!
//! La interfaz pinta los dos en un `<canvas>` con los mismos píxeles crudos.
//! Así ninguno de los dos pasa por la gestión de color del navegador, y lo que
//! se compara es exactamente lo que hay en cada fichero (ADR 0013).

use libwebp_sys as w;

use crate::entrada::{Imagen, Pixeles};
use crate::error::{Error, Resultado};

/// RGBA de un WebP, decodificado con libwebp.
pub fn decodificar_webp(datos: &[u8]) -> Resultado<(u32, u32, Vec<u8>)> {
    let (mut ancho, mut alto) = (0, 0);
    // SAFETY: búfer propio; la salida se libera con WebPFree.
    unsafe {
        let p = w::WebPDecodeRGBA(datos.as_ptr(), datos.len(), &mut ancho, &mut alto);
        if p.is_null() {
            return Err(Error::lectura("WebP", "no se pudo decodificar"));
        }
        let v = std::slice::from_raw_parts(p, (ancho * alto * 4) as usize).to_vec();
        w::WebPFree(p as *mut _);
        Ok((ancho as u32, alto as u32, v))
    }
}

/// RGBA de una imagen leída, sea del formato que sea.
pub fn rgba(img: &Imagen) -> Resultado<(u32, u32, Vec<u8>)> {
    let (w, h) = (img.ancho, img.alto);
    Ok(match &img.pixeles {
        Pixeles::Rgba(v) => (w, h, v.clone()),
        Pixeles::Rgbx(v) => (
            w,
            h,
            v.as_chunks::<4>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect(),
        ),
        Pixeles::Rgb(v) => (
            w,
            h,
            v.as_chunks::<3>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect(),
        ),
        Pixeles::WebP(d) => decodificar_webp(d)?,
        Pixeles::Yuv(d) => yuv_a_rgba(d, w, h),
    })
}

/// YUV 4:2:0 crudo a RGBA, con la conversión BT.601 de rango limitado que usa
/// libwebp. Solo para enseñarlo: no interviene en la codificación.
fn yuv_a_rgba(d: &[u8], ancho: u32, alto: u32) -> (u32, u32, Vec<u8>) {
    let (w, h) = (ancho as usize, alto as usize);
    let uw = w.div_ceil(2);
    let (pu, pv) = (w * h, w * h + uw * h.div_ceil(2));
    let mut s = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            let yy = d[y * w + x] as f32 - 16.0;
            let i = (y / 2) * uw + x / 2;
            let u = d[pu + i] as f32 - 128.0;
            let v = d[pv + i] as f32 - 128.0;
            let c = |f: f32| f.round().clamp(0.0, 255.0) as u8;
            s.extend([
                c(1.164 * yy + 1.596 * v),
                c(1.164 * yy - 0.391 * u - 0.813 * v),
                c(1.164 * yy + 2.018 * u),
                255,
            ]);
        }
    }
    (ancho, alto, s)
}

/// RGBA de un fichero de salida, para enseñarlo: WebP, JPEG, PNG o QOI.
/// Sin corrección de gamma (como lo pinta un navegador).
pub fn decodificar(datos: &[u8]) -> Resultado<(u32, u32, Vec<u8>)> {
    match datos.get(..4) {
        Some(b"RIFF") => decodificar_webp(datos),
        Some(b"qoif") => {
            let img = image::load_from_memory_with_format(datos, image::ImageFormat::Qoi)
                .map_err(|e| crate::Error::lectura("QOI", e))?;
            Ok((img.width(), img.height(), img.into_rgba8().into_raw()))
        }
        Some([0x89, b'P', b'N', b'G']) => {
            let mut d = png::Decoder::new(std::io::Cursor::new(datos));
            d.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
            let mut l = d.read_info().map_err(|e| crate::Error::lectura("PNG", e))?;
            let mut buf = vec![0; l.output_buffer_size().unwrap_or(0)];
            let marco = l
                .next_frame(&mut buf)
                .map_err(|e| crate::Error::lectura("PNG", e))?;
            buf.truncate(marco.buffer_size());
            let (w, h) = (marco.width, marco.height);
            let rgba = match marco.color_type {
                png::ColorType::Rgba => buf,
                png::ColorType::Rgb => buf
                    .as_chunks::<3>()
                    .0
                    .iter()
                    .flat_map(|p| [p[0], p[1], p[2], 255])
                    .collect(),
                png::ColorType::Grayscale => buf.iter().flat_map(|&g| [g, g, g, 255]).collect(),
                png::ColorType::GrayscaleAlpha => buf
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .flat_map(|p| [p[0], p[0], p[0], p[1]])
                    .collect(),
                png::ColorType::Indexed => {
                    return Err(crate::Error::lectura("PNG", "paleta sin expandir"));
                }
            };
            Ok((w, h, rgba))
        }
        _ => {
            // JPEG (u otro que Apolo lea).
            let img = crate::entrada::leer(
                datos,
                crate::entrada::Lectura {
                    conservar_alfa: true,
                    metadatos: false,
                },
            )?;
            rgba(&img)
        }
    }
}
