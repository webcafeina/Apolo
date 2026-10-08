//! AVIF y JPEG XL con las herramientas oficiales compiladas dentro: avifenc y
//! avifdec de libavif 1.4.2, cjxl y djxl de libjxl 0.12.0 (ADR 0021).

// Para que se enlacen: el JPEG y el sharpyuv que usan las herramientas.
use libwebp_sys as _;
use mozjpeg_sys as _;

use std::ffi::{CString, c_char, c_int};
use std::sync::Mutex;

unsafe extern "C" {
    fn apolo_avifenc(argc: c_int, argv: *mut *mut c_char, callado: c_int) -> c_int;
    fn apolo_avifdec(argc: c_int, argv: *mut *mut c_char, callado: c_int) -> c_int;
    fn apolo_cjxl(argc: c_int, argv: *mut *mut c_char, callado: c_int) -> c_int;
    fn apolo_djxl(argc: c_int, argv: *mut *mut c_char, callado: c_int) -> c_int;
}

/// Una herramienta compilada dentro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Herramienta {
    Avifenc,
    Avifdec,
    Cjxl,
    Djxl,
}

impl Herramienta {
    pub fn nombre(self) -> &'static str {
        match self {
            Herramienta::Avifenc => "avifenc",
            Herramienta::Avifdec => "avifdec",
            Herramienta::Cjxl => "cjxl",
            Herramienta::Djxl => "djxl",
        }
    }
}

// Los main de las herramientas no están hechos para ejecutarse a la vez en el
// mismo proceso; cada uno reparte su trabajo en hilos por su cuenta.
static AVIF: Mutex<()> = Mutex::new(());
static JXL: Mutex<()> = Mutex::new(());

/// Ejecuta la herramienta con estos argumentos (sin el nombre del programa),
/// como si fuera su binario. Devuelve su código de salida.
pub fn ejecutar(herramienta: Herramienta, argumentos: &[String]) -> i32 {
    ejecutar_con(herramienta, argumentos, false)
}

/// Lo mismo; con `callado`, lo que escriba por la salida estándar se tira.
fn ejecutar_con(herramienta: Herramienta, argumentos: &[String], callado: bool) -> i32 {
    let mut cadenas: Vec<CString> = Vec::with_capacity(argumentos.len() + 1);
    cadenas.push(CString::new(herramienta.nombre()).unwrap());
    for a in argumentos {
        cadenas.push(CString::new(a.as_bytes()).unwrap_or_else(|_| CString::new("").unwrap()));
    }
    let mut punteros: Vec<*mut c_char> =
        cadenas.iter().map(|c| c.as_ptr() as *mut c_char).collect();
    punteros.push(std::ptr::null_mut());
    let argc = cadenas.len() as c_int;
    let argv = punteros.as_mut_ptr();
    let candado = match herramienta {
        Herramienta::Avifenc | Herramienta::Avifdec => &AVIF,
        Herramienta::Cjxl | Herramienta::Djxl => &JXL,
    };
    let _guarda = candado.lock().unwrap_or_else(|e| e.into_inner());
    let callado = c_int::from(callado);
    // SAFETY: argv son cadenas terminadas en cero que viven hasta el final, con
    // el puntero nulo detrás, como el argv de un main.
    unsafe {
        match herramienta {
            Herramienta::Avifenc => apolo_avifenc(argc, argv, callado),
            Herramienta::Avifdec => apolo_avifdec(argc, argv, callado),
            Herramienta::Cjxl => apolo_cjxl(argc, argv, callado),
            Herramienta::Djxl => apolo_djxl(argc, argv, callado),
        }
    }
}

/// Una carpeta temporal para los ficheros de entrada y salida de una
/// herramienta, que se borra al soltarla.
struct Temporal(std::path::PathBuf);

impl Temporal {
    fn nueva() -> Result<Temporal, String> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static CUENTA: AtomicU64 = AtomicU64::new(0);
        let n = CUENTA.fetch_add(1, Ordering::Relaxed);
        let d = std::env::temp_dir().join(format!("apolo-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&d).map_err(|e| format!("carpeta temporal: {e}"))?;
        Ok(Temporal(d))
    }

    fn ruta(&self, nombre: &str) -> String {
        self.0.join(nombre).to_string_lossy().into_owned()
    }
}

impl Drop for Temporal {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Ejecuta la herramienta con `entrada` como fichero de entrada (con la
/// extensión `ext`, que avifenc mira para saber el formato) y devuelve el
/// fichero de salida. Los argumentos van delante de los dos ficheros, salvo en
/// cjxl y djxl, que los quieren detrás (también los aceptan delante, pero así
/// es como se escriben).
pub fn convertir(
    herramienta: Herramienta,
    entrada: &[u8],
    ext: &str,
    argumentos: &[String],
    ext_salida: &str,
) -> Result<Vec<u8>, String> {
    let t = Temporal::nueva()?;
    let (e, s) = (
        t.ruta(&format!("entrada.{ext}")),
        t.ruta(&format!("salida.{ext_salida}")),
    );
    std::fs::write(&e, entrada).map_err(|x| format!("fichero temporal: {x}"))?;
    let mut todos: Vec<String> = Vec::with_capacity(argumentos.len() + 2);
    match herramienta {
        Herramienta::Avifenc | Herramienta::Avifdec => {
            todos.extend_from_slice(argumentos);
            todos.push(e);
            todos.push(s.clone());
        }
        Herramienta::Cjxl | Herramienta::Djxl => {
            todos.push(e);
            todos.push(s.clone());
            todos.extend_from_slice(argumentos);
        }
    }
    // cjxl y djxl se callan con --quiet; avifenc y avifdec, con el puente.
    if matches!(herramienta, Herramienta::Cjxl | Herramienta::Djxl) {
        todos.push("--quiet".into());
    }
    let codigo = ejecutar_con(herramienta, &todos, true);
    if codigo != 0 {
        return Err(format!(
            "{} terminó con el código {codigo}",
            herramienta.nombre()
        ));
    }
    std::fs::read(&s).map_err(|x| format!("{} no escribió la salida: {x}", herramienta.nombre()))
}

/// Si son los bytes de un AVIF: la caja ftyp con la marca avif o avis, de
/// principal o entre las compatibles.
pub fn es_avif(datos: &[u8]) -> bool {
    if datos.len() < 16 || &datos[4..8] != b"ftyp" {
        return false;
    }
    let caja =
        (u32::from_be_bytes([datos[0], datos[1], datos[2], datos[3]]) as usize).min(datos.len());
    let marca = |m: &[u8; 4]| m == b"avif" || m == b"avis";
    marca(datos[8..12].try_into().unwrap())
        || datos
            .get(16..caja)
            .is_some_and(|c| c.as_chunks::<4>().0.iter().any(marca))
}

/// Si son los bytes de un JPEG XL: el código desnudo o el contenedor.
pub fn es_jxl(datos: &[u8]) -> bool {
    datos.starts_with(&[0xFF, 0x0A])
        || datos.starts_with(&[
            0, 0, 0, 0x0C, b'J', b'X', b'L', b' ', 0x0D, 0x0A, 0x87, 0x0A,
        ])
}

/// Un AVIF a PNG con avifdec, con su perfil y sus metadatos. `ocho_bits`
/// fuerza 8 bits por canal (para enseñarlo).
pub fn avif_a_png(datos: &[u8], ocho_bits: bool) -> Result<Vec<u8>, String> {
    let args: Vec<String> = if ocho_bits {
        vec!["-d".into(), "8".into()]
    } else {
        Vec::new()
    };
    convertir(Herramienta::Avifdec, datos, "avif", &args, "png")
}

/// Un JPEG XL a PNG con djxl, con su perfil y sus metadatos.
pub fn jxl_a_png(datos: &[u8], ocho_bits: bool) -> Result<Vec<u8>, String> {
    let args: Vec<String> = if ocho_bits {
        vec!["--bits_per_sample=8".into()]
    } else {
        Vec::new()
    };
    convertir(Herramienta::Djxl, datos, "jxl", &args, "png")
}

unsafe extern "C" {
    fn avifVersion() -> *const c_char;
    fn aom_codec_version_str() -> *const c_char;
    fn JxlEncoderVersion() -> u32;
}

/// Las versiones de libavif, aom y libjxl enlazadas, para «Acerca de».
pub fn versiones() -> [(&'static str, String); 3] {
    let texto = |p: *const c_char| {
        // SAFETY: las dos devuelven una cadena estática terminada en cero.
        unsafe { std::ffi::CStr::from_ptr(p) }
            .to_string_lossy()
            .trim_start_matches('v')
            .to_string()
    };
    // SAFETY: funciones puras de las bibliotecas, sin argumentos ni estado.
    let (avif, aom, jxl) = unsafe { (avifVersion(), aom_codec_version_str(), JxlEncoderVersion()) };
    [
        ("libavif", texto(avif)),
        ("aom", texto(aom)),
        (
            "libjxl",
            format!("{}.{}.{}", jxl / 1_000_000, jxl / 1000 % 1000, jxl % 1000),
        ),
    ]
}

unsafe extern "C" {
    fn apolo_ssimulacra2(
        original: *const u8,
        distorsionada: *const u8,
        ancho: usize,
        alto: usize,
        canales: c_int,
    ) -> f64;
}

/// La nota SSIMULACRA 2 de libjxl (de −∞ a 100; 100 es idéntica) de
/// `distorsionada` frente a `original`: píxeles sRGB de 8 bits, del mismo
/// tamaño, con 3 canales (RGB) o 4 (RGBA). Con transparencia, la peor nota
/// sobre fondo oscuro y claro, como la herramienta `ssimulacra2`. `None` si
/// no se puede medir (menos de 8×8, o tamaños que no cuadran).
pub fn ssimulacra2(
    original: &[u8],
    distorsionada: &[u8],
    ancho: u32,
    alto: u32,
    canales: u8,
) -> Option<f64> {
    let n = ancho as usize * alto as usize * canales as usize;
    if original.len() != n || distorsionada.len() != n {
        return None;
    }
    // SAFETY: los dos búferes tienen exactamente ancho × alto × canales bytes,
    // y la función solo los lee.
    let nota = unsafe {
        apolo_ssimulacra2(
            original.as_ptr(),
            distorsionada.as_ptr(),
            ancho as usize,
            alto as usize,
            c_int::from(canales),
        )
    };
    (!nota.is_nan()).then_some(nota)
}
