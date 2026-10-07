//! `cjpeg` de MozJPEG 4.1.5, trasladado a Rust sobre `mozjpeg-sys`.
//!
//! No es «algo parecido a cjpeg»: es su `main` y su `parse_switches`, paso por
//! paso, con las funciones de `rdswitch.c` que usa (`set_quality_ratings`,
//! `set_quant_slots`, `set_sample_factors` y su propio `jpeg_default_qtables`,
//! que con el ABI 62 vive en cjpeg y no en la librería). Así sale el mismo
//! fichero, byte a byte (ADR 0020). Lo que cambia es solo la frontera: la
//! entrada llega ya leída ([`super::EntradaJpeg`]) y la salida va a memoria.
//!
//! Lo raro que hay que repetir tal cual:
//! - las opciones se leen **dos veces**: una antes de conocer la imagen y otra
//!   después de `jpeg_default_colorspace`, sobre el mismo `cinfo`;
//! - `-revert` llama a `jpeg_set_defaults` y deshace lo anterior que vive en
//!   `cinfo`, pero no lo que vive en variables del propio cjpeg (`-quality`,
//!   `-sample`, `-qslots`, `-progressive`, `-baseline`);
//! - `-quality` de 90 en adelante quita el submuestreo del color (1x1), y de
//!   80 en adelante lo deja en 2x1;
//! - el factor de escala de calidad se guarda en un entero: se trunca.

use std::ffi::c_int;
use std::panic::{AssertUnwindSafe, catch_unwind};

use mozjpeg_sys as j;

use super::EntradaJpeg;
use super::tablas::{CROMINANCIA, LUMINANCIA};
use crate::{Error, Resultado};

const JCP_FASTEST: c_int = 0x2AEA_5CB4;
const NUM_QUANT_TBLS: usize = 4;
const MAX_COMPONENTS: usize = 10;

/// Lo que cjpeg recibe además de las opciones: el perfil de `-icc` y si los
/// avisos son errores (`-strict`).
#[derive(Debug, Clone, Default)]
pub struct Extra {
    pub icc: Option<Vec<u8>>,
    pub estricto: bool,
}

/// Variables de `parse_switches` que no viven en `cinfo`.
struct Locales {
    force_baseline: bool,
    simple_progressive: bool,
    calidad: Option<String>,
    ranuras: Option<String>,
    muestreo: Option<String>,
    /// El `q_scale_factor` estático de rdswitch.c (ABI 62).
    escala: [c_int; NUM_QUANT_TBLS],
}

/// `keymatch` de cdjpeg.c: `arg` es un prefijo de `palabra` de al menos
/// `minimo` letras, sin distinguir mayúsculas en `arg`.
fn keymatch(arg: &str, palabra: &str, minimo: usize) -> bool {
    let mut p = palabra.bytes();
    let mut n = 0;
    for a in arg.bytes() {
        let Some(k) = p.next() else { return false };
        if a.to_ascii_lowercase() != k {
            return false;
        }
        n += 1;
    }
    n >= minimo
}

/// `atoi`: el entero del principio, y lo demás se ignora.
fn atoi(s: &str) -> c_int {
    let t = s.trim_start();
    let fin = t
        .char_indices()
        .take_while(|(i, c)| c.is_ascii_digit() || (*i == 0 && (*c == '-' || *c == '+')))
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);
    t[..fin].parse::<i64>().unwrap_or(0) as c_int
}

/// `atof`: el número del principio, y lo demás se ignora.
fn atof(s: &str) -> f64 {
    let t = s.trim_start();
    let mut mejor = 0.0;
    for (i, c) in t.char_indices() {
        if let Ok(v) = t[..i + c.len_utf8()].parse::<f64>() {
            mejor = v;
        } else if !matches!(c, '-' | '+' | '.' | 'e' | 'E') && !c.is_ascii_digit() {
            break;
        }
    }
    mejor
}

/// `sscanf("%ld%c")` de `-restart` y `-maxmemory`: el número y la letra que
/// le siga, si hay.
fn numero_y_letra(s: &str) -> Option<(i64, Option<char>)> {
    let t = s.trim_start();
    let fin = t
        .char_indices()
        .take_while(|(i, c)| c.is_ascii_digit() || (*i == 0 && (*c == '-' || *c == '+')))
        .last()
        .map(|(i, c)| i + c.len_utf8())?;
    let n = t[..fin].parse::<i64>().ok()?;
    Some((n, t[fin..].chars().next()))
}

/// Los trozos de «A,B,C» como los recorre rdswitch.c: una coma al final no
/// abre un trozo vacío (el bucle se para al llegar al final de la cadena).
fn trozos(arg: &str) -> Vec<&str> {
    if arg.is_empty() {
        return vec![];
    }
    let mut v: Vec<&str> = arg.split(',').collect();
    if v.len() > 1 && v.last() == Some(&"") {
        v.pop();
    }
    v
}

fn mal(texto: impl Into<String>) -> Error {
    Error::Configuracion(texto.into())
}

/// `set_quality_ratings`: «N[,N,...]»; si hay menos que tablas, se repite el
/// último.
unsafe fn calidad(
    c: &mut j::jpeg_compress_struct,
    l: &mut Locales,
    arg: &str,
    force_baseline: bool,
) -> Resultado<()> {
    let mut val: f32 = 75.0;
    let partes = trozos(arg);
    for tblno in 0..NUM_QUANT_TBLS {
        if let Some(p) = partes.get(tblno) {
            // sscanf("%f%c") con la coma como única terminación válida: un
            // trozo que no empiece por número es un error.
            let t = p.trim_start();
            if t.is_empty() || !t.starts_with(|ch: char| ch.is_ascii_digit() || "+-.".contains(ch))
            {
                return Err(mal(format!("-quality {arg}: no es una lista de números")));
            }
            let num_fin = t
                .char_indices()
                .take_while(|(_, ch)| ch.is_ascii_digit() || "+-.eE".contains(*ch))
                .last()
                .map(|(i, ch)| i + ch.len_utf8())
                .unwrap_or(0);
            if num_fin != t.len() {
                return Err(mal(format!("-quality {arg}: no es una lista de números")));
            }
            val = atof(t) as f32;
        }
        // q_scale_factor es int: la conversión de C trunca.
        l.escala[tblno] = unsafe { j::jpeg_float_quality_scaling(val) } as c_int;
    }
    unsafe { tablas_por_defecto(c, l, force_baseline) };
    if val >= 90.0 {
        unsafe { muestreo(c, "1x1")? };
    } else if val >= 80.0 {
        unsafe { muestreo(c, "2x1")? };
    }
    Ok(())
}

/// El `jpeg_default_qtables` propio de rdswitch.c para el ABI 62.
unsafe fn tablas_por_defecto(c: &mut j::jpeg_compress_struct, l: &Locales, force_baseline: bool) {
    let mut indice = 0;
    if unsafe { j::jpeg_c_int_param_supported(c, j::JINT_BASE_QUANT_TBL_IDX) } != 0 {
        indice = unsafe { j::jpeg_c_get_int_param(c, j::JINT_BASE_QUANT_TBL_IDX) } as usize;
    }
    let fb = force_baseline as j::boolean;
    unsafe {
        j::jpeg_add_quant_table(c, 0, LUMINANCIA[indice].as_ptr(), l.escala[0], fb);
        j::jpeg_add_quant_table(c, 1, CROMINANCIA[indice].as_ptr(), l.escala[1], fb);
    }
}

/// `set_quant_slots`: «N[,N,...]», tablas 0 a 3; si faltan, se repite la última.
unsafe fn ranuras(c: &mut j::jpeg_compress_struct, arg: &str) -> Resultado<()> {
    let partes = trozos(arg);
    let mut val = 0;
    for ci in 0..MAX_COMPONENTS {
        if let Some(p) = partes.get(ci) {
            if p.trim().is_empty() || p.trim().parse::<i32>().is_err() {
                return Err(mal(format!("-qslots {arg}: no es una lista de números")));
            }
            val = atoi(p);
            if !(0..NUM_QUANT_TBLS as c_int).contains(&val) {
                return Err(mal("Las tablas de cuantización van de 0 a 3"));
            }
        }
        unsafe { (*c.comp_info.add(ci)).quant_tbl_no = val };
    }
    Ok(())
}

/// `set_sample_factors`: «HxV[,HxV,...]», de 1 a 4; si faltan, 1x1.
unsafe fn muestreo(c: &mut j::jpeg_compress_struct, arg: &str) -> Resultado<()> {
    let partes = trozos(arg);
    for ci in 0..MAX_COMPONENTS {
        let (h, v) = match partes.get(ci) {
            Some(p) => {
                let (h, v) = p
                    .split_once(['x', 'X'])
                    .ok_or_else(|| mal(format!("-sample {arg}: tiene que ser HxV")))?;
                let (h, v) = (h.trim().parse::<c_int>(), v.trim().parse::<c_int>());
                let (Ok(h), Ok(v)) = (h, v) else {
                    return Err(mal(format!("-sample {arg}: tiene que ser HxV")));
                };
                if !(1..=4).contains(&h) || !(1..=4).contains(&v) {
                    return Err(mal("Los factores de muestreo van de 1 a 4"));
                }
                (h, v)
            }
            None => (1, 1),
        };
        unsafe {
            let ci = &mut *c.comp_info.add(ci);
            ci.h_samp_factor = h;
            ci.v_samp_factor = v;
        }
    }
    Ok(())
}

/// `parse_switches`. Las rutas (`-icc`, `-outfile`) y lo que solo es de la
/// línea de órdenes (`-verbose`, `-report`…) ya los ha quitado quien llama.
unsafe fn opciones(
    c: &mut j::jpeg_compress_struct,
    args: &[String],
    de_verdad: bool,
    l: &mut Locales,
) -> Resultado<()> {
    l.force_baseline = false;
    l.simple_progressive = c.num_scans != 0;
    l.calidad = None;
    l.ranuras = None;
    l.muestreo = None;

    let mut n = 0;
    let siguiente = |n: &mut usize, que: &str| -> Resultado<String> {
        *n += 1;
        args.get(*n)
            .cloned()
            .ok_or_else(|| mal(format!("falta el valor de -{que}")))
    };
    while n < args.len() {
        let Some(arg) = args[n].strip_prefix('-') else {
            return Err(mal(format!("«{}» no es una opción", args[n])));
        };
        unsafe {
            if keymatch(arg, "arithmetic", 1) {
                // El cjpeg 4.1.5 oficial viene sin codificación aritmética.
                return Err(mal("cjpeg no tiene codificación aritmética (-arithmetic)"));
            } else if keymatch(arg, "baseline", 1) {
                l.force_baseline = true;
                l.simple_progressive = false;
                c.num_scans = 0;
                c.scan_info = std::ptr::null();
            } else if keymatch(arg, "dct", 2) {
                let v = siguiente(&mut n, "dct")?;
                c.dct_method = if keymatch(&v, "int", 1) {
                    j::J_DCT_METHOD::JDCT_ISLOW
                } else if keymatch(&v, "fast", 2) {
                    j::J_DCT_METHOD::JDCT_IFAST
                } else if keymatch(&v, "float", 2) {
                    j::J_DCT_METHOD::JDCT_FLOAT
                } else {
                    return Err(mal(format!("-dct {v}: tiene que ser int, fast o float")));
                };
            } else if keymatch(arg, "fastcrush", 4) {
                j::jpeg_c_set_bool_param(c, j::JBOOLEAN_OPTIMIZE_SCANS, 0);
            } else if keymatch(arg, "grayscale", 2) || keymatch(arg, "greyscale", 2) {
                j::jpeg_set_colorspace(c, j::J_COLOR_SPACE::JCS_GRAYSCALE);
            } else if keymatch(arg, "rgb", 3) {
                j::jpeg_set_colorspace(c, j::J_COLOR_SPACE::JCS_RGB);
            } else if keymatch(arg, "lambda1", 7) {
                let v = siguiente(&mut n, "lambda1")?;
                j::jpeg_c_set_float_param(c, j::JFLOAT_LAMBDA_LOG_SCALE1, atof(&v) as f32);
            } else if keymatch(arg, "lambda2", 7) {
                let v = siguiente(&mut n, "lambda2")?;
                j::jpeg_c_set_float_param(c, j::JFLOAT_LAMBDA_LOG_SCALE2, atof(&v) as f32);
            } else if keymatch(arg, "dc-scan-opt", 3) {
                let v = siguiente(&mut n, "dc-scan-opt")?;
                j::jpeg_c_set_int_param(c, j::JINT_DC_SCAN_OPT_MODE, atoi(&v));
            } else if keymatch(arg, "optimize", 1) || keymatch(arg, "optimise", 1) {
                c.optimize_coding = 1;
            } else if keymatch(arg, "progressive", 1) {
                l.simple_progressive = true;
            } else if keymatch(arg, "quality", 1) {
                l.calidad = Some(siguiente(&mut n, "quality")?);
            } else if keymatch(arg, "qslots", 2) {
                l.ranuras = Some(siguiente(&mut n, "qslots")?);
            } else if keymatch(arg, "quant-table", 7) {
                let v = atoi(&siguiente(&mut n, "quant-table")?);
                j::jpeg_c_set_int_param(c, j::JINT_BASE_QUANT_TBL_IDX, v);
                if j::jpeg_c_get_int_param(c, j::JINT_BASE_QUANT_TBL_IDX) != v {
                    return Err(mal(format!("-quant-table {v}: tiene que ir de 0 a 8")));
                }
                j::jpeg_set_quality(c, 75, 1);
            } else if keymatch(arg, "quant-baseline", 7) {
                l.force_baseline = true;
            } else if keymatch(arg, "restart", 1) {
                let v = siguiente(&mut n, "restart")?;
                let (num, letra) = numero_y_letra(&v)
                    .filter(|(num, _)| (0..=65535).contains(num))
                    .ok_or_else(|| mal(format!("-restart {v}: tiene que ir de 0 a 65535")))?;
                if matches!(letra, Some('b' | 'B')) {
                    c.restart_interval = num as u32;
                    c.restart_in_rows = 0;
                } else {
                    c.restart_in_rows = num as c_int;
                }
            } else if keymatch(arg, "revert", 3) {
                j::jpeg_c_set_int_param(c, j::JINT_COMPRESS_PROFILE, JCP_FASTEST);
                j::jpeg_set_defaults(c);
            } else if keymatch(arg, "sample", 2) {
                l.muestreo = Some(siguiente(&mut n, "sample")?);
            } else if keymatch(arg, "smooth", 2) {
                let v = siguiente(&mut n, "smooth")?;
                let s = v
                    .trim()
                    .parse::<c_int>()
                    .ok()
                    .filter(|s| (0..=100).contains(s))
                    .ok_or_else(|| mal(format!("-smooth {v}: tiene que ir de 0 a 100")))?;
                c.smoothing_factor = s;
            } else if keymatch(arg, "notrellis-dc", 11) {
                j::jpeg_c_set_bool_param(c, j::JBOOLEAN_TRELLIS_QUANT_DC, 0);
            } else if keymatch(arg, "notrellis", 1) {
                j::jpeg_c_set_bool_param(c, j::JBOOLEAN_TRELLIS_QUANT, 0);
            } else if keymatch(arg, "trellis-dc-ver-weight", 12) {
                let v = siguiente(&mut n, "trellis-dc-ver-weight")?;
                j::jpeg_c_set_float_param(c, j::JFLOAT_TRELLIS_DELTA_DC_WEIGHT, atof(&v) as f32);
            } else if keymatch(arg, "trellis-dc", 9) {
                j::jpeg_c_set_bool_param(c, j::JBOOLEAN_TRELLIS_QUANT_DC, 1);
            } else if keymatch(arg, "tune-psnr", 6) {
                afinar(c, 1, 9.0, 0.0, false);
            } else if keymatch(arg, "tune-ssim", 6) {
                afinar(c, 1, 11.5, 12.75, false);
            } else if keymatch(arg, "tune-ms-ssim", 6) {
                afinar(c, 3, 12.0, 13.0, true);
            } else if keymatch(arg, "tune-hvs-psnr", 6) {
                afinar(c, 3, 14.75, 16.5, true);
            } else if keymatch(arg, "noovershoot", 11) {
                j::jpeg_c_set_bool_param(c, j::JBOOLEAN_OVERSHOOT_DERINGING, 0);
            } else if keymatch(arg, "nojfif", 6) {
                c.write_JFIF_header = 0;
            } else {
                return Err(mal(format!("cjpeg no tiene la opción -{arg}")));
            }
        }
        n += 1;
    }

    if de_verdad {
        let fb = l.force_baseline;
        if let Some(q) = l.calidad.clone() {
            unsafe { calidad(c, l, &q, fb)? };
        }
        if let Some(r) = l.ranuras.clone() {
            unsafe { ranuras(c, &r)? };
        }
        if let Some(m) = l.muestreo.clone() {
            unsafe { muestreo(c, &m)? };
        }
        if l.simple_progressive {
            unsafe { j::jpeg_simple_progression(c) };
        }
    }
    Ok(())
}

unsafe fn afinar(c: &mut j::jpeg_compress_struct, tabla: c_int, l1: f32, l2: f32, pesos: bool) {
    unsafe {
        j::jpeg_c_set_int_param(c, j::JINT_BASE_QUANT_TBL_IDX, tabla);
        j::jpeg_c_set_float_param(c, j::JFLOAT_LAMBDA_LOG_SCALE1, l1);
        j::jpeg_c_set_float_param(c, j::JFLOAT_LAMBDA_LOG_SCALE2, l2);
        j::jpeg_c_set_bool_param(c, j::JBOOLEAN_USE_LAMBDA_WEIGHT_TBL, pesos as j::boolean);
        j::jpeg_set_quality(c, 75, 1);
    }
}

// ---------------------------------------------------------------- errores

/// libjpeg avisa de los errores llamando a `error_exit`, que por defecto
/// termina el proceso. Aquí se convierte en un pánico con el texto, que
/// `catch_unwind` recoge (como hace el crate mozjpeg al leer).
unsafe extern "C-unwind" fn salir(cinfo: &mut j::jpeg_common_struct) {
    let texto = unsafe { mensaje(cinfo) };
    std::panic::resume_unwind(Box::new(texto));
}

/// Los avisos no se escriben; se cuentan, y con `-strict` son errores.
unsafe extern "C-unwind" fn avisar(cinfo: &mut j::jpeg_common_struct, nivel: c_int) {
    if nivel < 0 {
        unsafe {
            (*cinfo.err).num_warnings += 1;
            if ESTRICTO.with(|e| e.get()) {
                salir(cinfo);
            }
        }
    }
}

thread_local! {
    static ESTRICTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

unsafe fn mensaje(cinfo: &mut j::jpeg_common_struct) -> String {
    let buf = [0u8; 80];
    unsafe {
        if let Some(f) = (*cinfo.err).format_message {
            f(cinfo, &buf);
        }
    }
    let fin = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..fin]).into_owned()
}

// ------------------------------------------------------------------ main

/// Comprime `entrada` como lo haría `cjpeg <args> entrada`.
pub fn ejecutar(entrada: &EntradaJpeg, args: &[String], extra: &Extra) -> Resultado<Vec<u8>> {
    ESTRICTO.with(|e| e.set(extra.estricto));
    let r = catch_unwind(AssertUnwindSafe(|| unsafe {
        ejecutar_sin_proteger(entrada, args, extra)
    }));
    match r {
        Ok(r) => r,
        Err(p) => Err(Error::Codificacion(
            p.downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "MozJPEG falló".into()),
        )),
    }
}

/// La memoria de `jpeg_mem_dest`, que libjpeg reserva con malloc.
struct Destino {
    ptr: *mut u8,
    tam: std::ffi::c_ulong,
}

impl Drop for Destino {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // SAFETY: lo reservó libjpeg con malloc.
            unsafe { libc::free(self.ptr.cast()) };
        }
    }
}

/// El objeto de compresión, que se destruye pase lo que pase.
struct Compresor(Box<j::jpeg_compress_struct>);

impl Drop for Compresor {
    fn drop(&mut self) {
        // SAFETY: creado con jpeg_create_compress; destruir aborta lo que haya.
        unsafe { j::jpeg_destroy_compress(&mut self.0) };
    }
}

unsafe fn ejecutar_sin_proteger(
    e: &EntradaJpeg,
    args: &[String],
    extra: &Extra,
) -> Resultado<Vec<u8>> {
    // El gestor de errores tiene que vivir más que el compresor: va en una caja
    // que se suelta después (orden de los `let`, al revés al salir).
    let mut err: Box<j::jpeg_error_mgr> = Box::new(unsafe { std::mem::zeroed() });
    unsafe { j::jpeg_std_error(&mut err) };
    err.error_exit = Some(salir);
    err.emit_message = Some(avisar);
    let mut destino = Destino {
        ptr: std::ptr::null_mut(),
        tam: 0,
    };
    let mut c = Compresor(Box::new(unsafe { std::mem::zeroed() }));
    c.0.common.err = &mut *err;
    unsafe { j::jpeg_create_compress(&mut *c.0) };
    let c = &mut *c.0;

    let mut l = Locales {
        force_baseline: false,
        simple_progressive: false,
        calidad: None,
        ranuras: None,
        muestreo: None,
        escala: [100; NUM_QUANT_TBLS],
    };

    c.in_color_space = j::J_COLOR_SPACE::JCS_RGB;
    unsafe { j::jpeg_set_defaults(c) };
    unsafe { opciones(c, args, false, &mut l)? };

    // start_input del lector: lo que sabe de la imagen.
    if e.ancho == 0 || e.alto == 0 || e.ancho > 65535 || e.alto > 65535 {
        return Err(mal("JPEG admite de 1 a 65535 píxeles por lado"));
    }
    c.in_color_space = if e.gris {
        j::J_COLOR_SPACE::JCS_GRAYSCALE
    } else {
        j::J_COLOR_SPACE::JCS_RGB
    };
    c.input_components = if e.gris { 1 } else { 3 };
    c.data_precision = 8;
    c.image_width = e.ancho;
    c.image_height = e.alto;

    unsafe { j::jpeg_default_colorspace(c) };
    unsafe { opciones(c, args, true, &mut l)? };

    unsafe { j::jpeg_mem_dest(c, &mut destino.ptr, &mut destino.tam) };
    unsafe { j::jpeg_start_compress(c, 1) };

    for (marcador, datos) in &e.marcadores {
        let jfif = *marcador == 0xE0 && datos.len() >= 5 && datos[..5] == *b"JFIF\0";
        let adobe = *marcador == 0xEE && datos.len() >= 5 && datos[..5] == *b"Adobe";
        if (c.write_JFIF_header != 0 && jfif) || (c.write_Adobe_marker != 0 && adobe) {
            continue;
        }
        unsafe { j::jpeg_write_marker(c, *marcador as c_int, datos.as_ptr(), datos.len() as u32) };
    }
    if let Some(icc) = &extra.icc {
        unsafe { j::jpeg_write_icc_profile(c, icc.as_ptr(), icc.len() as u32) };
    }

    let paso = e.ancho as usize * if e.gris { 1 } else { 3 };
    while c.next_scanline < c.image_height {
        let fila = &e.filas[c.next_scanline as usize * paso..][..paso];
        let mut puntero = fila.as_ptr();
        unsafe { j::jpeg_write_scanlines(c, (&mut puntero as *mut *const u8).cast(), 1) };
    }
    unsafe { j::jpeg_finish_compress(c) };

    // SAFETY: jpeg_mem_dest dejó ahí tam bytes.
    Ok(unsafe { std::slice::from_raw_parts(destino.ptr, destino.tam as usize) }.to_vec())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn keymatch_como_cdjpeg() {
        assert!(keymatch("q", "quality", 1));
        assert!(keymatch("QUAL", "quality", 1));
        assert!(!keymatch("qualityx", "quality", 1));
        assert!(!keymatch("d", "dct", 2));
        assert!(keymatch("dc", "dct", 2)); // y no dc-scan-opt: va antes en la cadena
        assert!(!keymatch("notrellis", "notrellis-dc", 11));
    }

    #[test]
    fn atoi_y_atof_como_c() {
        assert_eq!(atoi("12abc"), 12);
        assert_eq!(atoi("  -3"), -3);
        assert_eq!(atoi("x"), 0);
        assert_eq!(atof("9.5x"), 9.5);
        assert_eq!(atof("1e2"), 100.0);
        assert_eq!(numero_y_letra("16B"), Some((16, Some('B'))));
        assert_eq!(numero_y_letra("4"), Some((4, None)));
    }
}
