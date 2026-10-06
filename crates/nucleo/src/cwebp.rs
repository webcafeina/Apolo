//! La orden `cwebp`: leerla y escribirla.
//!
//! `leer` es el bucle de opciones de `examples/cwebp.c` (libwebp 1.6.0): las
//! aplica sobre una `WebPConfig` de verdad, en el orden en que llegan, con sus
//! rarezas (`-preset` reescribe todo lo anterior, `-q` y `-m` anulan `-z`, los
//! enteros se leen con `strtol` en base 0, así que `010` es 8). `escribir` hace
//! el camino de vuelta: de unas opciones, la orden más corta que las produce.

use crate::entrada::{self, Lectura};
use crate::error::Resultado;
use crate::metadatos::Conservar;
use crate::webp::opciones::{
    FiltradoAlfa, ModoRedimension, OpcionesWebp, Pista, Preset, Recorte, Redimension, config_base,
    config_preset,
};
use crate::webp::{self, Codificado, Extras, Medida, Progreso};

/// Qué ayuda pidió.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ayuda {
    Corta,
    Larga,
}

/// Una orden cwebp leída.
#[derive(Debug, Clone, Default)]
pub struct OrdenCwebp {
    pub opciones: OpcionesWebp,
    pub entrada: Option<String>,
    pub salida: Option<String>,
    pub extras: Extras,
    /// `-d`: dónde volcar el PGM.
    pub volcado: Option<String>,
    /// `-s w h`: la entrada es YUV crudo de ese tamaño.
    pub tamano_yuv: Option<(i32, i32)>,
    /// `-short`, que se puede repetir.
    pub breve: u32,
    pub silencio: bool,
    pub detalle: bool,
    pub progreso: bool,
    pub sin_asm: bool,
    pub ayuda: Option<Ayuda>,
    pub version: bool,
}

/// Un error al leer la orden, con el texto que hay que enseñar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorOrden(pub String);

impl std::fmt::Display for ErrorOrden {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// `strtol(v, &end, base)` con `base` 0 o 16: lo que se pueda leer, y error
/// solo si no se leyó ningún dígito.
fn strtol(v: &str, base: u32) -> Option<i64> {
    let s = v.trim_start();
    let (signo, s) = match s.as_bytes().first() {
        Some(b'-') => (-1, &s[1..]),
        Some(b'+') => (1, &s[1..]),
        _ => (1, s),
    };
    let con_0x = s.len() > 2
        && (s.starts_with("0x") || s.starts_with("0X"))
        && s.as_bytes()[2].is_ascii_hexdigit();
    let (base, s) = match base {
        16 if con_0x => (16, &s[2..]),
        0 if con_0x => (16, &s[2..]),
        0 if s.starts_with('0') && s.len() > 1 => (8, s),
        0 => (10, s),
        b => (b, s),
    };
    let digitos: String = s.chars().take_while(|c| c.is_digit(base)).collect();
    if digitos.is_empty() {
        return None;
    }
    let n = i64::from_str_radix(&digitos, base).unwrap_or(i64::MAX);
    Some(signo * n)
}

/// `strtod`: el prefijo más largo que sea un número.
fn strtod(v: &str) -> Option<f32> {
    let s = v.trim_start();
    let b = s.as_bytes();
    let mut i = 0;
    if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
        i += 1;
    }
    let inicio_num = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == inicio_num || (i == inicio_num + 1 && b[inicio_num] == b'.') {
        return None;
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            i = j;
        }
    }
    s[..i].parse::<f64>().ok().map(|x| x as f32)
}

/// Opción propia de Apolo para enderezar (ADR 0012). No existe en cwebp.
pub const ENDEREZAR: &str = "-apolo_enderezar";

/// Lee los argumentos de cwebp (sin el nombre del programa).
pub fn leer<S: AsRef<str>>(args: &[S]) -> Result<OrdenCwebp, ErrorOrden> {
    leer_desde(&OpcionesWebp::default(), args)
}

/// Lee los argumentos de cwebp **encima** de unas opciones de partida (un
/// preset guardado), como si cwebp ya las tuviera puestas.
pub fn leer_desde<S: AsRef<str>>(
    base: &OpcionesWebp,
    args: &[S],
) -> Result<OrdenCwebp, ErrorOrden> {
    let args: Vec<&str> = args.iter().map(|a| a.as_ref()).collect();
    let mut o = OrdenCwebp::default();
    let mut c = base.a_config_sin_normalizar();
    let mut preset = base.preset;
    let mut nivel_sin_perdida = 6;
    let mut usar_nivel: i32 = -1; // -1 sin decir, 0 no, 1 sí
    let mut recorte = base.recorte;
    let mut redim = base
        .redimension
        .map(|r| (r.ancho, r.alto))
        .unwrap_or((0, 0));
    let mut modo = base.modo_redimension;
    let mut mezclar = base.mezclar_alfa;
    let mut conservar_alfa = !base.sin_alfa;
    let mut metadatos = base.metadatos;
    let mut enderezar = base.enderezar;

    let n = args.len();
    let mut i = 0;
    while i < n {
        let a = args[i];
        let mut error = false;
        let entero = |error: &mut bool, j: usize| -> i32 {
            match strtol(args[j], 0) {
                Some(v) => v as i32,
                None => {
                    *error = true;
                    0
                }
            }
        };
        let decimal = |error: &mut bool, j: usize| -> f32 {
            strtod(args[j]).unwrap_or_else(|| {
                *error = true;
                0.0
            })
        };
        let hay = |k: usize| i + k < n;
        match a {
            "-h" | "-help" => {
                o.ayuda = Some(Ayuda::Corta);
                return Ok(o);
            }
            "-H" | "-longhelp" => {
                o.ayuda = Some(Ayuda::Larga);
                return Ok(o);
            }
            "-o" if hay(1) => {
                i += 1;
                o.salida = Some(args[i].to_string());
            }
            "-d" if hay(1) => {
                i += 1;
                o.volcado = Some(args[i].to_string());
                o.extras.volcar = true;
            }
            "-print_psnr" => o.extras.medir = Some(Medida::Psnr),
            "-print_ssim" => o.extras.medir = Some(Medida::Ssim),
            "-print_lsim" => o.extras.medir = Some(Medida::Lsim),
            "-short" => o.breve += 1,
            "-s" if hay(2) => {
                let w = entero(&mut error, i + 1);
                let h = entero(&mut error, i + 2);
                i += 2;
                if !(0..=16383).contains(&w) || !(0..=16383).contains(&h) {
                    return Err(ErrorOrden(format!(
                        "El tamaño indicado ({w} × {h}) se sale del rango."
                    )));
                }
                o.tamano_yuv = Some((w, h));
            }
            "-m" if hay(1) => {
                i += 1;
                c.method = entero(&mut error, i);
                usar_nivel = 0;
            }
            "-q" if hay(1) => {
                i += 1;
                c.quality = decimal(&mut error, i);
                usar_nivel = 0;
            }
            "-z" if hay(1) => {
                i += 1;
                nivel_sin_perdida = entero(&mut error, i);
                if usar_nivel != 0 {
                    usar_nivel = 1;
                }
            }
            "-alpha_q" if hay(1) => {
                i += 1;
                c.alpha_quality = entero(&mut error, i);
            }
            "-alpha_method" if hay(1) => {
                i += 1;
                c.alpha_compression = entero(&mut error, i);
            }
            "-alpha_cleanup" => c.exact = 0,
            "-exact" => c.exact = 1,
            "-blend_alpha" if hay(1) => {
                i += 1;
                let v = strtol(args[i], 16).unwrap_or_else(|| {
                    error = true;
                    0
                });
                mezclar = Some((v as i32 as u32) & 0x00ff_ffff);
            }
            "-alpha_filter" if hay(1) => {
                i += 1;
                c.alpha_filtering = match args[i] {
                    "none" => 0,
                    "fast" => 1,
                    "best" => 2,
                    x => return Err(ErrorOrden(format!("Filtro de alfa desconocido: {x}"))),
                };
            }
            "-noalpha" => conservar_alfa = false,
            "-lossless" => c.lossless = 1,
            "-near_lossless" if hay(1) => {
                i += 1;
                c.near_lossless = entero(&mut error, i);
                c.lossless = 1;
            }
            "-hint" if hay(1) => {
                i += 1;
                use libwebp_sys::WebPImageHint::*;
                c.image_hint = match args[i] {
                    "photo" => WEBP_HINT_PHOTO,
                    "picture" => WEBP_HINT_PICTURE,
                    "graph" => WEBP_HINT_GRAPH,
                    x => return Err(ErrorOrden(format!("Pista de imagen desconocida: {x}"))),
                };
            }
            "-size" if hay(1) => {
                i += 1;
                c.target_size = entero(&mut error, i);
            }
            "-psnr" if hay(1) => {
                i += 1;
                c.target_PSNR = decimal(&mut error, i);
            }
            "-sns" if hay(1) => {
                i += 1;
                c.sns_strength = entero(&mut error, i);
            }
            "-f" if hay(1) => {
                i += 1;
                c.filter_strength = entero(&mut error, i);
            }
            "-af" => c.autofilter = 1,
            "-jpeg_like" => c.emulate_jpeg_size = 1,
            "-mt" => c.thread_level += 1,
            "-low_memory" => c.low_memory = 1,
            "-strong" => c.filter_type = 1,
            "-nostrong" => c.filter_type = 0,
            "-sharpness" if hay(1) => {
                i += 1;
                c.filter_sharpness = entero(&mut error, i);
            }
            "-sharp_yuv" => c.use_sharp_yuv = 1,
            "-pass" if hay(1) => {
                i += 1;
                c.pass = entero(&mut error, i);
            }
            "-qrange" if hay(2) => {
                c.qmin = entero(&mut error, i + 1).max(0);
                c.qmax = entero(&mut error, i + 2).min(100);
                i += 2;
            }
            "-pre" if hay(1) => {
                i += 1;
                c.preprocessing = entero(&mut error, i);
            }
            "-segments" if hay(1) => {
                i += 1;
                c.segments = entero(&mut error, i);
            }
            "-partition_limit" if hay(1) => {
                i += 1;
                c.partition_limit = entero(&mut error, i);
            }
            "-map" if hay(1) => {
                i += 1;
                o.extras.mapa = entero(&mut error, i);
            }
            "-crop" if hay(4) => {
                recorte = Some(Recorte {
                    x: entero(&mut error, i + 1),
                    y: entero(&mut error, i + 2),
                    ancho: entero(&mut error, i + 3),
                    alto: entero(&mut error, i + 4),
                });
                i += 4;
            }
            "-resize" if hay(2) => {
                redim = (entero(&mut error, i + 1), entero(&mut error, i + 2));
                i += 2;
            }
            "-resize_mode" if hay(1) => {
                i += 1;
                modo = match args[i] {
                    "down_only" => ModoRedimension::SoloReducir,
                    "up_only" => ModoRedimension::SoloAmpliar,
                    "always" => ModoRedimension::Siempre,
                    x => return Err(ErrorOrden(format!("Modo de redimensión desconocido: {x}"))),
                };
            }
            "-noasm" => o.sin_asm = true,
            "-version" => {
                o.version = true;
                return Ok(o);
            }
            "-progress" => o.progreso = true,
            "-quiet" => o.silencio = true,
            "-preset" if hay(1) => {
                i += 1;
                let p = Preset::desde_cwebp(args[i])
                    .ok_or_else(|| ErrorOrden(format!("Preset desconocido: {}", args[i])))?;
                c = config_preset(p, c.quality);
                preset = Some(p);
            }
            "-metadata" if hay(1) => {
                i += 1;
                for t in args[i].split(',') {
                    match t {
                        "all" => metadatos = Conservar::TODOS,
                        "none" => metadatos = Conservar::NINGUNO,
                        "exif" => metadatos.exif = true,
                        "icc" => metadatos.icc = true,
                        "xmp" => metadatos.xmp = true,
                        x => {
                            return Err(ErrorOrden(format!("Tipo de metadato desconocido: «{x}»")));
                        }
                    }
                }
            }
            "-v" => o.detalle = true,
            ENDEREZAR => enderezar = true,
            "--" => {
                if hay(1) {
                    o.entrada = Some(args[i + 1].to_string());
                }
                break;
            }
            x if x.starts_with('-') => {
                o.ayuda = Some(Ayuda::Larga);
                return Err(ErrorOrden(format!("Opción desconocida: «{x}»")));
            }
            x => o.entrada = Some(x.to_string()),
        }
        if error {
            return Err(ErrorOrden(format!("Valor no válido para «{a}»")));
        }
        i += 1;
    }

    if usar_nivel == 1 {
        // SAFETY: c es una configuración propia y válida en memoria.
        if unsafe { libwebp_sys::WebPConfigLosslessPreset(&mut c, nivel_sin_perdida) } == 0 {
            return Err(ErrorOrden(format!(
                "Nivel sin pérdida no válido (-z {nivel_sin_perdida})"
            )));
        }
    }

    let mut op = OpcionesWebp::desde_config(&c, preset);
    op.recorte = recorte;
    op.redimension = (redim != (0, 0)).then_some(Redimension {
        ancho: redim.0,
        alto: redim.1,
    });
    op.modo_redimension = modo;
    op.mezclar_alfa = mezclar;
    op.sin_alfa = !conservar_alfa;
    op.metadatos = metadatos;
    op.enderezar = enderezar;
    o.opciones = op;
    Ok(o)
}

/// Lee la imagen de entrada y la codifica, como hace cwebp con esta orden:
/// los metadatos solo se leen si se van a copiar, y `-s` hace la entrada YUV.
pub fn ejecutar(o: &OrdenCwebp, datos: &[u8], progreso: Option<Progreso>) -> Resultado<Codificado> {
    let op = &o.opciones;
    let img = match o.tamano_yuv {
        Some((w, h)) if w > 0 && h > 0 => entrada::leer_yuv(datos, w as u32, h as u32)?,
        _ => entrada::leer(
            datos,
            Lectura {
                conservar_alfa: !op.sin_alfa,
                // Para enderezar hay que leer el EXIF aunque no se copie. Es
                // una opción de Apolo: no afecta a la equivalencia con cwebp.
                metadatos: op.metadatos.alguno() || op.enderezar,
            },
        )?,
    };
    webp::codificar(&img, op, o.extras, progreso)
}

/// Si la orden cwebp de `escribir` da exactamente el mismo fichero que Apolo
/// con estas opciones. Deja de darlo con las opciones propias de Apolo.
pub fn es_equivalente(op: &OpcionesWebp) -> bool {
    !op.enderezar
}

/// La orden de `apolo webp`: la de cwebp más las opciones propias de Apolo.
pub fn escribir_apolo(op: &OpcionesWebp) -> Vec<String> {
    let mut a = escribir(op);
    if op.enderezar {
        a.push(ENDEREZAR.into());
    }
    a
}

/// La orden cwebp más corta que da estas opciones (sin entrada ni salida).
pub fn escribir(op: &OpcionesWebp) -> Vec<String> {
    let base = OpcionesWebp::desde_config(&config_base(op.preset), op.preset);
    let mut a: Vec<String> = Vec::new();
    fn par(a: &mut Vec<String>, o: &str, v: String) {
        a.push(o.into());
        a.push(v);
    }

    if let Some(p) = op.preset {
        par(&mut a, "-preset", p.nombre_cwebp().into());
    }
    if op.calidad != base.calidad {
        par(&mut a, "-q", op.calidad.to_string());
    }
    if op.calidad_alfa != base.calidad_alfa {
        par(&mut a, "-alpha_q", op.calidad_alfa.to_string());
    }
    if op.metodo != base.metodo {
        par(&mut a, "-m", op.metodo.to_string());
    }
    if op.segmentos != base.segmentos {
        par(&mut a, "-segments", op.segmentos.to_string());
    }
    if op.tamano_objetivo != base.tamano_objetivo {
        par(&mut a, "-size", op.tamano_objetivo.to_string());
    }
    if op.psnr_objetivo != base.psnr_objetivo {
        par(&mut a, "-psnr", op.psnr_objetivo.to_string());
    }
    if op.sns != base.sns {
        par(&mut a, "-sns", op.sns.to_string());
    }
    if op.fuerza_filtro != base.fuerza_filtro {
        par(&mut a, "-f", op.fuerza_filtro.to_string());
    }
    if op.nitidez_filtro != base.nitidez_filtro {
        par(&mut a, "-sharpness", op.nitidez_filtro.to_string());
    }
    if op.pasadas != base.pasadas {
        par(&mut a, "-pass", op.pasadas.to_string());
    }
    if (op.calidad_minima, op.calidad_maxima) != (base.calidad_minima, base.calidad_maxima) {
        a.push("-qrange".into());
        a.push(op.calidad_minima.to_string());
        a.push(op.calidad_maxima.to_string());
    }
    if op.limite_particion != base.limite_particion {
        par(&mut a, "-partition_limit", op.limite_particion.to_string());
    }
    if op.preprocesado != base.preprocesado {
        par(&mut a, "-pre", op.preprocesado.to_string());
    }
    if op.compresion_alfa != base.compresion_alfa {
        par(&mut a, "-alpha_method", op.compresion_alfa.to_string());
    }
    if op.filtrado_alfa != base.filtrado_alfa {
        let f = match op.filtrado_alfa {
            FiltradoAlfa::None => "none",
            FiltradoAlfa::Fast => "fast",
            FiltradoAlfa::Best => "best",
        };
        par(&mut a, "-alpha_filter", f.into());
    }
    if op.pista != base.pista {
        let h = match op.pista {
            Pista::Photo => Some("photo"),
            Pista::Picture => Some("picture"),
            Pista::Graph => Some("graph"),
            Pista::Ninguna => None,
        };
        if let Some(h) = h {
            par(&mut a, "-hint", h.into());
        }
    }
    if op.casi_sin_perdida != base.casi_sin_perdida && op.sin_perdida {
        par(&mut a, "-near_lossless", op.casi_sin_perdida.to_string());
    } else if op.sin_perdida && !base.sin_perdida {
        a.push("-lossless".into());
    }
    let mut bandera = |si: bool, b: bool, o: &str| {
        if si && !b {
            a.push(o.into());
        }
    };
    bandera(op.autofiltro, base.autofiltro, "-af");
    bandera(op.emular_jpeg, base.emular_jpeg, "-jpeg_like");
    bandera(op.poca_memoria, base.poca_memoria, "-low_memory");
    bandera(op.yuv_nitido, base.yuv_nitido, "-sharp_yuv");
    bandera(op.sin_alfa, false, "-noalpha");
    if op.filtro_fuerte != base.filtro_fuerte {
        a.push(
            if op.filtro_fuerte {
                "-strong"
            } else {
                "-nostrong"
            }
            .into(),
        );
    }
    if op.exacto != base.exacto {
        a.push(
            if op.exacto {
                "-exact"
            } else {
                "-alpha_cleanup"
            }
            .into(),
        );
    }
    for _ in base.hilos..op.hilos {
        a.push("-mt".into());
    }
    if let Some(r) = op.recorte {
        a.extend([
            "-crop".into(),
            r.x.to_string(),
            r.y.to_string(),
            r.ancho.to_string(),
            r.alto.to_string(),
        ]);
    }
    if let Some(r) = op.redimension {
        a.extend(["-resize".into(), r.ancho.to_string(), r.alto.to_string()]);
    }
    if op.modo_redimension != ModoRedimension::Siempre {
        let m = match op.modo_redimension {
            ModoRedimension::SoloReducir => "down_only",
            ModoRedimension::SoloAmpliar => "up_only",
            ModoRedimension::Siempre => "always",
        };
        a.extend(["-resize_mode".into(), m.into()]);
    }
    if let Some(color) = op.mezclar_alfa {
        a.extend(["-blend_alpha".into(), format!("0x{color:06x}")]);
    }
    let m = op.metadatos;
    if m == Conservar::TODOS {
        a.extend(["-metadata".into(), "all".into()]);
    } else if m.alguno() {
        let t: Vec<&str> = [(m.exif, "exif"), (m.icc, "icc"), (m.xmp, "xmp")]
            .into_iter()
            .filter_map(|(si, n)| si.then_some(n))
            .collect();
        a.extend(["-metadata".into(), t.join(",")]);
    }
    a
}

/// La orden entera como texto, para enseñarla y copiarla.
pub fn texto(op: &OpcionesWebp, entrada: &str, salida: &str) -> String {
    let mut partes = vec!["cwebp".to_string()];
    partes.extend(escribir(op));
    partes.push(citar(entrada));
    partes.push("-o".into());
    partes.push(citar(salida));
    partes.join(" ")
}

fn citar(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-/+,:=@".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn strtol_como_c() {
        assert_eq!(strtol("80", 0), Some(80));
        assert_eq!(strtol("010", 0), Some(8));
        assert_eq!(strtol("0x1f", 0), Some(31));
        assert_eq!(strtol("  42abc", 0), Some(42));
        assert_eq!(strtol("-3", 0), Some(-3));
        assert_eq!(strtol("abc", 0), None);
        assert_eq!(strtol("c0e0d0", 16), Some(0xc0e0d0));
        assert_eq!(strtol("0xc0e0d0", 16), Some(0xc0e0d0));
        assert_eq!(strtol("0", 0), Some(0));
    }

    #[test]
    fn strtod_como_c() {
        assert_eq!(strtod("75"), Some(75.0));
        assert_eq!(strtod("82.5x"), Some(82.5));
        assert_eq!(strtod("1e2"), Some(100.0));
        assert_eq!(strtod(".5"), Some(0.5));
        assert_eq!(strtod("x"), None);
    }

    #[test]
    fn el_preset_reescribe_lo_anterior() {
        let o = leer(&["-sns", "10", "-preset", "photo", "a.png"]).unwrap();
        let foto =
            OpcionesWebp::desde_config(&config_preset(Preset::Photo, 75.0), Some(Preset::Photo));
        assert_eq!(o.opciones.sns, foto.sns);
        assert_eq!(o.entrada.as_deref(), Some("a.png"));
    }

    #[test]
    fn q_anula_z() {
        let z = leer(&["-z", "9"]).unwrap().opciones;
        assert!(z.sin_perdida);
        let zq = leer(&["-z", "9", "-q", "50"]).unwrap().opciones;
        assert!(!zq.sin_perdida);
        assert_eq!(zq.calidad, 50.0);
    }

    #[test]
    fn ida_y_vuelta() {
        let casos: &[&[&str]] = &[
            &[],
            &["-q", "82.5", "-m", "6"],
            &["-preset", "drawing", "-q", "90", "-sharp_yuv"],
            &["-lossless", "-z", "9"],
            &["-near_lossless", "60", "-exact"],
            &["-size", "20000", "-pass", "10", "-qrange", "10", "90"],
            &["-mt", "-mt", "-af", "-strong", "-sharpness", "3"],
            &[
                "-crop",
                "1",
                "2",
                "30",
                "40",
                "-resize",
                "20",
                "0",
                "-resize_mode",
                "down_only",
            ],
            &[
                "-blend_alpha",
                "0xc0e0d0",
                "-noalpha",
                "-metadata",
                "exif,icc",
            ],
            &[
                "-alpha_q",
                "50",
                "-alpha_method",
                "0",
                "-alpha_filter",
                "best",
                "-hint",
                "graph",
            ],
            &[
                "-metadata",
                "all",
                "-jpeg_like",
                "-low_memory",
                "-segments",
                "2",
                "-pre",
                "2",
            ],
            &[
                "-preset",
                "text",
                "-nostrong",
                "-partition_limit",
                "50",
                "-psnr",
                "42",
            ],
        ];
        for caso in casos {
            let op = leer(caso).unwrap().opciones;
            let vuelta = leer(&escribir(&op)).unwrap().opciones;
            assert_eq!(op, vuelta, "{caso:?} → {:?}", escribir(&op));
        }
    }

    #[test]
    fn opciones_desconocidas() {
        assert!(leer(&["-noexiste"]).is_err());
        assert!(leer(&["-q", "abc"]).is_err());
        assert!(leer(&["-metadata", "gps"]).is_err());
    }
}
