//! PNG, como `imageio/pngdec.c`.
//!
//! libpng con `png_set_strip_16`, paleta y tRNS expandidos, gris a RGB y
//! corrección de gamma cuando hay `sRGB` o `gAMA`. Con `sRGB`, el gamma de la
//! imagen queda en 1/2,2 y la corrección es nula; con un `gAMA` distinto de
//! 1/2,2, cambian los píxeles, y hay que hacerlo con la misma aritmética.

use std::io::Cursor;

use png::{BitDepth, ColorType, Transformations};

use super::{Formato, Imagen, Lectura, Pixeles, gris_a_rgb, gris_alfa_a_rgba, quitar_alfa};
use crate::error::{Error, Resultado};
use crate::metadatos::Metadatos;

const F: &str = "PNG";

pub(super) fn leer(datos: &[u8], lectura: Lectura) -> Resultado<Imagen> {
    let mut decodificador = png::Decoder::new(Cursor::new(datos));
    decodificador.set_transformations(Transformations::EXPAND | Transformations::STRIP_16);
    decodificador.set_ignore_text_chunk(!lectura.metadatos);
    let mut lector = decodificador
        .read_info()
        .map_err(|e| Error::lectura(F, e))?;
    let tamano = lector
        .output_buffer_size()
        .ok_or_else(|| Error::lectura(F, "la imagen es demasiado grande"))?;
    let mut buf = vec![0; tamano];
    let marco = lector
        .next_frame(&mut buf)
        .map_err(|e| Error::lectura(F, e))?;
    buf.truncate(marco.buffer_size());
    // Lo que va detrás de la imagen (texto, EXIF) también cuenta: pngdec mira
    // al principio y al final.
    lector.finish().map_err(|e| Error::lectura(F, e))?;

    let (tipo, profundidad) = lector.output_color_type();
    debug_assert_eq!(profundidad, BitDepth::Eight);
    let info = lector.info();

    let mut pixeles = match tipo {
        ColorType::Rgb => Pixeles::Rgb(buf),
        ColorType::Rgba => Pixeles::Rgba(buf),
        ColorType::Grayscale => Pixeles::Rgb(gris_a_rgb(&buf)),
        ColorType::GrayscaleAlpha => Pixeles::Rgba(gris_alfa_a_rgba(&buf)),
        ColorType::Indexed => return Err(Error::lectura(F, "la paleta no se expandió")),
    };

    if info.srgb.is_none()
        && let Some(g) = info.gama_chunk
    {
        corregir_gamma(&mut pixeles, g.into_scaled());
    }

    if !lectura.conservar_alfa
        && let Pixeles::Rgba(rgba) = &pixeles
    {
        pixeles = Pixeles::Rgb(quitar_alfa(rgba));
    }

    let metadatos = if lectura.metadatos {
        metadatos(info)?
    } else {
        Metadatos::default()
    };

    Ok(Imagen {
        ancho: info.width,
        alto: info.height,
        formato: Formato::Png,
        pixeles,
        metadatos,
    })
}

/// `png_set_gamma(png, 2.2, gamma_de_la_imagen)` para 8 bits, con la
/// aritmética de libpng 1.6: valores en punto fijo de 1e-5, umbral del 5 %
/// para decidir si la corrección importa, y la tabla de `png_gamma_8bit_correct`.
fn corregir_gamma(pixeles: &mut Pixeles, gamma_imagen: u32) {
    const PANTALLA: u64 = 220_000; // 2,2
    if gamma_imagen == 0 {
        return;
    }
    // png_gamma_significant(png_product2(pantalla, imagen))
    let producto = PANTALLA as f64 * gamma_imagen as f64 * 1e-5;
    if (producto - 100_000.0).abs() < 5_000.0 {
        return;
    }
    // png_reciprocal2(imagen, pantalla)
    let exponente = (1e15 / gamma_imagen as f64 / PANTALLA as f64 + 0.5).floor();
    let tabla: Vec<u8> = (0..=255u32)
        .map(|i| {
            if i == 0 || i == 255 {
                i as u8
            } else {
                (255.0 * (i as f64 / 255.0).powf(exponente * 1e-5) + 0.5).floor() as u8
            }
        })
        .collect();
    let (datos, paso) = match pixeles {
        Pixeles::Rgb(v) => (v, 3),
        Pixeles::Rgba(v) => (v, 4),
        _ => return,
    };
    for p in datos.chunks_exact_mut(paso) {
        for c in &mut p[..3] {
            *c = tabla[*c as usize];
        }
    }
}

fn metadatos(info: &png::Info) -> Resultado<Metadatos> {
    let mut m = Metadatos::default();

    if let Some(exif) = &info.exif_metadata
        && !exif.is_empty()
    {
        m.exif = Some(exif.to_vec());
    }

    // pngdec recorre los trozos de texto en el orden del fichero y se queda
    // con el primero de cada tipo. El crate png los separa por clase (tEXt,
    // zTXt, iTXt), así que con trozos repetidos de clases distintas el orden
    // puede no coincidir. No se ha visto ninguna imagen así.
    let mut textos: Vec<(String, Vec<u8>)> = Vec::new();
    for t in &info.uncompressed_latin1_text {
        textos.push((t.keyword.clone(), latin1(&t.text)));
    }
    for t in &info.compressed_latin1_text {
        let mut t = t.clone();
        t.decompress_text().map_err(|e| Error::lectura(F, e))?;
        let texto = t.get_text().map_err(|e| Error::lectura(F, e))?;
        textos.push((t.keyword.clone(), latin1(&texto)));
    }
    for t in &info.utf8_text {
        let mut t = t.clone();
        t.decompress_text().map_err(|e| Error::lectura(F, e))?;
        let texto = t.get_text().map_err(|e| Error::lectura(F, e))?;
        textos.push((t.keyword.clone(), texto.into_bytes()));
    }
    for (clave, texto) in textos {
        let (destino, crudo) = match clave.as_str() {
            "Raw profile type exif" | "Raw profile type APP1" | "Raw profile type app1" => {
                (&mut m.exif, true)
            }
            "Raw profile type xmp" => (&mut m.xmp, true),
            "XML:com.adobe.xmp" => (&mut m.xmp, false),
            _ => continue,
        };
        if destino.is_some() {
            continue; // «Ignoring additional»
        }
        let datos = if crudo {
            perfil_crudo(&texto)
                .ok_or_else(|| Error::lectura(F, format!("el perfil «{clave}» está mal formado")))?
        } else if texto.is_empty() {
            return Err(Error::lectura(F, format!("«{clave}» está vacío")));
        } else {
            texto
        };
        *destino = Some(datos);
    }

    if let Some(icc) = &info.icc_profile {
        let color = matches!(
            info.color_type,
            ColorType::Rgb | ColorType::Rgba | ColorType::Indexed
        );
        m.icc = icc_valido(icc, color);
    }
    Ok(m)
}

/// libpng (1.6, `png_handle_iCCP` y `png_icc_check_*`) descarta los perfiles
/// ICC que no pasan sus comprobaciones, con solo un aviso, y cwebp nunca los
/// ve. Hay que descartar los mismos. Lo que sobra tras la longitud que dice la
/// cabecera también se tira.
fn icc_valido(perfil: &[u8], png_en_color: bool) -> Option<Vec<u8>> {
    let be32 =
        |i: usize| u32::from_be_bytes([perfil[i], perfil[i + 1], perfil[i + 2], perfil[i + 3]]);
    if perfil.len() < 132 {
        return None;
    }
    let longitud = be32(0) as usize;
    if longitud < 132 || longitud > perfil.len() {
        return None; // demasiado corto, o truncado
    }
    if perfil[8] > 3 && longitud & 3 != 0 {
        return None;
    }
    let etiquetas = be32(128) as usize;
    if etiquetas > 357_913_930 || longitud < 132 + 12 * etiquetas {
        return None;
    }
    if be32(64) >= 0xffff {
        return None; // intención de renderizado no válida
    }
    if be32(36) != 0x6163_7370 {
        return None; // 'acsp'
    }
    match be32(16) {
        0x5247_4220 if png_en_color => {}  // 'RGB '
        0x4752_4159 if !png_en_color => {} // 'GRAY'
        _ => return None,
    }
    if matches!(be32(12), 0x6162_7374 | 0x6c69_6e6b) {
        return None; // 'abst', 'link'
    }
    if !matches!(be32(20), 0x5859_5a20 | 0x4c61_6220) {
        return None; // PCS: 'XYZ ' o 'Lab '
    }
    for t in 0..etiquetas {
        let base = 132 + 12 * t;
        let (inicio, largo) = (be32(base + 4) as usize, be32(base + 8) as usize);
        if inicio > longitud || largo > longitud - inicio {
            return None;
        }
    }
    Some(perfil[..longitud].to_vec())
}

/// El texto de tEXt/zTXt es Latin-1; el crate lo da en UTF-8.
fn latin1(s: &str) -> Vec<u8> {
    s.chars().map(|c| c as u32 as u8).collect()
}

/// Los «raw profiles» de ImageMagick: `\n<nombre>\n<longitud>\n<hex…>`.
fn perfil_crudo(texto: &[u8]) -> Option<Vec<u8>> {
    let resto = texto.strip_prefix(b"\n")?;
    let fin_nombre = resto.iter().position(|&b| b == b'\n')?;
    let resto = &resto[fin_nombre + 1..];
    // strtol: se salta los espacios de delante, lee dígitos, y detrás tiene
    // que venir '\n'.
    let inicio = resto.iter().position(|b| !b.is_ascii_whitespace())?;
    let resto = &resto[inicio..];
    let fin_num = resto.iter().position(|b| !b.is_ascii_digit())?;
    let longitud: usize = std::str::from_utf8(&resto[..fin_num]).ok()?.parse().ok()?;
    if resto.get(fin_num) != Some(&b'\n') {
        return None;
    }
    let hex = &resto[fin_num + 1..];
    let mut salida = Vec::with_capacity(longitud);
    let mut i = 0;
    while salida.len() < longitud && i < hex.len() {
        if hex[i] == b'\n' {
            i += 1;
            continue;
        }
        let par = hex.get(i..i + 2)?;
        let v = u8::from_str_radix(std::str::from_utf8(par).ok()?, 16).ok()?;
        salida.push(v);
        i += 2;
    }
    (salida.len() == longitud).then_some(salida)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn perfil_crudo_de_imagemagick() {
        let t = b"\nexif\n       4\n0a0b\n0c0d\n";
        assert_eq!(perfil_crudo(t), Some(vec![0x0a, 0x0b, 0x0c, 0x0d]));
        assert_eq!(perfil_crudo(b"exif\n4\n00"), None);
        assert_eq!(perfil_crudo(b"\nexif\n       4\n0a0b"), None);
    }
}
