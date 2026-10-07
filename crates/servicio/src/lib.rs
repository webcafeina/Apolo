//! Lo que hace la ventana, sin ventana: el Estudio (abrir imágenes, codificar
//! la vista previa, exportar), los lotes, los presets y los ajustes.
//!
//! Lo usan dos caras (ADR 0013): la aplicación de Tauri (`src-tauri`) y el
//! servidor de desarrollo por HTTP (`crates/dev`), que permite probar la
//! interfaz entera en un navegador sin compilar la ventana. Las dos llaman a
//! estas mismas funciones; ninguna tiene lógica propia.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use apolo_nucleo::entrada::{self, Imagen, Lectura, Pixeles};
use apolo_nucleo::presets::{self, PresetGuardado};
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida, Motivo};
use apolo_nucleo::webp::{self, Estadisticas, OpcionesWebp};
use apolo_nucleo::{Error, cwebp, orientacion, vista};
use serde::{Deserialize, Serialize};

mod ajustes;
mod lotes;
pub use ajustes::Ajustes;
pub use lotes::{EstadoLote, Fila, FilaSalida, LoteEmpezado, Recogida, recoger_lote};

/// Un error para la interfaz: el texto que hay que enseñar, y si fue una
/// cancelación (que no se enseña).
#[derive(Debug, Clone, Serialize)]
pub struct Fallo {
    pub mensaje: String,
    pub cancelado: bool,
}

impl From<Error> for Fallo {
    fn from(e: Error) -> Self {
        Fallo {
            cancelado: matches!(e, Error::Cancelado),
            mensaje: e.to_string(),
        }
    }
}

impl Fallo {
    fn nuevo(m: impl Into<String>) -> Self {
        Fallo {
            mensaje: m.into(),
            cancelado: false,
        }
    }
}

pub type R<T> = Result<T, Fallo>;

/// Lo que la interfaz sabe de una imagen abierta.
#[derive(Debug, Clone, Serialize)]
pub struct InfoImagen {
    pub id: u64,
    pub nombre: String,
    pub formato: &'static str,
    pub ancho: u32,
    pub alto: u32,
    /// Lo que pesa el fichero original.
    pub bytes: u64,
    pub alfa: bool,
    pub exif: Option<usize>,
    pub icc: Option<usize>,
    pub xmp: Option<usize>,
    /// Orientación EXIF, 1–8. Distinta de 1: la foto se ve girada si no se
    /// endereza (ADR 0012).
    pub orientacion: u8,
}

/// El resultado de una vista previa.
#[derive(Debug, Clone, Serialize)]
pub struct Vista {
    pub generacion: u64,
    /// El lado del comparador (0, izquierdo; 1, derecho).
    pub lado: u8,
    pub formato: FormatoSalida,
    /// cwebp, cjpeg, oxipng o qoiconv: la herramienta de la orden.
    pub herramienta: &'static str,
    pub bytes: usize,
    pub ancho: u32,
    pub alto: u32,
    pub milisegundos: u64,
    /// Solo WebP las da.
    pub estadisticas: Option<Estadisticas>,
    /// La orden de la herramienta, con el nombre de la imagen: la que se ve.
    pub orden: String,
    /// La misma con las rutas completas: la que se copia, para que funcione
    /// pegada en cualquier carpeta. Sin ruta de disco, igual que `orden`.
    pub orden_completa: String,
    /// Si esa orden da exactamente este fichero.
    pub equivalente: bool,
    /// Por qué no lo da, si no lo da (la interfaz lo explica).
    pub motivo: Option<Motivo>,
}

struct Abierta {
    nombre: String,
    /// La ruta de disco, si se abrió de disco (en desarrollo llega por HTTP y
    /// no la hay). Sirve para la orden copiable con rutas completas.
    ruta: Option<PathBuf>,
    /// Los bytes del fichero: cjpeg, oxipng y qoiconv leen el fichero, no
    /// los píxeles de Apolo (ADR 0020).
    datos: Vec<u8>,
    imagen: Imagen,
}

#[derive(Default)]
struct Ultimo {
    ajuste: Option<Ajuste>,
    datos: Vec<u8>,
    rgba: (u32, u32, Vec<u8>),
}

/// Imagen y lado del comparador.
type Clave = (u64, u8);

/// El estado del Estudio.
pub struct Servicio {
    abiertas: Mutex<HashMap<u64, Arc<Abierta>>>,
    ultimos: Mutex<HashMap<Clave, Arc<Mutex<Ultimo>>>>,
    siguiente: AtomicU64,
    /// La generación más reciente pedida **para cada imagen y lado**. Una
    /// codificación en marcha con una generación anterior se cancela en el
    /// siguiente aviso de progreso. Va por imagen y no global: si la interfaz
    /// se recarga, vuelve a contar desde 1, y con un contador global todas sus
    /// peticiones parecerían viejas (docs/trampas.md).
    generaciones: Mutex<HashMap<Clave, Arc<AtomicU64>>>,
    carpeta_presets: PathBuf,
    ajustes: ajustes::Almacen,
    lotes: Mutex<HashMap<u64, Arc<lotes::Lote>>>,
}

impl Servicio {
    /// `carpeta` es la de configuración de Apolo: dentro van `presets/` y
    /// `ajustes.json`.
    pub fn nuevo(carpeta: PathBuf) -> Self {
        Servicio {
            abiertas: Mutex::default(),
            ultimos: Mutex::default(),
            siguiente: AtomicU64::new(1),
            generaciones: Mutex::default(),
            carpeta_presets: carpeta.join("presets"),
            ajustes: ajustes::Almacen::abrir(carpeta.join("ajustes.json")),
            lotes: Mutex::default(),
        }
    }

    /// Con la carpeta de configuración de siempre (la misma que la CLI).
    pub fn con_carpeta_por_defecto() -> Self {
        Servicio::nuevo(presets::carpeta_config().unwrap_or_else(|| PathBuf::from("apolo")))
    }

    pub fn ajustes(&self) -> Ajustes {
        self.ajustes.leer()
    }

    /// La casilla «Avisarme cuando haya una versión nueva».
    pub fn buscar_actualizaciones(&self, si: bool) -> Ajustes {
        self.ajustes.buscar_actualizaciones(si)
    }

    /// Si toca preguntar a GitHub por una versión nueva (ADR 0018).
    pub fn reservar_comprobacion(&self, forzar: bool) -> bool {
        self.ajustes.reservar_comprobacion(forzar)
    }

    fn abierta(&self, id: u64) -> R<Arc<Abierta>> {
        self.abiertas
            .lock()
            .unwrap()
            .get(&id)
            .cloned()
            .ok_or_else(|| Fallo::nuevo("Esa imagen ya no está abierta"))
    }

    fn ultimo(&self, clave: Clave) -> Arc<Mutex<Ultimo>> {
        self.ultimos
            .lock()
            .unwrap()
            .entry(clave)
            .or_default()
            .clone()
    }

    /// Abre una imagen de disco.
    pub fn abrir_ruta(&self, ruta: &Path) -> R<InfoImagen> {
        let datos = std::fs::read(ruta)
            .map_err(|e| Fallo::nuevo(format!("No se puede leer «{}»: {e}", ruta.display())))?;
        let nombre = ruta
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ruta = std::path::absolute(ruta).unwrap_or_else(|_| ruta.to_path_buf());
        self.abrir(&nombre, Some(ruta), datos)
    }

    /// Abre una imagen de memoria (lo que sube el navegador en desarrollo).
    pub fn abrir_bytes(&self, nombre: &str, datos: Vec<u8>) -> R<InfoImagen> {
        self.abrir(nombre, None, datos)
    }

    fn abrir(&self, nombre: &str, ruta: Option<PathBuf>, datos: Vec<u8>) -> R<InfoImagen> {
        // Los metadatos se leen siempre, para enseñarlos; si están rotos, se
        // abre sin ellos en vez de no abrir.
        let imagen = entrada::leer(&datos, Lectura::default()).or_else(|_| {
            entrada::leer(
                &datos,
                Lectura {
                    conservar_alfa: true,
                    metadatos: false,
                },
            )
        })?;
        let id = self.siguiente.fetch_add(1, Ordering::Relaxed);
        let alfa = matches!(imagen.pixeles, Pixeles::Rgba(_))
            || matches!(&imagen.pixeles, Pixeles::WebP(d) if webp_tiene_alfa(d));
        let m = &imagen.metadatos;
        let info = InfoImagen {
            id,
            nombre: nombre.into(),
            formato: imagen.formato.nombre(),
            ancho: imagen.ancho,
            alto: imagen.alto,
            bytes: datos.len() as u64,
            alfa,
            exif: m.exif.as_ref().map(Vec::len),
            icc: m.icc.as_ref().map(Vec::len),
            xmp: m.xmp.as_ref().map(Vec::len),
            orientacion: orientacion::leer(m.exif.as_deref()),
        };
        self.abiertas.lock().unwrap().insert(
            id,
            Arc::new(Abierta {
                nombre: nombre.into(),
                ruta,
                datos,
                imagen,
            }),
        );
        Ok(info)
    }

    /// Cierra una imagen y libera su memoria.
    pub fn cerrar(&self, id: u64) {
        self.abiertas.lock().unwrap().remove(&id);
        self.ultimos.lock().unwrap().retain(|(i, _), _| *i != id);
        self.generaciones
            .lock()
            .unwrap()
            .retain(|(i, _), _| *i != id);
    }

    /// Los píxeles del original, enderezados o no.
    pub fn pixeles_original(&self, id: u64, enderezar: bool) -> R<(u32, u32, Vec<u8>)> {
        let a = self.abierta(id)?;
        let enderezada = if enderezar {
            orientacion::enderezar(&a.imagen)?
        } else {
            None
        };
        Ok(vista::rgba(enderezada.as_ref().unwrap_or(&a.imagen))?)
    }

    /// Los píxeles de la última vista previa de un lado.
    pub fn pixeles_resultado(&self, id: u64, lado: u8) -> R<(u32, u32, Vec<u8>)> {
        let u = self.ultimo((id, lado));
        let u = u.lock().unwrap();
        if u.ajuste.is_none() {
            return Err(Fallo::nuevo("Todavía no hay resultado"));
        }
        Ok(u.rgba.clone())
    }

    /// Codifica la vista previa de un lado del comparador. `generacion` crece
    /// con cada cambio de ese lado; lo que se esté codificando con una
    /// anterior se cancela.
    pub fn codificar(&self, id: u64, ajuste: &Ajuste, lado: u8, generacion: u64) -> R<Vista> {
        let a = self.abierta(id)?;
        let ultima = self
            .generaciones
            .lock()
            .unwrap()
            .entry((id, lado))
            .or_default()
            .clone();
        ultima.fetch_max(generacion, Ordering::SeqCst);
        let actual = ultima.clone();
        let mut seguir = move |_p: i32| actual.load(Ordering::SeqCst) == generacion;

        let reloj = Instant::now();
        let r = salida::codificar(&a.datos, &a.imagen, ajuste, Some(&mut seguir))?;
        let milisegundos = reloj.elapsed().as_millis() as u64;
        if ultima.load(Ordering::SeqCst) != generacion {
            return Err(Error::Cancelado.into());
        }
        let rgba = vista::decodificar(&r.datos)?;

        let nombre = nombre_salida(&a.nombre, ajuste.formato);
        let motivo = salida::motivo(a.imagen.formato, ajuste);
        let v = Vista {
            generacion,
            lado,
            formato: ajuste.formato,
            herramienta: ajuste.formato.herramienta(),
            bytes: r.datos.len(),
            ancho: r.ancho,
            alto: r.alto,
            milisegundos,
            estadisticas: r.estadisticas,
            orden: salida::orden(ajuste, &a.nombre, &nombre),
            orden_completa: match &a.ruta {
                Some(r) => salida::orden(
                    ajuste,
                    &r.display().to_string(),
                    &r.with_file_name(&nombre).display().to_string(),
                ),
                None => salida::orden(ajuste, &a.nombre, &nombre),
            },
            equivalente: motivo.is_none(),
            motivo,
        };
        let u = self.ultimo((id, lado));
        *u.lock().unwrap() = Ultimo {
            ajuste: Some(ajuste.clone()),
            datos: r.datos,
            rgba,
        };
        Ok(v)
    }

    /// Los bytes del fichero final: el de la última vista previa de ese lado
    /// si el ajuste no ha cambiado, o uno nuevo.
    pub fn bytes_finales(&self, id: u64, ajuste: &Ajuste, lado: u8) -> R<Vec<u8>> {
        let a = self.abierta(id)?;
        let u = self.ultimo((id, lado));
        {
            let u = u.lock().unwrap();
            if u.ajuste.as_ref() == Some(ajuste) {
                return Ok(u.datos.clone());
            }
        }
        Ok(salida::codificar(&a.datos, &a.imagen, ajuste, None)?.datos)
    }

    /// Escribe el fichero final.
    pub fn exportar(&self, id: u64, ajuste: &Ajuste, lado: u8, ruta: &Path) -> R<usize> {
        let datos = self.bytes_finales(id, ajuste, lado)?;
        std::fs::write(ruta, &datos)
            .map_err(|e| Fallo::nuevo(format!("No se puede escribir «{}»: {e}", ruta.display())))?;
        Ok(datos.len())
    }

    /// El nombre que se propone al exportar.
    pub fn nombre_salida(&self, id: u64, formato: FormatoSalida) -> R<String> {
        Ok(nombre_salida(&self.abierta(id)?.nombre, formato))
    }

    pub fn presets(&self) -> Vec<PresetGuardado> {
        presets::listar(&self.carpeta_presets)
    }

    pub fn guardar_preset(&self, p: &PresetGuardado) -> R<Vec<PresetGuardado>> {
        if p.nombre.trim().is_empty() {
            return Err(Fallo::nuevo("El preset necesita un nombre"));
        }
        presets::guardar(&self.carpeta_presets, p)
            .map_err(|e| Fallo::nuevo(format!("No se pudo guardar: {e}")))?;
        Ok(self.presets())
    }

    pub fn borrar_preset(&self, nombre: &str) -> R<Vec<PresetGuardado>> {
        presets::borrar(&self.carpeta_presets, nombre)
            .map_err(|e| Fallo::nuevo(format!("No se pudo borrar: {e}")))?;
        Ok(self.presets())
    }

    pub fn carpeta_presets(&self) -> &Path {
        &self.carpeta_presets
    }
}

/// Las opciones de un preset de cwebp, partiendo de las actuales (como
/// `-preset`: reescribe el resto y conserva la calidad).
pub fn aplicar_preset_cwebp(opciones: &OpcionesWebp, preset: webp::Preset) -> OpcionesWebp {
    let mut o = opciones.clone();
    o.aplicar_preset(preset);
    o
}

/// `-z nivel` sobre las opciones actuales: sin pérdida con el método y la
/// calidad de ese nivel (0–9), con la tabla de libwebp.
pub fn nivel_sin_perdida(opciones: &OpcionesWebp, nivel: i32) -> R<OpcionesWebp> {
    let mut o = opciones.clone();
    if !o.aplicar_nivel_sin_perdida(nivel) {
        return Err(Fallo::nuevo(format!(
            "Nivel sin pérdida no válido: {nivel}"
        )));
    }
    Ok(o)
}

/// Una orden pegada por quien usa la interfaz: de cwebp, cjpeg, oxipng,
/// qoiconv, avifenc o cjxl (o `apolo webp`, `apolo jpeg`…), con o sin ficheros.
/// Devuelve el ajuste que da: el formato y sus opciones, sobre `base` (lo
/// demás no se toca). Sin herramienta delante, se lee como del formato de
/// `base`.
pub fn leer_orden(texto: &str, base: &Ajuste) -> R<Ajuste> {
    let mut palabras = partir(texto);
    let es = |p: &str, h: &str| {
        p == h || p.ends_with(&format!("/{h}")) || p.ends_with(&format!("{h}.exe"))
    };
    let formato = match palabras.first().map(String::as_str) {
        Some(p) if es(p, "cwebp") => Some(FormatoSalida::Webp),
        Some(p) if es(p, "cjpeg") => Some(FormatoSalida::Jpeg),
        Some(p) if es(p, "oxipng") => Some(FormatoSalida::Png),
        Some(p) if es(p, "qoiconv") => Some(FormatoSalida::Qoi),
        Some(p) if es(p, "avifenc") => Some(FormatoSalida::Avif),
        Some(p) if es(p, "cjxl") => Some(FormatoSalida::Jxl),
        Some("apolo") => match palabras.get(1).map(String::as_str) {
            Some("webp") => Some(FormatoSalida::Webp),
            Some("jpeg") => Some(FormatoSalida::Jpeg),
            Some("png") => Some(FormatoSalida::Png),
            Some("qoi") => Some(FormatoSalida::Qoi),
            Some("avif") => Some(FormatoSalida::Avif),
            Some("jxl") => Some(FormatoSalida::Jxl),
            _ => None,
        },
        _ => None,
    };
    match (formato, palabras.first().map(String::as_str)) {
        (Some(_), Some("apolo")) => {
            palabras.drain(..2);
        }
        (Some(_), _) => {
            palabras.remove(0);
        }
        _ => {}
    }
    let mut a = base.clone();
    a.formato = formato.unwrap_or(base.formato);
    match a.formato {
        FormatoSalida::Webp => {
            a.webp = cwebp::leer(&palabras)
                .map_err(|e| Fallo::nuevo(e.0))?
                .opciones;
        }
        FormatoSalida::Jpeg => {
            a.jpeg = apolo_nucleo::jpeg::opciones::leer_orden(&palabras)
                .map_err(Fallo::nuevo)?
                .opciones;
        }
        FormatoSalida::Png => {
            a.png = apolo_nucleo::formatos::png::leer_orden(&palabras)?.opciones;
        }
        FormatoSalida::Qoi => {
            if palabras.len() > 2 {
                return Err(Fallo::nuevo("qoiconv solo lleva la entrada y la salida"));
            }
        }
        FormatoSalida::Avif => {
            a.avif = apolo_nucleo::formatos::avif::leer_orden(&palabras)?.opciones;
        }
        FormatoSalida::Jxl => {
            a.jxl = apolo_nucleo::formatos::jxl::leer_orden(&palabras)?.opciones;
        }
    }
    Ok(a)
}

/// Parte una orden en palabras como lo haría una shell sencilla: espacios,
/// comillas simples y dobles.
fn partir(texto: &str) -> Vec<String> {
    let mut v = Vec::new();
    let mut actual = String::new();
    let mut hay = false;
    let mut comilla: Option<char> = None;
    for c in texto.chars() {
        match (comilla, c) {
            (Some(q), c) if c == q => comilla = None,
            (Some(_), c) => actual.push(c),
            (None, '\'' | '"') => {
                comilla = Some(c);
                hay = true;
            }
            (None, c) if c.is_whitespace() => {
                if hay || !actual.is_empty() {
                    v.push(std::mem::take(&mut actual));
                    hay = false;
                }
            }
            (None, '\\') => {}
            (None, c) => actual.push(c),
        }
    }
    if hay || !actual.is_empty() {
        v.push(actual);
    }
    v
}

/// El nombre que se propone para el fichero de salida. Si el original ya
/// tiene esa extensión, se le añade «-apolo»: con el mismo nombre, exportar
/// o la orden de la herramienta sobrescribirían el original sin avisar.
fn nombre_salida(nombre: &str, formato: FormatoSalida) -> String {
    let p = Path::new(nombre);
    let base = p
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "imagen".into());
    let ext = formato.extension();
    let misma = p.extension().is_some_and(|e| {
        e.eq_ignore_ascii_case(ext)
            || (formato == FormatoSalida::Jpeg && e.eq_ignore_ascii_case("jpeg"))
    });
    if misma {
        format!("{base}-apolo.{ext}")
    } else {
        format!("{base}.{ext}")
    }
}

/// La orden de la herramienta que reproduce un ajuste (sin entrada ni
/// salida), para enseñarla junto a cada preset y en Lotes.
pub fn orden_opciones(ajuste: &Ajuste) -> String {
    let a = salida::argumentos(ajuste);
    if a.is_empty() {
        ajuste.formato.herramienta().to_string()
    } else {
        format!("{} {}", ajuste.formato.herramienta(), a.join(" "))
    }
}

fn webp_tiene_alfa(datos: &[u8]) -> bool {
    let mut r = std::mem::MaybeUninit::<libwebp_sys::WebPBitstreamFeatures>::zeroed();
    // SAFETY: búfer propio y estructura propia.
    unsafe {
        libwebp_sys::WebPGetFeaturesInternal(
            datos.as_ptr(),
            datos.len(),
            r.as_mut_ptr(),
            libwebp_sys::WEBP_DECODER_ABI_VERSION as i32,
        ) == libwebp_sys::VP8StatusCode::VP8_STATUS_OK
            && r.assume_init().has_alpha != 0
    }
}

/// Los píxeles en el formato que espera la interfaz: ancho y alto en u32
/// little-endian, y detrás el RGBA.
pub fn empaquetar_pixeles((ancho, alto, rgba): (u32, u32, Vec<u8>)) -> Vec<u8> {
    let mut v = Vec::with_capacity(8 + rgba.len());
    v.extend(ancho.to_le_bytes());
    v.extend(alto.to_le_bytes());
    v.extend(rgba);
    v
}

/// Las opciones por defecto, para que la interfaz no las tenga que saber.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inicio {
    pub opciones: OpcionesWebp,
    /// El ajuste por defecto (WebP, sin proceso), con todas las opciones.
    pub ajuste: Ajuste,
    pub presets_cwebp: Vec<webp::Preset>,
    /// Dónde se guardan los presets con nombre, para decirlo en la interfaz.
    pub carpeta_presets: String,
    /// La versión de Apolo, para la firma y «Acerca de».
    pub version: &'static str,
}

impl Servicio {
    pub fn inicio(&self) -> Inicio {
        Inicio {
            opciones: OpcionesWebp::default(),
            ajuste: Ajuste::default(),
            presets_cwebp: webp::Preset::TODOS.to_vec(),
            carpeta_presets: self.carpeta_presets.display().to_string(),
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn partir_como_una_shell() {
        assert_eq!(
            partir("cwebp -q 80 'mi foto.png' -o x.webp"),
            ["cwebp", "-q", "80", "mi foto.png", "-o", "x.webp"]
        );
        assert_eq!(partir("  -lossless  "), ["-lossless"]);
        assert_eq!(partir(r#"-o "" a"#), ["-o", "", "a"]);
    }

    #[test]
    fn leer_orden_pegada() {
        let base = Ajuste::default();
        let a = leer_orden("cwebp -preset photo -q 82 foto.jpg -o foto.webp", &base).unwrap();
        assert_eq!(a.webp.calidad, 82.0);
        assert_eq!(a.webp.preset, Some(webp::Preset::Photo));
        let a = leer_orden("apolo webp -lossless -apolo_enderezar", &base).unwrap();
        assert!(a.webp.sin_perdida && a.webp.enderezar);
        assert!(leer_orden("cwebp -noexiste", &base).is_err());
        // Cada herramienta da su formato.
        let a = leer_orden("cjpeg -quality 70 -outfile x.jpg 'mi foto.png'", &base).unwrap();
        assert_eq!(
            (a.formato, a.jpeg.calidad.clone()),
            (FormatoSalida::Jpeg, vec![70.0])
        );
        let a = leer_orden("oxipng -o 4 --strip safe a.png", &base).unwrap();
        assert_eq!((a.formato, a.png.nivel), (FormatoSalida::Png, 4));
        assert_eq!(
            leer_orden("qoiconv a.png a.qoi", &base).unwrap().formato,
            FormatoSalida::Qoi
        );
        // Sin herramienta, las opciones del formato que ya había.
        let jpeg = Ajuste {
            formato: FormatoSalida::Jpeg,
            ..Default::default()
        };
        assert_eq!(
            leer_orden("-quality 50", &jpeg).unwrap().jpeg.calidad,
            vec![50.0]
        );
    }

    #[test]
    fn la_generacion_vieja_se_cancela() {
        let s = Servicio::nuevo(std::env::temp_dir().join("apolo-servicio"));
        let foto = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus/foto.webp");
        let info = s.abrir_ruta(&foto).unwrap();
        assert_eq!((info.ancho, info.alto, info.formato), (128, 128, "WebP"));
        let a = Ajuste::default();
        let v = s.codificar(info.id, &a, 0, 5).unwrap();
        assert_eq!(v.generacion, 5);
        assert!(v.equivalente);
        assert!(v.orden.starts_with("cwebp foto.webp"));
        // Un .webp no se propone con su mismo nombre: se sobrescribiría.
        assert!(v.orden.ends_with("-o foto-apolo.webp"), "{}", v.orden);
        // La orden para copiar lleva la ruta completa.
        assert!(
            v.orden_completa.contains("pruebas/corpus/foto.webp"),
            "{}",
            v.orden_completa
        );
        assert_eq!(v.motivo, None);
        // Una petición con generación anterior a la última ya no vale.
        let r = s.codificar(info.id, &a, 0, 3);
        assert!(r.is_err_and(|f| f.cancelado));
        // El otro lado lleva su propia cuenta.
        assert!(s.codificar(info.id, &a, 1, 1).is_ok());
        let (w, h, px) = s.pixeles_resultado(info.id, 0).unwrap();
        assert_eq!((w, h, px.len()), (128, 128, 128 * 128 * 4));
        // Exportar con las mismas opciones da los bytes de la vista previa.
        let b = s.bytes_finales(info.id, &a, 0).unwrap();
        assert_eq!(b.len(), v.bytes);
        // Otra imagen empieza su propia cuenta: su generación 1 no es vieja.
        let otra = s.abrir_ruta(&foto).unwrap();
        assert!(s.codificar(otra.id, &a, 0, 1).is_ok());
        // En JPEG, la orden es de cjpeg y el nombre lleva .jpg.
        let j = Ajuste {
            formato: FormatoSalida::Jpeg,
            ..Default::default()
        };
        let v = s.codificar(otra.id, &j, 1, 2).unwrap();
        assert_eq!(v.herramienta, "cjpeg");
        assert!(
            v.orden.ends_with("-outfile foto.jpg foto.webp"),
            "{}",
            v.orden
        );
        // cjpeg no lee WebP: la orden no da ese fichero.
        assert_eq!(v.motivo, Some(Motivo::FormatoSinHerramienta));
    }
}
