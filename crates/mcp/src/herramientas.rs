//! Las diez herramientas (ADR 0023), como funciones que no saben nada del
//! protocolo ni del transporte: reciben lo que pide el agente y devuelven una
//! `Respuesta` o un `ErrorMcp`. La capa de rmcp (lib.rs) solo las envuelve.
//!
//! Reglas comunes:
//! - Nunca se sobrescribe ni se borra: todo se escribe con
//!   `escribir_sin_pisar` (si el nombre existe, `-2`, `-3`…).
//! - Las rutas salen absolutas.
//! - Lo pesado (codificar, medir) va de uno en uno: medir una foto de 12 MP
//!   son 1,75 GB.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use apolo_nucleo::entrada::{self, Formato, Lectura};
use apolo_nucleo::formatos::png;
use apolo_nucleo::jpeg;
use apolo_nucleo::lote::{escribir_sin_pisar, salida_sugerida, sufijo};
use apolo_nucleo::medir::{self, Medidas, TipoMapa};
use apolo_nucleo::presets::{self, PresetGuardado};
use apolo_nucleo::proceso::{Proceso, Redimension};
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida, Hallada, Motivo};
use apolo_nucleo::{orientacion, vista};
use apolo_servicio::{InfoImagen, Servicio};
use base64::Engine;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::entrada::{EntradaFormato, FormatoMcp, NOTA_POR_DEFECTO};
use crate::errores::{ErrorMcp, FORMATOS_QUE_LEE, Hecho};
use crate::respuesta::{Imagen, Respuesta, ahorro, ahorro_pct, banda, coma, peso};
use crate::rutas::resolver;

/// El estado del servidor: el servicio de Apolo y el turno de lo pesado.
pub struct Contexto {
    pub servicio: Servicio,
    pesadas: Mutex<()>,
    lote: Mutex<Option<u64>>,
}

impl Contexto {
    /// Con la carpeta de configuración de Apolo (la de sus presets), o la de
    /// `APOLO_CONFIG` si está puesta (para las pruebas).
    pub fn nuevo() -> Contexto {
        let servicio = match std::env::var_os("APOLO_CONFIG") {
            Some(c) => Servicio::nuevo(PathBuf::from(c)),
            None => Servicio::con_carpeta_por_defecto(),
        };
        Contexto::con(servicio)
    }

    pub fn con(servicio: Servicio) -> Contexto {
        Contexto {
            servicio,
            pesadas: Mutex::new(()),
            lote: Mutex::new(None),
        }
    }

    fn turno(&self) -> std::sync::MutexGuard<'_, ()> {
        self.pesadas.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Cómo informa una herramienta larga de su progreso (de 0 a 100) y cómo se
/// entera de que la han cancelado.
pub struct Seguimiento<'a> {
    pub avisar: &'a (dyn Fn(i32) + Sync),
    pub cancelado: &'a AtomicBool,
}

impl Seguimiento<'_> {
    pub fn nada() -> Seguimiento<'static> {
        static NO: AtomicBool = AtomicBool::new(false);
        Seguimiento {
            avisar: &|_| {},
            cancelado: &NO,
        }
    }
}

/// Una imagen abierta en el servicio, que se cierra al soltarla (también si
/// algo falla por el camino).
struct Abierta<'a> {
    cx: &'a Contexto,
    info: InfoImagen,
    ruta: PathBuf,
    formato: Option<Formato>,
}

impl Drop for Abierta<'_> {
    fn drop(&mut self) {
        self.cx.servicio.cerrar(self.info.id);
    }
}

fn existe(texto: &str) -> Hecho<PathBuf> {
    let ruta = resolver(texto)?;
    if !ruta.exists() {
        return Err(
            ErrorMcp::nuevo("no_existe", format!("No existe «{}».", ruta.display()))
                .con_pista("Comprueba la ruta; usa la absoluta (se admite ~)."),
        );
    }
    Ok(ruta)
}

fn abrir<'a>(cx: &'a Contexto, texto: &str) -> Hecho<Abierta<'a>> {
    let ruta = existe(texto)?;
    if ruta.is_dir() {
        return Err(ErrorMcp::nuevo(
            "es_carpeta",
            format!("«{}» es una carpeta.", ruta.display()),
        )
        .con_pista("Para una carpeta, empezar_lote (o analizar_imagen para ver qué hay)."));
    }
    let info = cx.servicio.abrir_ruta(&ruta).map_err(|f| {
        if f.mensaje.contains("No se reconoce") {
            ErrorMcp::nuevo(
                "no_es_imagen",
                format!("«{}» no es una imagen que Apolo lea.", ruta.display()),
            )
            .con_pista(format!("Apolo lee {FORMATOS_QUE_LEE}."))
        } else {
            ErrorMcp::from(f)
        }
    })?;
    let formato = formato_de(&ruta);
    Ok(Abierta {
        cx,
        info,
        ruta,
        formato,
    })
}

/// El formato de un fichero por sus primeros bytes.
fn formato_de(ruta: &Path) -> Option<Formato> {
    use std::io::Read;
    let mut cabeza = vec![0u8; 4096];
    let n = std::fs::File::open(ruta).ok()?.read(&mut cabeza).ok()?;
    Formato::adivinar(&cabeza[..n])
}

/// Lee un fichero como RGBA (para medir y ver), con su orientación EXIF.
/// Ancho, alto y píxeles RGBA; la orientación EXIF; y la imagen leída.
type Leida = ((u32, u32, Vec<u8>), u8, apolo_nucleo::entrada::Imagen);

fn leer_rgba(ruta: &Path) -> Hecho<Leida> {
    let datos = std::fs::read(ruta)?;
    let img = entrada::leer(&datos, Lectura::default())
        .or_else(|_| {
            entrada::leer(
                &datos,
                Lectura {
                    conservar_alfa: true,
                    metadatos: false,
                },
            )
        })
        .map_err(|e| match e {
            apolo_nucleo::Error::FormatoDesconocido => ErrorMcp::nuevo(
                "no_es_imagen",
                format!("«{}» no es una imagen que Apolo lea.", ruta.display()),
            )
            .con_pista(format!("Apolo lee {FORMATOS_QUE_LEE}.")),
            otro => ErrorMcp::from(otro),
        })?;
    let o = orientacion::leer(img.metadatos.exif.as_deref());
    Ok((vista::rgba(&img)?, o, img))
}

fn texto_motivo(m: Motivo, herramienta: &str, formato: &str) -> String {
    match m {
        Motivo::Enderezada => {
            format!("Apolo la ha enderezado según su EXIF, y {herramienta} no gira")
        }
        Motivo::Procesada => {
            format!("Apolo la ha redimensionado antes, y {herramienta} no lo hace igual")
        }
        Motivo::FormatoSinHerramienta => {
            format!("{herramienta} no lee {formato}: Apolo la ha convertido antes")
        }
    }
}

fn segundos(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else {
        format!("{} s", coma(ms as f64 / 1000.0, 1))
    }
}

fn linea_hallada(h: &Hallada, objetivo: f32) -> String {
    if h.alcanzada {
        format!(
            "Calidad {} encontrada para la nota {} (SSIMULACRA 2 {} · {} pruebas)",
            h.calidad,
            coma(objetivo as f64, 0),
            coma(h.nota, 1),
            h.pruebas
        )
    } else {
        format!(
            "No llega a la nota {} ni con calidad 100: se queda en 100 (SSIMULACRA 2 {}). Este formato no da más con esta imagen: prueba otros con comparar_formatos, o sin_perdida",
            coma(objetivo as f64, 0),
            coma(h.nota, 1)
        )
    }
}

fn linea_medidas(m: &Medidas) -> String {
    let nota = match m.ssimulacra2 {
        Some(n) => format!("SSIMULACRA 2 {} ({})", coma(n, 1), banda(n)),
        None => "SSIMULACRA 2: la imagen es menor de 8×8".into(),
    };
    let psnr = match m.psnr {
        Some(p) => format!("PSNR {} dB", coma(p, 1)),
        None => "PSNR ∞ (idénticas)".into(),
    };
    format!("{nota} · {psnr} · SSIM {}", coma(m.ssim, 4))
}

// ─── 1. analizar_imagen ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaAnalizar {
    /// Ruta absoluta de una imagen o de una carpeta (se admite ~).
    pub ruta: String,
}

#[derive(Serialize)]
struct DatosImagen {
    ruta: String,
    formato: &'static str,
    ancho: u32,
    alto: u32,
    megapixeles: f64,
    bytes: u64,
    alfa: bool,
    exif: Option<usize>,
    icc: Option<usize>,
    xmp: Option<usize>,
    orientacion: u8,
}

pub fn analizar_imagen(cx: &Contexto, e: EntradaAnalizar) -> Hecho<Respuesta> {
    let ruta = existe(&e.ruta)?;
    if ruta.is_dir() {
        let r = apolo_servicio::recoger_lote(&[ruta.display().to_string()]);
        let mut texto = format!(
            "{} · carpeta con {} {} · {}",
            ruta.display(),
            r.imagenes,
            if r.imagenes == 1 {
                "imagen"
            } else {
                "imágenes"
            },
            peso(r.bytes)
        );
        for m in &r.muestra {
            texto.push_str(&format!("\n  {m}"));
        }
        if r.imagenes > r.muestra.len() {
            texto.push_str(&format!("\n  … y {} más", r.imagenes - r.muestra.len()));
        }
        texto.push_str("\nPara convertirlas: empezar_lote.");
        return Ok(Respuesta::nueva(texto, &r));
    }
    let a = abrir(cx, &e.ruta)?;
    let i = &a.info;
    let mp = (i.ancho as f64 * i.alto as f64) / 1e6;
    let metadatos: Vec<String> = [("EXIF", i.exif), ("ICC", i.icc), ("XMP", i.xmp)]
        .iter()
        .filter_map(|(n, v)| v.map(|b| format!("{n} {}", peso(b as u64))))
        .collect();
    let mut texto = format!(
        "{} · {} {}×{} ({} MP) · {} · {}",
        a.ruta.display(),
        i.formato,
        i.ancho,
        i.alto,
        coma(mp, 1),
        peso(i.bytes),
        if i.alfa {
            "con transparencia"
        } else {
            "sin transparencia"
        }
    );
    if !metadatos.is_empty() {
        texto.push_str(&format!(" · {}", metadatos.join(", ")));
    }
    if i.orientacion != 1 {
        texto.push_str(&format!(
            "\nOrientación EXIF {}: se ve girada si no se endereza (optimizar_imagen la endereza por defecto).",
            i.orientacion
        ));
    }
    let datos = DatosImagen {
        ruta: a.ruta.display().to_string(),
        formato: i.formato,
        ancho: i.ancho,
        alto: i.alto,
        megapixeles: (mp * 10.0).round() / 10.0,
        bytes: i.bytes,
        alfa: i.alfa,
        exif: i.exif,
        icc: i.icc,
        xmp: i.xmp,
        orientacion: i.orientacion,
    };
    Ok(Respuesta::nueva(texto, &datos))
}

// ─── 2. optimizar_imagen ─────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaOptimizar {
    /// Ruta absoluta de la imagen (se admite ~).
    pub ruta: String,
    #[serde(flatten)]
    pub formato: EntradaFormato,
    /// Dónde dejarla: una carpeta, o un fichero con la extensión del
    /// formato. Por defecto, junto a la original. Nunca se sobrescribe nada.
    #[serde(default)]
    pub salida: Option<String>,
    /// Medir también PSNR y SSIM (la nota ya se mide al buscarla).
    #[serde(default)]
    pub medir: bool,
}

#[derive(Serialize)]
struct DatosOptimizar {
    entrada: String,
    salida: String,
    formato: FormatoSalida,
    bytes_entrada: u64,
    bytes_salida: u64,
    ahorro_pct: f64,
    mas_pesado: bool,
    ancho: u32,
    alto: u32,
    milisegundos: u64,
    hallada: Option<Hallada>,
    medidas: Option<Medidas>,
    orden_equivalente: Option<String>,
    motivo: Option<Motivo>,
}

/// Dónde escribir: junto a la original, en una carpeta, o en el fichero dado.
fn destino(a: &Abierta, salida: Option<&str>, formato: FormatoSalida) -> Hecho<PathBuf> {
    let nombre = a.cx.servicio.nombre_salida(a.info.id, formato)?;
    let Some(s) = salida else {
        return Ok(a.ruta.with_file_name(nombre));
    };
    let p = resolver(s)?;
    if p.is_dir() || s.ends_with('/') || s.ends_with('\\') || p.extension().is_none() {
        return Ok(p.join(nombre));
    }
    let ext = p
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let vale = ext == formato.extension() || (formato == FormatoSalida::Jpeg && ext == "jpeg");
    if !vale {
        return Err(ErrorMcp::nuevo(
            "extension_no_cuadra",
            format!(
                "«{}» no tiene la extensión de {} (.{}).",
                p.display(),
                formato.nombre(),
                formato.extension()
            ),
        )
        .con_pista("Cambia la extensión, o pasa solo la carpeta en salida."));
    }
    Ok(p)
}

pub fn optimizar_imagen(cx: &Contexto, e: EntradaOptimizar, s: &Seguimiento) -> Hecho<Respuesta> {
    let a = abrir(cx, &e.ruta)?;
    let id = a.info.id;
    let ajuste = e
        .formato
        .ajuste(&cx.servicio.presets(), a.formato, a.info.orientacion)?;
    let destino = destino(&a, e.salida.as_deref(), ajuste.formato)?;
    let _turno = cx.turno();
    let avisar = |p: i32| {
        (s.avisar)(p);
        if s.cancelado.load(Ordering::SeqCst) {
            cx.servicio.cancelar(id, 0);
        }
    };
    let v = cx
        .servicio
        .codificar_con(id, &ajuste, 0, 1, &avisar)
        .map_err(|f| tamano_cambia(ErrorMcp::from(f)))?;
    let bytes = cx.servicio.bytes_finales(id, &ajuste, 0)?;
    let medidas = if e.medir {
        Some(cx.servicio.medir(id, 0, 1)?)
    } else {
        None
    };
    let escrita = escribir_sin_pisar(&destino, &bytes)?;
    let efectivo = match v.hallada {
        Some(h) => salida::con_calidad(&ajuste, h.calidad),
        None => ajuste.clone(),
    };
    let orden = v.motivo.is_none().then(|| {
        salida::orden(
            &efectivo,
            a.formato.unwrap_or(Formato::Png),
            &a.ruta.display().to_string(),
            &escrita.display().to_string(),
        )
    });
    let total = a.info.bytes;
    let mut texto = format!(
        "{} → {} · {} {} ({}) · {}×{} · {}",
        a.ruta
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default(),
        escrita.display(),
        ajuste.formato.nombre(),
        peso(v.bytes as u64),
        ahorro(total, v.bytes as u64),
        v.ancho,
        v.alto,
        segundos(v.milisegundos)
    );
    if let (Some(h), Some(o)) = (v.hallada, ajuste.objetivo) {
        texto.push_str(&format!("\n{}", linea_hallada(&h, o)));
    }
    if let Some(m) = &medidas {
        texto.push_str(&format!("\n{}", linea_medidas(m)));
    }
    match (&orden, v.motivo) {
        (Some(o), _) => texto.push_str(&format!("\nOrden equivalente: {o}")),
        (None, Some(m)) => texto.push_str(&format!(
            "\nSin orden equivalente: {}.",
            texto_motivo(m, v.herramienta, a.info.formato)
        )),
        _ => {}
    }
    let mas_pesado = v.bytes as u64 > total;
    if mas_pesado {
        texto.push_str("\nAviso: pesa más que el original (se ha guardado igual).");
    }
    let datos = DatosOptimizar {
        entrada: a.ruta.display().to_string(),
        salida: escrita.display().to_string(),
        formato: ajuste.formato,
        bytes_entrada: total,
        bytes_salida: v.bytes as u64,
        ahorro_pct: (ahorro_pct(total, v.bytes as u64) * 10.0).round() / 10.0,
        mas_pesado,
        ancho: v.ancho,
        alto: v.alto,
        milisegundos: v.milisegundos,
        hallada: v.hallada,
        medidas,
        orden_equivalente: orden,
        motivo: v.motivo,
    };
    Ok(Respuesta::nueva(texto, &datos))
}

/// Buscar una nota con `-resize` de cwebp no se puede: se dice cómo sí.
fn tamano_cambia(e: ErrorMcp) -> ErrorMcp {
    if e.mensaje.contains("cambia el tamaño") {
        ErrorMcp::nuevo("tamano_cambia", e.mensaje)
            .con_pista("Redimensiona con ancho o alto, no con -resize en orden.")
    } else {
        e
    }
}

// ─── 3. comparar_formatos ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Guardar {
    /// Solo el más ligero de los que llegan a la nota.
    #[default]
    Mejor,
    /// Todos, cada uno en su fichero.
    Todos,
    /// Ninguno: solo comparar.
    Ninguno,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaComparar {
    /// Ruta absoluta de la imagen (se admite ~).
    pub ruta: String,
    /// Los formatos a comparar. Por defecto, webp, avif y jpeg.
    #[serde(default)]
    pub formatos: Option<Vec<FormatoMcp>>,
    /// La nota SSIMULACRA 2 que tienen que alcanzar todos. Por defecto, 80.
    #[serde(default)]
    #[schemars(range(min = 0, max = 100))]
    pub nota_objetivo: Option<f32>,
    /// Qué guardar: mejor (por defecto), todos o ninguno.
    #[serde(default)]
    pub guardar: Guardar,
    /// Carpeta donde guardar. Por defecto, junto a la original.
    #[serde(default)]
    pub salida: Option<String>,
    /// Redimensionar a este ancho, en píxeles (con alto solo, proporcional).
    #[serde(default)]
    pub ancho: Option<u32>,
    /// Redimensionar a este alto, en píxeles (con ancho solo, proporcional).
    #[serde(default)]
    pub alto: Option<u32>,
    /// Girar según la orientación EXIF si la foto lo necesita. Por defecto, sí.
    #[serde(default)]
    pub enderezar: Option<bool>,
}

#[derive(Serialize)]
struct Comparado {
    formato: FormatoSalida,
    bytes: u64,
    calidad: Option<u8>,
    nota: Option<f64>,
    alcanzada: bool,
    ahorro_pct: f64,
}

#[derive(Serialize)]
struct DatosComparar {
    entrada: String,
    nota_objetivo: f32,
    resultados: Vec<Comparado>,
    mejor: Option<FormatoSalida>,
    guardados: Vec<String>,
}

pub fn comparar_formatos(cx: &Contexto, e: EntradaComparar, s: &Seguimiento) -> Hecho<Respuesta> {
    let formatos = e
        .formatos
        .clone()
        .unwrap_or_else(|| vec![FormatoMcp::Webp, FormatoMcp::Avif, FormatoMcp::Jpeg]);
    if formatos.is_empty() {
        return Err(
            ErrorMcp::nuevo("formatos_vacios", "No hay formatos que comparar.")
                .con_pista("Quita formatos (webp, avif y jpeg) o da al menos uno."),
        );
    }
    let objetivo = e.nota_objetivo.unwrap_or(NOTA_POR_DEFECTO);
    let a = abrir(cx, &e.ruta)?;
    let id = a.info.id;
    let presets = cx.servicio.presets();
    let _turno = cx.turno();
    let total = formatos.len() as i32;
    let mut hechos: Vec<(Ajuste, Comparado, u8)> = Vec::new();
    for (i, f) in formatos.iter().enumerate().take(6) {
        let lado = i as u8;
        let base = EntradaFormato {
            formato: Some(*f),
            ancho: e.ancho,
            alto: e.alto,
            enderezar: e.enderezar,
            ..Default::default()
        };
        let con_nota = EntradaFormato {
            nota_objetivo: Some(objetivo),
            ..base.clone()
        };
        // Sin calidad que buscar (PNG, QOI, un JPEG que cjxl recomprime sin
        // pérdida), tal cual.
        let ajuste = match con_nota.ajuste(&presets, a.formato, a.info.orientacion) {
            Err(x) if x.error == "nota_no_admitida" => {
                base.ajuste(&presets, a.formato, a.info.orientacion)?
            }
            otro => otro?,
        };
        let avisar = |p: i32| {
            (s.avisar)((i as i32 * 100 + p) / total);
            if s.cancelado.load(Ordering::SeqCst) {
                cx.servicio.cancelar(id, lado);
            }
        };
        let v = cx
            .servicio
            .codificar_con(id, &ajuste, lado, 1, &avisar)
            .map_err(|f| tamano_cambia(ErrorMcp::from(f)))?;
        let nota = match v.hallada {
            Some(h) => Some(h.nota),
            None => cx.servicio.medir(id, lado, 1)?.ssimulacra2,
        };
        let c = Comparado {
            formato: ajuste.formato,
            bytes: v.bytes as u64,
            calidad: v.hallada.map(|h| h.calidad),
            nota,
            alcanzada: nota.is_some_and(|n| n >= objetivo as f64),
            ahorro_pct: (ahorro_pct(a.info.bytes, v.bytes as u64) * 10.0).round() / 10.0,
        };
        hechos.push((ajuste, c, lado));
    }
    hechos.sort_by_key(|(_, c, _)| c.bytes);
    // El más ligero de los que llegan; si ninguno llega, el de mejor nota.
    let mejor = hechos.iter().position(|(_, c, _)| c.alcanzada).or_else(|| {
        (0..hechos.len()).max_by(|&x, &y| {
            hechos[x]
                .1
                .nota
                .unwrap_or(f64::MIN)
                .total_cmp(&hechos[y].1.nota.unwrap_or(f64::MIN))
        })
    });
    let mut guardados = Vec::new();
    for (i, (ajuste, _, lado)) in hechos.iter().enumerate() {
        let toca = match e.guardar {
            Guardar::Todos => true,
            Guardar::Mejor => Some(i) == mejor,
            Guardar::Ninguno => false,
        };
        if toca {
            let bytes = cx.servicio.bytes_finales(id, ajuste, *lado)?;
            let d = destino(&a, e.salida.as_deref(), ajuste.formato)?;
            guardados.push(escribir_sin_pisar(&d, &bytes)?.display().to_string());
        }
    }
    let mut texto = format!(
        "{} ({}) a la nota {}:",
        a.ruta.display(),
        peso(a.info.bytes),
        coma(objetivo as f64, 0)
    );
    for (i, (_, c, _)) in hechos.iter().enumerate() {
        texto.push_str(&format!(
            "\n{} {} {} ({}) · {}{}",
            if Some(i) == mejor { "✔" } else { " " },
            c.formato.nombre(),
            peso(c.bytes),
            ahorro(a.info.bytes, c.bytes),
            c.nota
                .map(|n| format!("nota {}", coma(n, 1)))
                .unwrap_or_else(|| "sin nota".into()),
            match (c.calidad, c.alcanzada) {
                (Some(q), true) => format!(" · calidad {q}"),
                (_, false) => " · no llega".into(),
                (None, true) => String::new(),
            }
        ));
    }
    for g in &guardados {
        texto.push_str(&format!("\nGuardado: {g}"));
    }
    let datos = DatosComparar {
        entrada: a.ruta.display().to_string(),
        nota_objetivo: objetivo,
        mejor: mejor.map(|i| hechos[i].1.formato),
        resultados: hechos.into_iter().map(|(_, c, _)| c).collect(),
        guardados,
    };
    Ok(Respuesta::nueva(texto, &datos))
}

// ─── 4. medir_calidad ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaMedir {
    /// La imagen original, ruta absoluta.
    pub original: String,
    /// La imagen a medir, del mismo tamaño.
    pub resultado: String,
}

/// El original y el resultado como RGBA del mismo tamaño: si el resultado
/// sale enderezado y el original no, se endereza el original.
fn par(original: &Path, resultado: &Path) -> Hecho<(u32, u32, Vec<u8>, Vec<u8>)> {
    let ((w, h, a), o, img) = leer_rgba(original)?;
    let ((rw, rh, b), _, _) = leer_rgba(resultado)?;
    if (w, h) == (rw, rh) {
        return Ok((w, h, a, b));
    }
    if o != 1
        && let Some(girada) = orientacion::enderezar(&img)?
    {
        let (gw, gh, g) = vista::rgba(&girada)?;
        if (gw, gh) == (rw, rh) {
            return Ok((gw, gh, g, b));
        }
    }
    Err(ErrorMcp::nuevo(
        "tamano_distinto",
        format!("No miden lo mismo: {w}×{h} y {rw}×{rh}."),
    )
    .con_pista("Si se redimensionó, mide con optimizar_imagen y medir: true, que compara con lo que recibió el codificador."))
}

pub fn medir_calidad(cx: &Contexto, e: EntradaMedir) -> Hecho<Respuesta> {
    let (o, r) = (existe(&e.original)?, existe(&e.resultado)?);
    let (w, h, a, b) = par(&o, &r)?;
    let m = {
        let _turno = cx.turno();
        medir::medir(&a, &b, w, h)?
    };
    let texto = format!(
        "{} frente a {}:\n{}",
        r.display(),
        o.display(),
        linea_medidas(&m)
    );
    #[derive(Serialize)]
    struct Datos {
        original: String,
        resultado: String,
        #[serde(flatten)]
        medidas: Medidas,
        interpretacion: Option<&'static str>,
    }
    Ok(Respuesta::nueva(
        texto,
        &Datos {
            original: o.display().to_string(),
            resultado: r.display().to_string(),
            interpretacion: m.ssimulacra2.map(banda),
            medidas: m,
        },
    ))
}

// ─── 5. ver_imagen ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct Zona {
    /// Desde la izquierda, en píxeles.
    pub x: u32,
    /// Desde arriba, en píxeles.
    pub y: u32,
    pub ancho: u32,
    pub alto: u32,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TipoMapaMcp {
    /// Dónde cambian los píxeles, aunque sea poco.
    Diferencia,
    /// Dónde se pierde detalle (1 − SSIM local): lo que el ojo nota.
    #[default]
    Estructura,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaVer {
    /// Ruta absoluta de la imagen a ver.
    pub ruta: String,
    /// El lado mayor de la vista, en píxeles: 768 por defecto, como mucho 1568.
    #[serde(default)]
    pub lado_max: Option<u32>,
    /// Ver solo esta zona, en píxeles de la imagen (para mirar defectos a 1:1).
    #[serde(default)]
    pub zona: Option<Zona>,
    /// Ruta del original: en vez de la imagen, el mapa de dónde se aleja de él.
    #[serde(default)]
    pub diferencias_con: Option<String>,
    /// Con diferencias_con: estructura (por defecto) o diferencia.
    #[serde(default)]
    pub tipo_mapa: TipoMapaMcp,
}

/// Una vista: como mucho este peso en base64.
const TOPE_VISTA: usize = 1024 * 1024;

pub fn ver_imagen(cx: &Contexto, e: EntradaVer) -> Hecho<Respuesta> {
    let ruta = existe(&e.ruta)?;
    let (ow, oh, mut rgba) = match &e.diferencias_con {
        Some(o) => {
            let original = existe(o)?;
            let (w, h, a, b) = par(&original, &ruta)?;
            let tipo = match e.tipo_mapa {
                TipoMapaMcp::Diferencia => TipoMapa::Diferencia,
                TipoMapaMcp::Estructura => TipoMapa::Estructura,
            };
            let _turno = cx.turno();
            (w, h, medir::mapa(&a, &b, w, h, tipo)?)
        }
        None => leer_rgba(&ruta)?.0,
    };
    let (mut w, mut h) = (ow, oh);
    if let Some(z) = e.zona {
        if z.ancho == 0
            || z.alto == 0
            || z.x.saturating_add(z.ancho) > w
            || z.y.saturating_add(z.alto) > h
        {
            return Err(ErrorMcp::nuevo(
                "zona_fuera",
                format!("La zona se sale de la imagen ({w}×{h})."),
            )
            .con_pista("x + ancho y y + alto no pueden pasar de las medidas de la imagen."));
        }
        let mut recorte = Vec::with_capacity((z.ancho * z.alto * 4) as usize);
        for fila in z.y..z.y + z.alto {
            let i = ((fila * w + z.x) * 4) as usize;
            recorte.extend_from_slice(&rgba[i..i + (z.ancho * 4) as usize]);
        }
        (rgba, w, h) = (recorte, z.ancho, z.alto);
    }
    let alfa = rgba.as_chunks::<4>().0.iter().any(|p| p[3] != 255);
    let mut lado = e.lado_max.unwrap_or(768).clamp(64, 1568);
    let mut sin_png = false;
    loop {
        let (vw, vh, v) = if w.max(h) > lado {
            let r = if w >= h {
                Redimension {
                    ancho: Some(lado),
                    alto: None,
                    ..Default::default()
                }
            } else {
                Redimension {
                    ancho: None,
                    alto: Some(lado),
                    ..Default::default()
                }
            };
            apolo_nucleo::proceso::aplicar(
                &Proceso {
                    redimension: Some(r),
                    ..Default::default()
                },
                w,
                h,
                &rgba,
            )?
        } else {
            (w, h, rgba.clone())
        };
        // JPEG si es opaca (pesa menos); PNG si tiene transparencia, o si es
        // una zona para buscar defectos y cabe: el JPEG de la vista pondría
        // los suyos (lo notó un agente en el paseo). WebP no: algún cliente
        // no lo pinta.
        let png = (alfa || (e.zona.is_some() && !sin_png))
            .then(|| png::png_de_pixeles(vw, vh, &v, None))
            .transpose()?
            .filter(|d| alfa || d.len() / 3 * 4 <= TOPE_VISTA);
        if e.zona.is_some() && !alfa && png.is_none() {
            sin_png = true;
        }
        let (mime, datos) = if let Some(d) = png {
            ("image/png", d)
        } else {
            let entrada = jpeg::EntradaJpeg::desde_rgba(vw, vh, &v);
            (
                "image/jpeg",
                jpeg::cjpeg::ejecutar(
                    &entrada,
                    &["-quality".into(), "82".into()],
                    &Default::default(),
                )?,
            )
        };
        let base64 = base64::engine::general_purpose::STANDARD.encode(&datos);
        if base64.len() > TOPE_VISTA && lado > 256 {
            lado = lado * 2 / 3;
            continue;
        }
        let que = if e.diferencias_con.is_some() {
            "Mapa de diferencias de"
        } else {
            "Vista de"
        };
        let texto = format!(
            "{que} {} a {vw}×{vh} (original {ow}×{oh}{}).",
            ruta.display(),
            e.zona
                .map(|z| format!(", zona {}×{} en {},{}", z.ancho, z.alto, z.x, z.y))
                .unwrap_or_default()
        );
        #[derive(Serialize)]
        struct Datos {
            ruta: String,
            ancho_original: u32,
            alto_original: u32,
            ancho: u32,
            alto: u32,
            mime: &'static str,
        }
        let mut r = Respuesta::nueva(
            texto,
            &Datos {
                ruta: ruta.display().to_string(),
                ancho_original: ow,
                alto_original: oh,
                ancho: vw,
                alto: vh,
                mime,
            },
        );
        r.imagen = Some(Imagen { mime, base64 });
        return Ok(r);
    }
}

// ─── 6, 7 y 8. Lotes ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaLote {
    /// Carpetas o imágenes, rutas absolutas (se admite ~).
    pub entradas: Vec<String>,
    /// Formatos de salida (uno o varios). Por defecto, el del preset o webp.
    #[serde(default)]
    pub formatos: Option<Vec<FormatoMcp>>,
    /// Un preset guardado, base de todo.
    #[serde(default)]
    pub preset: Option<String>,
    /// Opciones de la herramienta oficial (solo con un formato).
    #[serde(default)]
    pub orden: Option<String>,
    /// Calidad fija de 0 a 100. No a la vez que nota_objetivo.
    #[serde(default)]
    #[schemars(range(min = 0, max = 100))]
    pub calidad: Option<u8>,
    /// Buscar, imagen a imagen, la calidad que da esta nota. Por defecto, 80.
    #[serde(default)]
    #[schemars(range(min = 0, max = 100))]
    pub nota_objetivo: Option<f32>,
    /// Sin pérdida (WebP, AVIF, JPEG XL). PNG y QOI siempre lo son.
    #[serde(default)]
    pub sin_perdida: bool,
    /// Redimensionar a este ancho, en píxeles (con alto solo, proporcional).
    #[serde(default)]
    pub ancho: Option<u32>,
    /// Redimensionar a este alto, en píxeles (con ancho solo, proporcional).
    #[serde(default)]
    pub alto: Option<u32>,
    /// Girar cada foto según su orientación EXIF si lo necesita. Por defecto, sí.
    #[serde(default)]
    pub enderezar: Option<bool>,
    /// Con varios formatos: guardar de cada imagen solo el que menos pese.
    #[serde(default)]
    pub mas_ligero: bool,
    /// Medir la nota de cada fichero (tarda más).
    #[serde(default)]
    pub medir: bool,
    /// Carpeta de salida. Por defecto, junto a la de entrada con -webp, -apolo…
    #[serde(default)]
    pub salida: Option<String>,
}

fn id_lote(texto: &str) -> Hecho<u64> {
    texto
        .trim()
        .strip_prefix("l-")
        .and_then(|n| n.parse().ok())
        .ok_or_else(|| {
            ErrorMcp::nuevo("lote_no_existe", format!("«{texto}» no es un lote."))
                .con_pista("El id lo da empezar_lote, como «l-3».")
        })
}

pub fn empezar_lote(cx: &Contexto, e: EntradaLote) -> Hecho<Respuesta> {
    if let Some(id) = *cx.lote.lock().unwrap_or_else(|x| x.into_inner())
        && cx
            .servicio
            .estado_lote(id, usize::MAX)
            .is_ok_and(|st| !st.terminado)
    {
        return Err(ErrorMcp::nuevo(
            "lote_en_marcha",
            format!("Ya hay un lote en marcha: l-{id}."),
        )
        .con_pista("Espera con ver_lote, o páralo con cancelar_lote."));
    }
    let mut rutas = Vec::new();
    for t in &e.entradas {
        rutas.push(existe(t)?);
    }
    if rutas.is_empty() {
        return Err(ErrorMcp::nuevo("sin_imagenes", "No hay entradas.")
            .con_pista("Pasa en entradas carpetas o imágenes."));
    }
    let formatos: Vec<Option<FormatoMcp>> = match &e.formatos {
        Some(v) if !v.is_empty() => v.iter().map(|f| Some(*f)).collect(),
        _ => vec![None],
    };
    if formatos.len() > 1 && e.orden.is_some() {
        return Err(ErrorMcp::nuevo(
            "orden_con_varios_formatos",
            "Una orden es de una herramienta: no vale para varios formatos.",
        )
        .con_pista("Quita orden, o deja un solo formato."));
    }
    let presets = cx.servicio.presets();
    let mut ajustes = Vec::new();
    for f in formatos {
        let ef = EntradaFormato {
            formato: f,
            preset: e.preset.clone(),
            orden: e.orden.clone(),
            calidad: e.calidad,
            nota_objetivo: e.nota_objetivo,
            sin_perdida: e.sin_perdida,
            ancho: e.ancho,
            alto: e.alto,
            enderezar: e.enderezar,
        };
        let mut a = ef.ajuste(&presets, None, 1)?;
        // En un lote cada foto tiene su orientación: el núcleo endereza solo
        // las que lo necesitan.
        a.proceso.enderezar = e.enderezar.unwrap_or(true);
        ajustes.push(a);
    }
    let salida = match &e.salida {
        Some(s) => resolver(s)?,
        None => salida_sugerida(&rutas, &sufijo(&ajustes)).ok_or_else(|| {
            ErrorMcp::nuevo("sin_salida", "No se puede proponer una carpeta de salida.")
                .con_pista("Pasa la carpeta en salida.")
        })?,
    };
    let entradas: Vec<String> = rutas.iter().map(|r| r.display().to_string()).collect();
    let empezado = cx
        .servicio
        .empezar_lote(
            &entradas,
            &ajustes,
            e.mas_ligero && ajustes.len() > 1,
            e.medir,
            &salida.display().to_string(),
        )
        .map_err(|f| {
            if f.mensaje.contains("No hay imágenes") {
                ErrorMcp::nuevo("sin_imagenes", f.mensaje)
                    .con_pista("analizar_imagen sobre la carpeta dice qué hay.")
            } else {
                ErrorMcp::from(f)
            }
        })?;
    *cx.lote.lock().unwrap_or_else(|x| x.into_inner()) = Some(empezado.id);
    let mut texto = format!(
        "Lote l-{} empezado: {} {} → {}\nPregunta cómo va con ver_lote(id: \"l-{}\", esperar_segundos: 20).",
        empezado.id,
        empezado.total,
        if empezado.total == 1 {
            "imagen"
        } else {
            "imágenes"
        },
        empezado.salida,
        empezado.id
    );
    if ajustes.iter().any(|a| a.objetivo.is_some()) {
        texto.push_str(" Buscando la nota, cada imagen hace unas 7 pruebas y van varias a la vez: las primeras pueden tardar un par de minutos en salir.");
    }
    #[derive(Serialize)]
    struct Datos {
        id: String,
        total: usize,
        salida: String,
    }
    Ok(Respuesta::nueva(
        texto,
        &Datos {
            id: format!("l-{}", empezado.id),
            total: empezado.total,
            salida: empezado.salida,
        },
    ))
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaVerLote {
    /// El id que dio empezar_lote, como «l-3».
    pub id: String,
    /// Desde qué imagen contar las nuevas (siguiente_desde de la respuesta
    /// anterior). Por defecto, 0: todas.
    #[serde(default)]
    pub desde: usize,
    /// Esperar hasta este número de segundos (como mucho 25) a que haya algo
    /// nuevo o acabe, en vez de preguntar en bucle.
    #[serde(default)]
    pub esperar_segundos: u64,
}

pub fn ver_lote(cx: &Contexto, e: EntradaVerLote) -> Hecho<Respuesta> {
    let id = id_lote(&e.id)?;
    let limite = Instant::now() + Duration::from_secs(e.esperar_segundos.min(25));
    let st = loop {
        let st = cx.servicio.estado_lote(id, e.desde).map_err(|_| {
            ErrorMcp::nuevo("lote_no_existe", format!("No hay ningún lote {}.", e.id))
                .con_pista("Los lotes viven mientras el servidor sigue abierto; empieza otro con empezar_lote.")
        })?;
        if !st.nuevas.is_empty() || st.terminado || Instant::now() >= limite {
            break st;
        }
        std::thread::sleep(Duration::from_millis(250));
    };
    Ok(respuesta_lote(id, e.desde, st))
}

fn respuesta_lote(id: u64, desde: usize, st: apolo_servicio::EstadoLote) -> Respuesta {
    let estado = if st.cancelado {
        "cancelado"
    } else if st.terminado {
        "terminado"
    } else {
        "en marcha"
    };
    let mut texto = format!(
        "l-{id}: {}/{} · {} · {estado}",
        st.hechas,
        st.total,
        segundos((st.segundos * 1000.0) as u64)
    );
    for f in &st.nuevas {
        match (&f.error, f.salidas.first()) {
            (Some(err), _) => texto.push_str(&format!("\n{}: ERROR {err}", f.relativa)),
            (None, Some(s)) => {
                texto.push_str(&format!(
                    "\n{} → {} {} ({})",
                    f.relativa,
                    s.formato.nombre(),
                    peso(s.bytes),
                    ahorro(f.bytes_entrada, s.bytes)
                ));
                if let Some(q) = s.calidad {
                    texto.push_str(&format!(" · calidad {q}"));
                }
                if let Some(n) = s.nota {
                    texto.push_str(&format!(" · nota {}", coma(n, 1)));
                }
            }
            (None, None) => {}
        }
    }
    if let Some(r) = &st.resumen {
        texto.push_str(&format!(
            "\nResumen: {} convertidas, {} con error · {} → {} ({})",
            r.convertidas,
            r.fallidas,
            peso(r.bytes_entrada),
            peso(r.bytes_salida),
            ahorro(r.bytes_entrada, r.bytes_salida)
        ));
        if r.mayores > 0 {
            texto.push_str(&format!(
                "\n{} pesan más que su original (se han guardado igual).",
                r.mayores
            ));
        }
        if let Some(m) = r.nota_media {
            texto.push_str(&format!(
                "\nNota media (SSIMULACRA 2): {} ({})",
                coma(m, 1),
                banda(m)
            ));
        }
    }
    let siguiente = desde + st.nuevas.len();
    #[derive(Serialize)]
    struct Datos {
        id: String,
        #[serde(flatten)]
        estado: apolo_servicio::EstadoLote,
        siguiente_desde: usize,
    }
    Respuesta::nueva(
        texto,
        &Datos {
            id: format!("l-{id}"),
            estado: st,
            siguiente_desde: siguiente,
        },
    )
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaCancelarLote {
    /// El id que dio empezar_lote, como «l-3».
    pub id: String,
}

pub fn cancelar_lote(cx: &Contexto, e: EntradaCancelarLote) -> Hecho<Respuesta> {
    let id = id_lote(&e.id)?;
    cx.servicio.estado_lote(id, usize::MAX).map_err(|_| {
        ErrorMcp::nuevo("lote_no_existe", format!("No hay ningún lote {}.", e.id))
            .con_pista("El id lo da empezar_lote, como «l-3».")
    })?;
    cx.servicio.cancelar_lote(id);
    // Lo que estuviera a medias se aborta enseguida; se espera un poco a que
    // conste como cancelado.
    let limite = Instant::now() + Duration::from_secs(10);
    let mut st = cx.servicio.estado_lote(id, 0)?;
    while !st.terminado && Instant::now() < limite {
        std::thread::sleep(Duration::from_millis(100));
        st = cx.servicio.estado_lote(id, 0)?;
    }
    let mut r = respuesta_lote(id, 0, st);
    r.texto = format!("Cancelado: lo ya convertido se queda.\n{}", r.texto);
    Ok(r)
}

// ─── 9 y 10. Presets ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct DatosPreset {
    nombre: String,
    formato: FormatoSalida,
    orden: String,
    nota_objetivo: Option<f32>,
}

fn datos_preset(p: &PresetGuardado) -> DatosPreset {
    DatosPreset {
        nombre: p.nombre.clone(),
        formato: p.ajuste.formato,
        orden: apolo_servicio::orden_opciones(&p.ajuste),
        nota_objetivo: p.ajuste.objetivo,
    }
}

pub fn listar_presets(cx: &Contexto) -> Hecho<Respuesta> {
    let lista: Vec<DatosPreset> = cx.servicio.presets().iter().map(datos_preset).collect();
    let carpeta = cx.servicio.carpeta_presets().display().to_string();
    let mut texto = if lista.is_empty() {
        format!("No hay presets todavía ({carpeta}). Se crean con guardar_preset o en Apolo.")
    } else {
        format!("{} presets ({carpeta}):", lista.len())
    };
    for p in &lista {
        texto.push_str(&format!(
            "\n{} · {} · {}",
            p.nombre,
            p.formato.nombre(),
            p.orden
        ));
        if let Some(n) = p.nota_objetivo {
            texto.push_str(&format!(" · nota objetivo {}", coma(n as f64, 0)));
        }
    }
    #[derive(Serialize)]
    struct Datos {
        carpeta: String,
        presets: Vec<DatosPreset>,
    }
    Ok(Respuesta::nueva(
        texto,
        &Datos {
            carpeta,
            presets: lista,
        },
    ))
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EntradaGuardarPreset {
    /// El nombre del preset nuevo. No puede existir ya.
    pub nombre: String,
    #[serde(flatten)]
    pub formato: EntradaFormato,
}

pub fn guardar_preset(cx: &Contexto, e: EntradaGuardarPreset) -> Hecho<Respuesta> {
    let nombre = e.nombre.trim().to_string();
    if nombre.is_empty() {
        return Err(
            ErrorMcp::nuevo("sin_nombre", "El preset necesita un nombre.")
                .con_pista("Pasa nombre."),
        );
    }
    let presets = cx.servicio.presets();
    let ya = presets
        .iter()
        .any(|p| p.nombre.eq_ignore_ascii_case(&nombre))
        || cx
            .servicio
            .carpeta_presets()
            .join(presets::fichero(&nombre))
            .exists();
    if ya {
        return Err(ErrorMcp::nuevo(
            "preset_existe",
            format!("Ya hay un preset «{nombre}», y no se sobrescribe."),
        )
        .con_pista("Elige otro nombre; para cambiarlo o borrarlo, en Apolo › Presets."));
    }
    let mut ajuste = e.formato.ajuste(&presets, None, 1)?;
    ajuste.proceso.enderezar = e.formato.enderezar.unwrap_or(true);
    let p = PresetGuardado { nombre, ajuste };
    cx.servicio.guardar_preset(&p)?;
    let d = datos_preset(&p);
    let texto = format!(
        "Preset «{}» guardado: {} · {}\nSe usa en Apolo, en empezar_lote con preset, o con apolo {} -apolo_preset \"{}\".",
        d.nombre,
        d.formato.nombre(),
        d.orden,
        match d.formato {
            FormatoSalida::Jpeg => "jpeg",
            f => f.extension(),
        },
        d.nombre
    );
    Ok(Respuesta::nueva(texto, &d))
}
