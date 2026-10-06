//! Leer HEIC, el formato de las fotos del iPhone, con libheif y libde265
//! compiladas dentro (build.rs).
//!
//! Solo lo que hace falta para abrir la imagen principal: sus píxeles en RGB(A)
//! de 8 bits y sus metadatos (EXIF, ICC y XMP). libheif aplica al decodificar
//! los giros y volteos que guarda el fichero (`irot`, `imir`), así que los
//! píxeles salen **ya derechos**. Si el EXIF dice además otra orientación, hay
//! que dejarla en 1, o Apolo la giraría dos veces: eso lo hace el núcleo.
//!
//! Los enlaces están escritos a mano, solo para las funciones que se usan,
//! contra las cabeceras de libheif 1.23 (vendor/libheif/libheif/api/libheif).

use std::ffi::{CStr, c_char, c_int, c_void};
use std::sync::Once;

#[repr(C)]
#[derive(Clone, Copy)]
struct HeifError {
    code: c_int,
    subcode: c_int,
    message: *const c_char,
}

type Contexto = c_void;
type Asa = c_void;
type Imagen = c_void;

const COLOR_RGB: c_int = 1; // heif_colorspace_RGB
const CROMA_RGB: c_int = 10; // heif_chroma_interleaved_RGB
const CROMA_RGBA: c_int = 11; // heif_chroma_interleaved_RGBA
const CANAL_INTERCALADO: c_int = 10; // heif_channel_interleaved

unsafe extern "C" {
    fn heif_init(params: *const c_void) -> HeifError;
    fn heif_context_alloc() -> *mut Contexto;
    fn heif_context_free(ctx: *mut Contexto);
    fn heif_context_read_from_memory_without_copy(
        ctx: *mut Contexto,
        mem: *const c_void,
        size: usize,
        options: *const c_void,
    ) -> HeifError;
    fn heif_context_get_primary_image_handle(ctx: *mut Contexto, asa: *mut *mut Asa) -> HeifError;
    fn heif_image_handle_release(asa: *const Asa);
    fn heif_image_handle_has_alpha_channel(asa: *const Asa) -> c_int;
    fn heif_decode_image(
        asa: *const Asa,
        imagen: *mut *mut Imagen,
        espacio: c_int,
        croma: c_int,
        opciones: *const c_void,
    ) -> HeifError;
    fn heif_image_get_plane_readonly2(
        imagen: *const Imagen,
        canal: c_int,
        paso: *mut usize,
    ) -> *const u8;
    fn heif_image_get_width(imagen: *const Imagen, canal: c_int) -> c_int;
    fn heif_image_get_height(imagen: *const Imagen, canal: c_int) -> c_int;
    fn heif_image_release(imagen: *const Imagen);
    fn heif_image_handle_get_number_of_metadata_blocks(
        asa: *const Asa,
        filtro: *const c_char,
    ) -> c_int;
    fn heif_image_handle_get_list_of_metadata_block_IDs(
        asa: *const Asa,
        filtro: *const c_char,
        ids: *mut u32,
        cuantos: c_int,
    ) -> c_int;
    fn heif_image_handle_get_metadata_content_type(asa: *const Asa, id: u32) -> *const c_char;
    fn heif_image_handle_get_metadata_size(asa: *const Asa, id: u32) -> usize;
    fn heif_image_handle_get_metadata(asa: *const Asa, id: u32, salida: *mut c_void) -> HeifError;
    fn heif_image_handle_get_raw_color_profile_size(asa: *const Asa) -> usize;
    fn heif_image_handle_get_raw_color_profile(asa: *const Asa, salida: *mut c_void) -> HeifError;
}

/// Una foto HEIC leída.
#[derive(Debug, Clone)]
pub struct Heic {
    pub ancho: u32,
    pub alto: u32,
    /// RGBA si tiene transparencia, RGB si no; 8 bits, ya derecha.
    pub pixeles: Vec<u8>,
    pub alfa: bool,
    /// El EXIF desde la cabecera TIFF (sin el desplazamiento que lleva delante
    /// en HEIF), como lo dan los lectores de JPEG.
    pub exif: Option<Vec<u8>>,
    pub icc: Option<Vec<u8>>,
    pub xmp: Option<Vec<u8>>,
}

/// Si los bytes son un HEIF con HEVC dentro. Mira la caja `ftyp`: la marca
/// principal, o `mif1`/`msf1` con una marca HEVC entre las compatibles. Un
/// AVIF también es HEIF (marca `avif`), pero con AV1: ése no.
pub fn es_heic(datos: &[u8]) -> bool {
    const HEVC: [&[u8; 4]; 6] = [b"heic", b"heix", b"heim", b"heis", b"hevc", b"hevx"];
    if datos.len() < 16 || &datos[4..8] != b"ftyp" {
        return false;
    }
    let caja = u32::from_be_bytes([datos[0], datos[1], datos[2], datos[3]]) as usize;
    let principal = &datos[8..12];
    if HEVC.iter().any(|m| principal == *m) {
        return true;
    }
    if principal != b"mif1" && principal != b"msf1" {
        return false;
    }
    let fin = caja.min(datos.len());
    datos
        .get(16..fin)
        .is_some_and(|c| c.as_chunks::<4>().0.iter().any(|m| HEVC.contains(&m)))
}

static INICIO: Once = Once::new();

fn fallo(e: HeifError) -> Result<(), String> {
    if e.code == 0 {
        return Ok(());
    }
    // SAFETY: libheif garantiza un mensaje válido mientras viva el objeto que
    // dio el error; se copia ya.
    let m = if e.message.is_null() {
        String::new()
    } else {
        unsafe { CStr::from_ptr(e.message) }
            .to_string_lossy()
            .into_owned()
    };
    Err(if m.is_empty() {
        format!("error {}.{}", e.code, e.subcode)
    } else {
        m
    })
}

/// Libera lo de libheif al salir, pase lo que pase.
struct Recursos {
    ctx: *mut Contexto,
    asa: *mut Asa,
    imagen: *mut Imagen,
}

impl Drop for Recursos {
    fn drop(&mut self) {
        // SAFETY: cada puntero o es nulo o lo dio libheif y no se ha liberado.
        unsafe {
            if !self.imagen.is_null() {
                heif_image_release(self.imagen);
            }
            if !self.asa.is_null() {
                heif_image_handle_release(self.asa);
            }
            if !self.ctx.is_null() {
                heif_context_free(self.ctx);
            }
        }
    }
}

/// Lee la imagen principal de un HEIC.
pub fn leer(datos: &[u8]) -> Result<Heic, String> {
    INICIO.call_once(|| {
        // SAFETY: sin parámetros; registra los decodificadores compilados dentro.
        let _ = unsafe { heif_init(std::ptr::null()) };
    });
    let mut r = Recursos {
        // SAFETY: reserva un contexto nuevo.
        ctx: unsafe { heif_context_alloc() },
        asa: std::ptr::null_mut(),
        imagen: std::ptr::null_mut(),
    };
    if r.ctx.is_null() {
        return Err("sin memoria".into());
    }
    // SAFETY: `datos` vive más que el contexto (se libera en `r` antes de
    // volver), que es lo que exige leer «sin copia».
    unsafe {
        fallo(heif_context_read_from_memory_without_copy(
            r.ctx,
            datos.as_ptr().cast(),
            datos.len(),
            std::ptr::null(),
        ))?;
        fallo(heif_context_get_primary_image_handle(r.ctx, &mut r.asa))?;
        let alfa = heif_image_handle_has_alpha_channel(r.asa) != 0;
        fallo(heif_decode_image(
            r.asa,
            &mut r.imagen,
            COLOR_RGB,
            if alfa { CROMA_RGBA } else { CROMA_RGB },
            std::ptr::null(),
        ))?;
        let ancho = heif_image_get_width(r.imagen, CANAL_INTERCALADO);
        let alto = heif_image_get_height(r.imagen, CANAL_INTERCALADO);
        let mut paso = 0usize;
        let plano = heif_image_get_plane_readonly2(r.imagen, CANAL_INTERCALADO, &mut paso);
        if plano.is_null() || ancho <= 0 || alto <= 0 {
            return Err("la imagen no tiene píxeles".into());
        }
        let (ancho, alto) = (ancho as usize, alto as usize);
        let canales = if alfa { 4 } else { 3 };
        let fila = ancho * canales;
        let mut pixeles = Vec::with_capacity(fila * alto);
        for y in 0..alto {
            pixeles.extend_from_slice(std::slice::from_raw_parts(plano.add(y * paso), fila));
        }

        Ok(Heic {
            ancho: ancho as u32,
            alto: alto as u32,
            pixeles,
            alfa,
            exif: exif(r.asa),
            icc: icc(r.asa),
            xmp: xmp(r.asa),
        })
    }
}

/// Los bloques de metadatos de un tipo (`Exif`, `mime`).
unsafe fn bloques(asa: *const Asa, tipo: &CStr) -> Vec<u32> {
    unsafe {
        let n = heif_image_handle_get_number_of_metadata_blocks(asa, tipo.as_ptr());
        if n <= 0 {
            return vec![];
        }
        let mut ids = vec![0u32; n as usize];
        let n = heif_image_handle_get_list_of_metadata_block_IDs(
            asa,
            tipo.as_ptr(),
            ids.as_mut_ptr(),
            n,
        );
        ids.truncate(n.max(0) as usize);
        ids
    }
}

unsafe fn bloque(asa: *const Asa, id: u32) -> Option<Vec<u8>> {
    unsafe {
        let n = heif_image_handle_get_metadata_size(asa, id);
        if n == 0 {
            return None;
        }
        let mut v = vec![0u8; n];
        fallo(heif_image_handle_get_metadata(
            asa,
            id,
            v.as_mut_ptr().cast(),
        ))
        .ok()?;
        Some(v)
    }
}

/// En HEIF, el bloque EXIF lleva delante 4 bytes con el desplazamiento hasta la
/// cabecera TIFF (normalmente 6: lo que ocupa «Exif\0\0»).
unsafe fn exif(asa: *const Asa) -> Option<Vec<u8>> {
    unsafe {
        let id = *bloques(asa, c"Exif").first()?;
        let b = bloque(asa, id)?;
        let desp = u32::from_be_bytes(b.get(..4)?.try_into().ok()?) as usize;
        b.get(4 + desp..)
            .filter(|t| !t.is_empty())
            .map(<[u8]>::to_vec)
    }
}

unsafe fn xmp(asa: *const Asa) -> Option<Vec<u8>> {
    unsafe {
        for id in bloques(asa, c"mime") {
            let tipo = heif_image_handle_get_metadata_content_type(asa, id);
            if !tipo.is_null() && CStr::from_ptr(tipo).to_bytes() == b"application/rdf+xml" {
                return bloque(asa, id);
            }
        }
        None
    }
}

unsafe fn icc(asa: *const Asa) -> Option<Vec<u8>> {
    unsafe {
        let n = heif_image_handle_get_raw_color_profile_size(asa);
        if n == 0 {
            return None;
        }
        let mut v = vec![0u8; n];
        fallo(heif_image_handle_get_raw_color_profile(
            asa,
            v.as_mut_ptr().cast(),
        ))
        .ok()?;
        Some(v)
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn ftyp(principal: &[u8; 4], compatibles: &[&[u8; 4]]) -> Vec<u8> {
        let n = 16 + 4 * compatibles.len();
        let mut v = (n as u32).to_be_bytes().to_vec();
        v.extend(b"ftyp");
        v.extend(principal);
        v.extend([0, 0, 0, 0]);
        for c in compatibles {
            v.extend(*c);
        }
        v.extend([0; 16]);
        v
    }

    #[test]
    fn reconoce_heic_y_no_avif() {
        assert!(es_heic(&ftyp(b"heic", &[b"mif1", b"heic"])));
        assert!(es_heic(&ftyp(b"mif1", &[b"mif1", b"heic"])));
        assert!(!es_heic(&ftyp(b"avif", &[b"mif1", b"avif"])));
        assert!(!es_heic(&ftyp(b"mif1", &[b"mif1", b"avif"])));
        assert!(!es_heic(b"\x89PNG\r\n\x1a\n0000000000"));
    }

    #[test]
    fn un_fichero_roto_no_revienta() {
        let mut v = ftyp(b"heic", &[b"heic"]);
        v.extend([7u8; 64]);
        assert!(leer(&v).is_err());
    }
}

#[cfg(test)]
mod con_foto {
    /// La foto de ejemplo de libheif (vendor/libheif/examples/example.heic).
    #[test]
    fn decodifica_la_foto_de_ejemplo() {
        let ruta = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("vendor/libheif/examples/example.heic");
        let datos = std::fs::read(ruta).unwrap();
        assert!(super::es_heic(&datos));
        let h = super::leer(&datos).unwrap();
        assert!(h.ancho > 100 && h.alto > 100, "{}×{}", h.ancho, h.alto);
        let canales = if h.alfa { 4 } else { 3 };
        assert_eq!(h.pixeles.len(), (h.ancho * h.alto) as usize * canales);
        // No es una imagen plana: hay variedad de colores.
        let distintos: std::collections::HashSet<_> =
            h.pixeles.chunks(canales * 97).map(|p| p[0]).collect();
        assert!(distintos.len() > 20);
    }
}
