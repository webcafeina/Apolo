//! WebP de entrada, como `imageio/webpdec.c`.
//!
//! No se decodifica aquí: cwebp lo decodifica directamente al espacio que va
//! a usar el codificador (YUV, o ARGB si hace falta), y eso se repite al
//! importar (`webp::importar`). Aquí se miran las dimensiones y los metadatos.

use libwebp_sys as w;

use super::{Formato, Imagen, Lectura, Pixeles};
use crate::error::{Error, Resultado};
use crate::metadatos::Metadatos;

const F: &str = "WebP";

pub(super) fn leer(datos: &[u8], lectura: Lectura) -> Resultado<Imagen> {
    let mut rasgos = std::mem::MaybeUninit::<w::WebPBitstreamFeatures>::zeroed();
    // SAFETY: datos válidos durante la llamada; rasgos es memoria propia.
    let estado = unsafe {
        w::WebPGetFeaturesInternal(
            datos.as_ptr(),
            datos.len(),
            rasgos.as_mut_ptr(),
            w::WEBP_DECODER_ABI_VERSION as i32,
        )
    };
    if estado != w::VP8StatusCode::VP8_STATUS_OK {
        return Err(Error::lectura(F, format!("{estado:?}")));
    }
    // SAFETY: inicializado por WebPGetFeatures, que ha devuelto OK.
    let rasgos = unsafe { rasgos.assume_init() };
    if rasgos.has_animation != 0 {
        return Err(Error::lectura(F, "los WebP animados no se admiten"));
    }
    let metadatos = if lectura.metadatos {
        metadatos(datos)?
    } else {
        Metadatos::default()
    };
    Ok(Imagen {
        ancho: rasgos.width as u32,
        alto: rasgos.height as u32,
        formato: Formato::WebP,
        pixeles: Pixeles::WebP(datos.to_vec()),
        metadatos,
    })
}

/// Los trozos ICCP, EXIF y «XMP » del contenedor, con WebPDemux.
fn metadatos(datos: &[u8]) -> Resultado<Metadatos> {
    let mut m = Metadatos::default();
    let entrada = w::WebPData {
        bytes: datos.as_ptr(),
        size: datos.len(),
    };
    // SAFETY: el demuxer no sobrevive a `datos` y se libera al final.
    unsafe {
        let demux = w::WebPDemuxInternal(
            &entrada,
            0,
            std::ptr::null_mut(),
            w::WEBP_DEMUX_ABI_VERSION as i32,
        );
        if demux.is_null() {
            return Err(Error::lectura(F, "no se pudo leer el contenedor"));
        }
        let banderas = w::WebPDemuxGetI(demux, w::WebPFormatFeature::WEBP_FF_FORMAT_FLAGS);
        for (fourcc, bandera, destino) in [
            (b"ICCP", w::WebPFeatureFlags::ICCP_FLAG as u32, &mut m.icc),
            (b"EXIF", w::WebPFeatureFlags::EXIF_FLAG as u32, &mut m.exif),
            (b"XMP ", w::WebPFeatureFlags::XMP_FLAG as u32, &mut m.xmp),
        ] {
            if banderas & bandera == 0 {
                continue;
            }
            let mut it = std::mem::zeroed::<w::WebPChunkIterator>();
            if w::WebPDemuxGetChunk(demux, fourcc.as_ptr() as *const _, 1, &mut it) != 0 {
                let c = it.chunk;
                if !c.bytes.is_null() && c.size > 0 {
                    *destino = Some(std::slice::from_raw_parts(c.bytes, c.size).to_vec());
                }
            }
            w::WebPDemuxReleaseChunkIterator(&mut it);
        }
        w::WebPDemuxDelete(demux);
    }
    Ok(m)
}
