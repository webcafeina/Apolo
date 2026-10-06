//! Codificar a WebP: el `main()` de `examples/cwebp.c` (libwebp 1.6.0) desde
//! que tiene la imagen leída hasta que escribe el fichero, en el mismo orden.
//!
//! Mezclar el alfa, recortar con una vista, aplicar el modo de redimensión,
//! redimensionar (con el truco de `-exact`), codificar, medir la distorsión y
//! añadir los metadatos. Cambiar el orden de cualquiera de esos pasos cambia
//! los bytes.

use std::ffi::c_void;

use libwebp_sys as w;
use serde::Serialize;

use super::opciones::{ModoRedimension, OpcionesWebp};
use crate::entrada::{Imagen, Pixeles, quitar_alfa};
use crate::error::{Error, Resultado};
use crate::metadatos::{self, Escritos};

/// Qué distorsión medir (`-print_psnr`, `-print_ssim`, `-print_lsim`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Medida {
    Psnr = 0,
    Ssim = 1,
    Lsim = 2,
}

/// Lo que cwebp puede sacar además del fichero.
#[derive(Debug, Clone, Copy, Default)]
pub struct Extras {
    pub medir: Option<Medida>,
    /// `-map`: tipo de mapa de información por macrobloque (0 = ninguno).
    pub mapa: i32,
    /// `-d`: volcar el resultado decodificado como PGM (solo con pérdida).
    pub volcar: bool,
}

/// Las estadísticas de libwebp (`WebPAuxStats`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct Estadisticas {
    pub bytes: i32,
    /// PSNR de Y, U, V, total y alfa.
    pub psnr: [f32; 5],
    /// Macrobloques intra4, intra16 y saltados.
    pub bloques: [i32; 3],
    pub bytes_cabecera: [i32; 2],
    pub bytes_residuos: [[i32; 4]; 3],
    pub tamano_segmento: [i32; 4],
    pub cuantizador_segmento: [i32; 4],
    pub filtro_segmento: [i32; 4],
    pub bytes_alfa: i32,
    pub sin_perdida_rasgos: u32,
    pub bits_histograma: i32,
    pub bits_transformada: i32,
    pub bits_color_cruzado: i32,
    pub bits_cache: i32,
    pub tamano_paleta: i32,
    pub sin_perdida_bytes: i32,
    pub sin_perdida_cabecera: i32,
    pub sin_perdida_datos: i32,
}

impl From<&w::WebPAuxStats> for Estadisticas {
    fn from(s: &w::WebPAuxStats) -> Self {
        Estadisticas {
            bytes: s.coded_size,
            psnr: s.PSNR,
            bloques: s.block_count,
            bytes_cabecera: s.header_bytes,
            bytes_residuos: s.residual_bytes,
            tamano_segmento: s.segment_size,
            cuantizador_segmento: s.segment_quant,
            filtro_segmento: s.segment_level,
            bytes_alfa: s.alpha_data_size,
            sin_perdida_rasgos: s.lossless_features,
            bits_histograma: s.histogram_bits,
            bits_transformada: s.transform_bits,
            bits_color_cruzado: s.cross_color_transform_bits,
            bits_cache: s.cache_bits,
            tamano_paleta: s.palette_size,
            sin_perdida_bytes: s.lossless_size,
            sin_perdida_cabecera: s.lossless_hdr_size,
            sin_perdida_datos: s.lossless_data_size,
        }
    }
}

/// El resultado de codificar.
#[derive(Debug, Clone)]
pub struct Codificado {
    /// El fichero WebP entero, con los metadatos pedidos.
    pub datos: Vec<u8>,
    pub ancho: u32,
    pub alto: u32,
    pub sin_perdida: bool,
    pub estadisticas: Estadisticas,
    pub metadatos: Escritos,
    /// B, G, R, A y total.
    pub distorsion: Option<[f32; 5]>,
    pub mapa: Option<Vec<u8>>,
    pub volcado: Option<Vec<u8>>,
}

/// Una `WebPPicture` que se libera sola.
pub(crate) struct Picture(pub w::WebPPicture);

impl Picture {
    pub(crate) fn nueva() -> Resultado<Self> {
        let mut p = std::mem::MaybeUninit::<w::WebPPicture>::zeroed();
        // SAFETY: inicializa la estructura entera.
        let ok = unsafe {
            w::WebPPictureInitInternal(p.as_mut_ptr(), w::WEBP_ENCODER_ABI_VERSION as i32)
        };
        if ok == 0 {
            return Err(Error::Codificacion(
                "versión de libwebp incompatible".into(),
            ));
        }
        // SAFETY: inicializada arriba.
        Ok(Picture(unsafe { p.assume_init() }))
    }
}

impl Drop for Picture {
    fn drop(&mut self) {
        // SAFETY: la picture es nuestra; WebPPictureFree admite una vacía.
        unsafe {
            w::WebPFree(self.0.extra_info as *mut c_void);
            self.0.extra_info = std::ptr::null_mut();
            w::WebPPictureFree(&mut self.0);
        }
    }
}

struct Escritor(w::WebPMemoryWriter);

impl Escritor {
    fn nuevo() -> Self {
        let mut m = std::mem::MaybeUninit::<w::WebPMemoryWriter>::zeroed();
        // SAFETY: inicializa la estructura entera.
        unsafe { w::WebPMemoryWriterInit(m.as_mut_ptr()) };
        // SAFETY: inicializada arriba.
        Escritor(unsafe { m.assume_init() })
    }

    fn bytes(&self) -> &[u8] {
        if self.0.mem.is_null() {
            return &[];
        }
        // SAFETY: mem tiene size bytes escritos por WebPMemoryWrite.
        unsafe { std::slice::from_raw_parts(self.0.mem, self.0.size) }
    }
}

impl Drop for Escritor {
    fn drop(&mut self) {
        // SAFETY: memoria reservada por WebPMemoryWrite.
        unsafe { w::WebPMemoryWriterClear(&mut self.0) };
    }
}

/// El progreso que se pasa a libwebp. Devolver `false` cancela.
pub type Progreso<'a> = &'a mut dyn FnMut(i32) -> bool;

unsafe extern "C" fn gancho_progreso(porcentaje: i32, picture: *const w::WebPPicture) -> i32 {
    // SAFETY: user_data apunta a un `Progreso` vivo mientras dura WebPEncode.
    unsafe {
        let p = &mut *((*picture).user_data as *mut Progreso);
        p(porcentaje) as i32
    }
}

fn error_libwebp(codigo: w::WebPEncodingError) -> Error {
    use w::WebPEncodingError::*;
    let texto = match codigo {
        VP8_ENC_ERROR_OUT_OF_MEMORY | VP8_ENC_ERROR_BITSTREAM_OUT_OF_MEMORY => {
            return Error::Memoria;
        }
        VP8_ENC_ERROR_USER_ABORT => return Error::Cancelado,
        VP8_ENC_ERROR_NULL_PARAMETER => "parámetro nulo",
        VP8_ENC_ERROR_INVALID_CONFIGURATION => "la configuración no es válida",
        VP8_ENC_ERROR_BAD_DIMENSION => {
            "dimensiones no válidas: el máximo es 16383 píxeles por lado"
        }
        VP8_ENC_ERROR_PARTITION0_OVERFLOW => {
            "la partición 0 no cabe en 512 KB; prueba con menos segmentos o con límite de partición"
        }
        VP8_ENC_ERROR_PARTITION_OVERFLOW => "una partición no cabe en 16 MB",
        VP8_ENC_ERROR_BAD_WRITE => "error al escribir",
        VP8_ENC_ERROR_FILE_TOO_BIG => "el fichero pasaría de 4 GB",
        _ => "error desconocido",
    };
    Error::Codificacion(texto.into())
}

/// Pasa los píxeles leídos a la picture, como los lectores de imageio.
pub(crate) fn importar(img: &Imagen, pic: &mut Picture, conservar_alfa: bool) -> Resultado<()> {
    let p = &mut pic.0;
    let (ancho, alto) = (img.ancho as i32, img.alto as i32);
    let fallo = || Error::Codificacion("no se pudo importar la imagen".into());
    // SAFETY: los búferes tienen el tamaño que dicen ancho, alto y stride.
    unsafe {
        match &img.pixeles {
            Pixeles::Rgb(v) => {
                p.width = ancho;
                p.height = alto;
                (w::WebPPictureImportRGB(p, v.as_ptr(), ancho * 3) != 0)
                    .then_some(())
                    .ok_or_else(fallo)
            }
            Pixeles::Rgba(v) if !conservar_alfa => {
                let rgb = quitar_alfa(v);
                p.width = ancho;
                p.height = alto;
                (w::WebPPictureImportRGB(p, rgb.as_ptr(), ancho * 3) != 0)
                    .then_some(())
                    .ok_or_else(fallo)
            }
            Pixeles::Rgba(v) => {
                p.width = ancho;
                p.height = alto;
                (w::WebPPictureImportRGBA(p, v.as_ptr(), ancho * 4) != 0)
                    .then_some(())
                    .ok_or_else(fallo)
            }
            Pixeles::Rgbx(v) => {
                p.width = ancho;
                p.height = alto;
                (w::WebPPictureImportRGBX(p, v.as_ptr(), ancho * 4) != 0)
                    .then_some(())
                    .ok_or_else(fallo)
            }
            Pixeles::WebP(datos) => importar_webp(datos, p, conservar_alfa),
            Pixeles::Yuv(datos) => importar_yuv(datos, ancho, alto, p),
        }
    }
}

/// `ReadWebP` de imageio/webpdec.c: decodifica directamente a la picture, en
/// YUV(A) o en ARGB según `use_argb`.
unsafe fn importar_webp(
    datos: &[u8],
    p: &mut w::WebPPicture,
    conservar_alfa: bool,
) -> Resultado<()> {
    let fallo = |e: &str| Error::lectura("WebP", e);
    unsafe {
        let mut config = std::mem::MaybeUninit::<w::WebPDecoderConfig>::zeroed().assume_init();
        if !w::WebPInitDecoderConfig(&mut config) {
            return Err(fallo("versión de libwebp incompatible"));
        }
        if w::WebPGetFeaturesInternal(
            datos.as_ptr(),
            datos.len(),
            &mut config.input,
            w::WEBP_DECODER_ABI_VERSION as i32,
        ) != w::VP8StatusCode::VP8_STATUS_OK
        {
            return Err(fallo("cabecera no válida"));
        }
        let rasgos = config.input;
        let tiene_alfa = conservar_alfa && rasgos.has_alpha != 0;
        p.width = rasgos.width;
        p.height = rasgos.height;
        if p.use_argb == 0 {
            p.colorspace = if tiene_alfa {
                w::WebPEncCSP::WEBP_YUV420A
            } else {
                w::WebPEncCSP::WEBP_YUV420
            };
        }
        if w::WebPPictureAlloc(p) == 0 {
            return Err(Error::Memoria);
        }
        let salida = &mut config.output;
        if p.use_argb != 0 {
            salida.colorspace = if cfg!(target_endian = "big") {
                w::WEBP_CSP_MODE::MODE_ARGB
            } else {
                w::WEBP_CSP_MODE::MODE_BGRA
            };
            salida.u.RGBA.rgba = p.argb as *mut u8;
            salida.u.RGBA.stride = p.argb_stride * 4;
            salida.u.RGBA.size = (p.argb_stride * 4 * p.height) as usize;
        } else {
            salida.colorspace = if tiene_alfa {
                w::WEBP_CSP_MODE::MODE_YUVA
            } else {
                w::WEBP_CSP_MODE::MODE_YUV
            };
            let yuva = &mut salida.u.YUVA;
            yuva.y = p.y;
            yuva.u = p.u;
            yuva.v = p.v;
            yuva.a = if tiene_alfa {
                p.a
            } else {
                std::ptr::null_mut()
            };
            yuva.y_stride = p.y_stride;
            yuva.u_stride = p.uv_stride;
            yuva.v_stride = p.uv_stride;
            yuva.a_stride = if tiene_alfa { p.a_stride } else { 0 };
            yuva.y_size = (p.height * p.y_stride) as usize;
            yuva.u_size = ((p.height + 1) / 2 * p.uv_stride) as usize;
            yuva.v_size = ((p.height + 1) / 2 * p.uv_stride) as usize;
            yuva.a_size = (p.height * p.a_stride) as usize;
        }
        salida.is_external_memory = 1;
        let estado = w::WebPDecode(datos.as_ptr(), datos.len(), &mut config);
        w::WebPFreeDecBuffer(&mut config.output);
        if estado != w::VP8StatusCode::VP8_STATUS_OK {
            return Err(fallo(&format!("{estado:?}")));
        }
        if !conservar_alfa && p.use_argb != 0 {
            for y in 0..p.height {
                let fila = std::slice::from_raw_parts_mut(
                    p.argb.add((y * p.argb_stride) as usize),
                    p.width as usize,
                );
                for px in fila {
                    *px |= 0xff00_0000;
                }
            }
        }
    }
    Ok(())
}

/// `ReadYUV` de cwebp.c.
unsafe fn importar_yuv(
    datos: &[u8],
    ancho: i32,
    alto: i32,
    p: &mut w::WebPPicture,
) -> Resultado<()> {
    let usa_argb = p.use_argb;
    let (uv_w, uv_h) = ((ancho + 1) / 2, (alto + 1) / 2);
    let y_tam = (ancho * alto) as usize;
    let uv_tam = (uv_w * uv_h) as usize;
    p.width = ancho;
    p.height = alto;
    p.use_argb = 0;
    unsafe {
        if w::WebPPictureAlloc(p) == 0 {
            return Err(Error::Memoria);
        }
        let copiar = |origen: &[u8], ancho_fila: i32, destino: *mut u8, stride: i32, filas: i32| {
            for f in 0..filas as usize {
                std::ptr::copy_nonoverlapping(
                    origen[f * ancho_fila as usize..].as_ptr(),
                    destino.add(f * stride as usize),
                    ancho_fila as usize,
                );
            }
        };
        copiar(&datos[..y_tam], ancho, p.y, p.y_stride, alto);
        copiar(&datos[y_tam..], uv_w, p.u, p.uv_stride, uv_h);
        copiar(&datos[y_tam + uv_tam..], uv_w, p.v, p.uv_stride, uv_h);
        if usa_argb != 0 && w::WebPPictureYUVAToARGB(p) == 0 {
            return Err(Error::Memoria);
        }
    }
    Ok(())
}

/// Codifica `img` con `op`.
pub fn codificar(
    img: &Imagen,
    op: &OpcionesWebp,
    extras: Extras,
    progreso: Option<Progreso>,
) -> Resultado<Codificado> {
    let mut config = op.a_config();
    if extras.medir.is_some() || extras.volcar {
        config.show_compressed = 1;
    }
    // SAFETY: config es propia y válida en memoria.
    if unsafe { w::WebPValidateConfig(&config) } == 0 {
        return Err(Error::Configuracion("libwebp no la acepta".into()));
    }

    let enderezada = if op.enderezar {
        crate::orientacion::enderezar(img)?
    } else {
        None
    };
    let img = enderezada.as_ref().unwrap_or(img);

    let mut pic = Picture::nueva()?;
    pic.0.use_argb = op.usa_argb() as i32;
    importar(img, &mut pic, !op.sin_alfa)?;

    // SAFETY: a partir de aquí todas las llamadas son sobre `pic`, propia y
    // con la memoria que libwebp le ha dado.
    unsafe {
        if let Some(color) = op.mezclar_alfa {
            w::WebPBlendAlpha(&mut pic.0, color & 0x00ff_ffff);
        }

        let mut escritor = Escritor::nuevo();
        pic.0.writer = Some(w::WebPMemoryWrite);
        pic.0.custom_ptr = &mut escritor.0 as *mut _ as *mut c_void;

        let mut estadisticas = std::mem::zeroed::<w::WebPAuxStats>();
        pic.0.stats = &mut estadisticas;

        let mut progreso = progreso;
        if let Some(p) = progreso.as_mut() {
            pic.0.progress_hook = Some(gancho_progreso);
            pic.0.user_data = p as *mut Progreso as *mut c_void;
        }

        if let Some(r) = op.recorte {
            let vista: *mut w::WebPPicture = &mut pic.0;
            if w::WebPPictureView(vista, r.x, r.y, r.ancho, r.alto, vista) == 0 {
                return Err(Error::Configuracion(
                    "el recorte se sale de la imagen".into(),
                ));
            }
        }

        let (mut rw, mut rh) = op.redimension.map(|r| (r.ancho, r.alto)).unwrap_or((0, 0));
        aplicar_modo(
            op.modo_redimension,
            pic.0.width,
            pic.0.height,
            &mut rw,
            &mut rh,
        );
        if (rw | rh) > 0 {
            redimensionar(&mut pic, rw, rh, op.exacto)?;
        }

        if extras.mapa > 0 {
            pic.0.extra_info_type = extras.mapa;
            let mb = ((pic.0.width + 15) / 16 * ((pic.0.height + 15) / 16)) as usize;
            pic.0.extra_info = w::WebPMalloc(mb) as *mut u8;
        }

        // Lossy modifica la picture al codificar: se guarda una copia para
        // medir. Lossless no la toca.
        let mut original = Picture::nueva()?;
        if extras.medir.is_some()
            && !op.sin_perdida
            && w::WebPPictureCopy(&pic.0, &mut original.0) == 0
        {
            return Err(Error::Memoria);
        }

        if w::WebPEncode(&config, &mut pic.0) == 0 {
            return Err(error_libwebp(pic.0.error_code));
        }
        pic.0.progress_hook = None;
        pic.0.user_data = std::ptr::null_mut();

        let (ancho, alto) = (pic.0.width as u32, pic.0.height as u32);

        let distorsion = match extras.medir {
            None => None,
            Some(m) => Some(medir(m, op, &mut pic, &mut original, escritor.bytes())?),
        };

        let volcado = if extras.volcar && pic.0.use_argb == 0 {
            Some(volcar_pgm(&pic.0))
        } else {
            None
        };

        let mapa = if extras.mapa > 0 && !pic.0.extra_info.is_null() {
            let mb = ((pic.0.width + 15) / 16 * ((pic.0.height + 15) / 16)) as usize;
            Some(std::slice::from_raw_parts(pic.0.extra_info, mb).to_vec())
        } else {
            None
        };

        let (datos, escritos) =
            metadatos::escribir(escritor.bytes(), ancho, alto, &img.metadatos, op.metadatos)
                .map_err(Error::Codificacion)?;

        Ok(Codificado {
            datos,
            ancho,
            alto,
            sin_perdida: op.sin_perdida,
            estadisticas: Estadisticas::from(&estadisticas),
            metadatos: escritos,
            distorsion,
            mapa,
            volcado,
        })
    }
}

/// `ApplyResizeMode` de cwebp.c.
fn aplicar_modo(modo: ModoRedimension, sw: i32, sh: i32, dw: &mut i32, dh: &mut i32) {
    match modo {
        ModoRedimension::SoloReducir => {
            if (*dw == 0 && sh <= *dh) || (*dh == 0 && sw <= *dw) || (sw <= *dw && sh <= *dh) {
                *dw = 0;
                *dh = 0;
            }
        }
        ModoRedimension::SoloAmpliar => {
            if sw >= *dw && sh >= *dh {
                *dw = 0;
                *dh = 0;
            }
        }
        ModoRedimension::Siempre => {}
    }
}

/// Redimensionar como cwebp. Con `-exact`, el RGB no se puede premultiplicar
/// por el alfa (se perdería donde A=0): se redimensiona aparte una copia
/// opaca y se vuelve a montar.
unsafe fn redimensionar(pic: &mut Picture, rw: i32, rh: i32, exacto: bool) -> Resultado<()> {
    unsafe {
        let mut opaca = Picture::nueva()?;
        if exacto {
            if w::WebPPictureCopy(&pic.0, &mut opaca.0) == 0 {
                return Err(Error::Memoria);
            }
            for y in 0..opaca.0.height {
                let fila = std::slice::from_raw_parts_mut(
                    opaca.0.argb.add((y * opaca.0.argb_stride) as usize),
                    opaca.0.width as usize,
                );
                for px in fila {
                    *px |= 0xff00_0000;
                }
            }
            if w::WebPPictureRescale(&mut opaca.0, rw, rh) == 0 {
                return Err(Error::Configuracion("no se pudo redimensionar".into()));
            }
        }
        if w::WebPPictureRescale(&mut pic.0, rw, rh) == 0 {
            return Err(Error::Configuracion("no se pudo redimensionar".into()));
        }
        if exacto {
            for y in 0..opaca.0.height {
                let o = std::slice::from_raw_parts(
                    opaca.0.argb.add((y * opaca.0.argb_stride) as usize),
                    opaca.0.width as usize,
                );
                let d = std::slice::from_raw_parts_mut(
                    pic.0.argb.add((y * pic.0.argb_stride) as usize),
                    pic.0.width as usize,
                );
                for (d, o) in d.iter_mut().zip(o) {
                    *d = (*d & 0xff00_0000) | (o & 0x00ff_ffff);
                }
            }
        }
    }
    Ok(())
}

/// La distorsión, como cwebp: lossy compara la copia previa con la picture
/// (que libwebp ha sobrescrito con lo comprimido); lossless puro no cambia la
/// picture; casi sin pérdida hay que decodificar el resultado.
unsafe fn medir(
    m: Medida,
    op: &OpcionesWebp,
    pic: &mut Picture,
    original: &mut Picture,
    webp: &[u8],
) -> Resultado<[f32; 5]> {
    let mut v = [0f32; 5];
    unsafe {
        let ok = if !op.sin_perdida {
            w::WebPPictureDistortion(&pic.0, &original.0, m as i32, v.as_mut_ptr())
        } else if op.casi_sin_perdida == 100 {
            w::WebPPictureDistortion(&pic.0, &pic.0, m as i32, v.as_mut_ptr())
        } else {
            let mut decodificada = Picture::nueva()?;
            decodificada.0.use_argb = 1;
            let tiene_alfa = w::WebPPictureHasTransparency(&pic.0) != 0;
            importar_webp(webp, &mut decodificada.0, tiene_alfa)?;
            w::WebPPictureDistortion(&decodificada.0, &pic.0, m as i32, v.as_mut_ptr())
        };
        if ok == 0 {
            return Err(Error::Codificacion("no se pudo medir la distorsión".into()));
        }
    }
    Ok(v)
}

/// `DumpPicture` de cwebp.c: los planos YUV(A) en un PGM con disposición IMC4.
unsafe fn volcar_pgm(p: &w::WebPPicture) -> Vec<u8> {
    unsafe {
        let uv_w = ((p.width + 1) / 2) as usize;
        let uv_h = ((p.height + 1) / 2) as usize;
        let paso = ((p.width + 1) & !1) as usize;
        let alto_alfa = if w::WebPPictureHasTransparency(p) != 0 {
            p.height as usize
        } else {
            0
        };
        let alto = p.height as usize + uv_h + alto_alfa;
        let mut s = format!("P5\n{paso} {alto}\n255\n").into_bytes();
        let ancho = p.width as usize;
        let fila = |ptr: *const u8, n: usize| std::slice::from_raw_parts(ptr, n);
        for y in 0..p.height as usize {
            s.extend_from_slice(fila(p.y.add(y * p.y_stride as usize), ancho));
            if ancho & 1 == 1 {
                s.push(0);
            }
        }
        for y in 0..uv_h {
            s.extend_from_slice(fila(p.u.add(y * p.uv_stride as usize), uv_w));
            s.extend_from_slice(fila(p.v.add(y * p.uv_stride as usize), uv_w));
        }
        for y in 0..alto_alfa {
            s.extend_from_slice(fila(p.a.add(y * p.a_stride as usize), ancho));
            if ancho & 1 == 1 {
                s.push(0);
            }
        }
        s
    }
}
