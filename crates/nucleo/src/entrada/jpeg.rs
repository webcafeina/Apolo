//! JPEG, como `imageio/jpegdec.c`.
//!
//! libjpeg-turbo (aquí, a través de mozjpeg, que decodifica con el mismo
//! código) con salida RGB, DCT entera lenta y `do_fancy_upsampling`, que son
//! los valores por defecto. Siempre RGB: un JPEG no tiene transparencia.
//!
//! El crate `mozjpeg` señala los errores **con un pánico**, así que todo va
//! dentro de `catch_unwind`. Por eso el perfil de compilación no puede usar
//! `panic = "abort"` (ver docs/trampas.md).

use std::panic::{AssertUnwindSafe, catch_unwind};

use mozjpeg::{Decompress, Marker};

use super::{Formato, Imagen, Lectura, Pixeles};
use crate::error::{Error, Resultado};
use crate::metadatos::Metadatos;

const F: &str = "JPEG";

pub(super) fn leer(datos: &[u8], lectura: Lectura) -> Resultado<Imagen> {
    let resultado = catch_unwind(AssertUnwindSafe(|| leer_sin_proteger(datos, lectura)));
    match resultado {
        Ok(r) => r,
        Err(panico) => {
            let detalle = panico
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| panico.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "el fichero está dañado".into());
            Err(Error::lectura(F, detalle))
        }
    }
}

fn leer_sin_proteger(datos: &[u8], lectura: Lectura) -> Resultado<Imagen> {
    let d = Decompress::with_markers(&[Marker::APP(1), Marker::APP(2)])
        .from_mem(datos)
        .map_err(|e| Error::lectura(F, e))?;

    let metadatos = if lectura.metadatos {
        let marcas: Vec<(u8, Vec<u8>)> = d
            .markers()
            .map(|m| {
                let n = match m.marker {
                    Marker::APP(n) => n,
                    _ => 0,
                };
                (n, m.data.to_vec())
            })
            .collect();
        metadatos(&marcas)?
    } else {
        Metadatos::default()
    };

    let mut rgb = d.rgb().map_err(|e| Error::lectura(F, e))?;
    let (ancho, alto) = (rgb.width() as u32, rgb.height() as u32);
    let pixeles: Vec<u8> = rgb
        .read_scanlines::<u8>()
        .map_err(|e| Error::lectura(F, e))?;
    rgb.finish().map_err(|e| Error::lectura(F, e))?;

    if pixeles.len() != ancho as usize * alto as usize * 3 {
        return Err(Error::lectura(F, "faltan líneas"));
    }
    Ok(Imagen {
        ancho,
        alto,
        formato: Formato::Jpeg,
        pixeles: Pixeles::Rgb(pixeles),
        metadatos,
    })
}

/// `ExtractMetadataFromJPEG`: el ICC, que puede venir troceado y desordenado
/// en varios APP2, y el primer EXIF y XMP de los APP1.
fn metadatos(marcas: &[(u8, Vec<u8>)]) -> Resultado<Metadatos> {
    let mut m = Metadatos {
        icc: icc(marcas)?,
        ..Default::default()
    };
    const EXIF: &[u8] = b"Exif\0\0";
    const XMP: &[u8] = b"http://ns.adobe.com/xap/1.0/\0";
    for (n, datos) in marcas {
        if *n != 1 {
            continue;
        }
        for (firma, destino) in [(EXIF, &mut m.exif), (XMP, &mut m.xmp)] {
            if datos.len() > firma.len() && datos.starts_with(firma) && destino.is_none() {
                *destino = Some(datos[firma.len()..].to_vec());
            }
        }
    }
    Ok(m)
}

fn icc(marcas: &[(u8, Vec<u8>)]) -> Resultado<Option<Vec<u8>>> {
    const FIRMA: &[u8] = b"ICC_PROFILE\0";
    let mal = |d: &str| {
        Err(Error::lectura(
            F,
            format!("el perfil ICC está mal troceado: {d}"),
        ))
    };
    let mut trozos: Vec<Option<&[u8]>> = vec![None; 255];
    let (mut esperados, mut vistos, mut maximo) = (0usize, 0usize, 0usize);
    for (n, datos) in marcas {
        if *n != 2 || datos.len() <= FIRMA.len() + 2 || !datos.starts_with(FIRMA) {
            continue;
        }
        let seq = datos[FIRMA.len()] as usize;
        let cuenta = datos[FIRMA.len() + 1] as usize;
        if cuenta == 0 || seq == 0 {
            return mal("número de trozo o cuenta a cero");
        }
        if esperados == 0 {
            esperados = cuenta;
        } else if esperados != cuenta {
            return mal("la cuenta de trozos no coincide");
        }
        if trozos[seq - 1].is_some() {
            return mal("trozo repetido");
        }
        trozos[seq - 1] = Some(&datos[FIRMA.len() + 2..]);
        vistos += 1;
        maximo = maximo.max(seq);
    }
    if vistos == 0 {
        return Ok(None);
    }
    if maximo != vistos || esperados != vistos {
        return mal("faltan trozos");
    }
    Ok(Some(
        trozos[..maximo]
            .iter()
            .flat_map(|t| t.unwrap().iter().copied())
            .collect(),
    ))
}
