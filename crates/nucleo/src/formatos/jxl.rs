//! JPEG XL con el `cjxl` de libjxl 0.12.0, compilado dentro de Apolo y con
//! clang, como el oficial (ADR 0021).
//!
//! Como con avifenc, Apolo llama al `main` de cjxl con los argumentos de la
//! orden: reconoce las opciones que enseña en el panel y deja las demás en
//! [`OpcionesJxl::otras`], tal cual.
//!
//! Con un JPEG de entrada, cjxl lo **recomprime sin pérdida** si no se le dice
//! otra cosa (`--lossless_jpeg=1`): el JPEG original se puede reconstruir
//! byte a byte desde el JXL. Para codificar sus píxeles hace falta
//! `--lossless_jpeg=0`.

use serde::{Deserialize, Serialize};

use apolo_avifjxl::Herramienta;

use crate::{Error, Resultado};

/// Las opciones de cjxl. Lo que no está puesto es lo de cjxl.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OpcionesJxl {
    /// `-q` (0 a 100; 100 es sin pérdida, 90 «sin pérdida visible»).
    pub calidad: Option<f32>,
    /// `-d` (0 a 25): la distancia visual. Por defecto, 1 (o 0 con JPEG).
    /// Excluye `-q`: la última que se ponga manda.
    pub distancia: Option<f32>,
    /// `-e` (1 a 10). Por defecto, 7.
    pub esfuerzo: Option<u8>,
    /// `-a`: la distancia del canal alfa. Por defecto, 0.
    pub distancia_alfa: Option<f32>,
    /// `-p`: más progresivo al decodificar.
    pub progresivo: bool,
    /// `-m 0|1`: VarDCT o modular.
    pub modular: Option<u8>,
    /// `--lossless_jpeg=0|1`: con un JPEG, recomprimirlo sin pérdida (sí, lo
    /// de cjxl) o codificar sus píxeles (no).
    pub jpeg_sin_perdida: bool,
    /// `--photon_noise_iso`: grano de película, como una ISO.
    pub ruido_iso: Option<u32>,
    /// `--epf` (-1 a 3): el filtro que conserva los bordes.
    pub epf: Option<i8>,
    /// `--gaborish=0|1`
    pub gaborish: Option<u8>,
    /// `--faster_decoding` (0 a 4)
    pub decodificacion_rapida: Option<u8>,
    /// Las demás opciones de cjxl, tal cual.
    pub otras: Vec<String>,
    /// De Apolo: girar según la orientación EXIF.
    pub enderezar: bool,
}

impl Default for OpcionesJxl {
    fn default() -> Self {
        OpcionesJxl {
            calidad: None,
            distancia: None,
            esfuerzo: None,
            distancia_alfa: None,
            progresivo: false,
            modular: None,
            jpeg_sin_perdida: true,
            ruido_iso: None,
            epf: None,
            gaborish: None,
            decodificacion_rapida: None,
            otras: Vec::new(),
            enderezar: false,
        }
    }
}

pub const ENDEREZAR: &str = "-apolo_enderezar";

fn mal(t: impl Into<String>) -> Error {
    Error::Configuracion(t.into())
}

impl OpcionesJxl {
    /// Los argumentos de cjxl, sin los ficheros.
    pub fn orden(&self) -> Vec<String> {
        let mut a: Vec<String> = Vec::new();
        let mut par = |k: &str, v: String| {
            a.push(k.into());
            a.push(v);
        };
        if let Some(d) = self.distancia {
            par("-d", d.to_string());
        } else if let Some(q) = self.calidad {
            par("-q", q.to_string());
        }
        if let Some(e) = self.esfuerzo {
            par("-e", e.to_string());
        }
        if let Some(d) = self.distancia_alfa {
            par("-a", d.to_string());
        }
        if let Some(m) = self.modular {
            par("-m", m.to_string());
        }
        if self.progresivo {
            a.push("-p".into());
        }
        if !self.jpeg_sin_perdida {
            a.push("--lossless_jpeg=0".into());
        }
        if let Some(n) = self.ruido_iso {
            a.push(format!("--photon_noise_iso={n}"));
        }
        if let Some(n) = self.epf {
            a.push(format!("--epf={n}"));
        }
        if let Some(n) = self.gaborish {
            a.push(format!("--gaborish={n}"));
        }
        if let Some(n) = self.decodificacion_rapida {
            a.push(format!("--faster_decoding={n}"));
        }
        a.extend(self.otras.iter().cloned());
        a
    }

    pub fn orden_apolo(&self) -> Vec<String> {
        let mut a = self.orden();
        if self.enderezar {
            a.push(ENDEREZAR.into());
        }
        a
    }
}

/// Una orden de cjxl entera.
#[derive(Debug, Clone, Default)]
pub struct OrdenCjxl {
    pub opciones: OpcionesJxl,
    /// Los ficheros: la entrada y la salida.
    pub ficheros: Vec<String>,
    /// Opciones aceptadas que no cambian el fichero, para avisar.
    pub ignoradas: Vec<String>,
}

/// Las opciones de cjxl 0.12.0: la letra (si tiene), el nombre largo y si
/// llevan valor.
const OPCIONES: &[(Option<char>, &str, bool)] = &[
    (Some('d'), "distance", true),
    (Some('q'), "quality", true),
    (Some('e'), "effort", true),
    (Some('a'), "alpha_distance", true),
    (Some('p'), "progressive", false),
    (None, "group_order", true),
    (None, "container", true),
    (None, "compress_boxes", true),
    (None, "brotli_effort", true),
    (Some('m'), "modular", true),
    (Some('j'), "lossless_jpeg", true),
    (None, "num_threads", true),
    (None, "photon_noise_iso", true),
    (None, "intensity_target", true),
    (Some('x'), "dec-hints", true),
    (None, "allow_jpeg_reconstruction", true),
    (None, "codestream_level", true),
    (None, "buffering", true),
    (None, "faster_decoding", true),
    (None, "premultiply", true),
    (None, "keep_invisible", true),
    (None, "center_x", true),
    (None, "center_y", true),
    (None, "progressive_ac", false),
    (None, "qprogressive_ac", false),
    (None, "progressive_dc", true),
    (None, "resampling", true),
    (None, "ec_resampling", true),
    (None, "already_downsampled", false),
    (None, "upsampling_mode", true),
    (None, "epf", true),
    (None, "gaborish", true),
    (None, "override_bitdepth", true),
    (None, "noise", true),
    (None, "jpeg_reconstruction_cfl", true),
    (None, "num_reps", true),
    (None, "streaming_input", false),
    (None, "streaming_output", false),
    (None, "output_mode", true),
    (None, "disable_output", false),
    (None, "dots", true),
    (None, "patches", true),
    (None, "frame_indexing", true),
    (None, "allow_expert_options", false),
    (None, "disable_perceptual_optimizations", false),
    (Some('I'), "iterations", true),
    (Some('C'), "modular_colorspace", true),
    (Some('g'), "modular_group_size", true),
    (Some('P'), "modular_predictor", true),
    (Some('E'), "modular_nb_prev_channels", true),
    (None, "modular_palette_colors", true),
    (None, "modular_lossy_palette", false),
    (Some('X'), "pre-compact", true),
    (Some('Y'), "post-compact", true),
    (Some('R'), "responsive", true),
    (None, "quiet", false),
    (Some('v'), "verbose", false),
    (Some('V'), "version", false),
    (Some('h'), "help", false),
];

fn decimal(v: &str, que: &str, min: f32, max: f32) -> Resultado<f32> {
    v.trim()
        .parse::<f32>()
        .ok()
        .filter(|n| (min..=max).contains(n))
        .ok_or_else(|| mal(format!("{que} {v}: va de {min} a {max}")))
}

fn entero(v: &str, que: &str, min: i64, max: i64) -> Resultado<i64> {
    v.trim()
        .parse::<i64>()
        .ok()
        .filter(|n| (min..=max).contains(n))
        .ok_or_else(|| mal(format!("{que} {v}: va de {min} a {max}")))
}

/// Lee una orden de cjxl: opciones y ficheros, en cualquier orden. Admite
/// `-d 1`, `--distance 1` y `--distance=1`, como cjxl.
pub fn leer_orden<S: AsRef<str>>(args: &[S]) -> Resultado<OrdenCjxl> {
    leer_orden_desde(&OpcionesJxl::default(), args)
}

/// Lo mismo, encima de unas opciones de partida (un preset).
pub fn leer_orden_desde<S: AsRef<str>>(base: &OpcionesJxl, args: &[S]) -> Resultado<OrdenCjxl> {
    let mut o = OrdenCjxl {
        opciones: base.clone(),
        ..Default::default()
    };
    let op = &mut o.opciones;
    let mut cola = args.iter().map(|s| s.as_ref().to_string());
    let mut tras_guiones = false;
    while let Some(a) = cola.next() {
        if tras_guiones || !a.starts_with('-') || a == "-" {
            o.ficheros.push(a);
            continue;
        }
        if a == "--" {
            tras_guiones = true;
            continue;
        }
        if a == ENDEREZAR {
            op.enderezar = true;
            continue;
        }
        let (largo, en_linea) = match a.strip_prefix("--") {
            Some(r) => match r.split_once('=') {
                Some((n, v)) => (n.to_string(), Some(v.to_string())),
                None => (r.to_string(), None),
            },
            None => (String::new(), None),
        };
        let encontrada = OPCIONES.iter().find(|(letra, nombre, _)| {
            if a.starts_with("--") {
                *nombre == largo
            } else {
                a.len() == 2 && *letra == a.chars().nth(1)
            }
        });
        let Some(&(_, nombre, con_valor)) = encontrada else {
            return Err(mal(format!("cjxl no tiene la opción {a}")));
        };
        let separado = en_linea.is_none();
        let valor = match (con_valor, en_linea) {
            (true, Some(v)) => Some(v),
            (true, None) => Some(
                cola.next()
                    .ok_or_else(|| mal(format!("falta el valor de {a}")))?,
            ),
            (false, Some(_)) => return Err(mal(format!("{a}: no lleva valor"))),
            (false, None) => None,
        };
        let v = valor.clone().unwrap_or_default();
        match nombre {
            "distance" => {
                op.distancia = Some(decimal(&v, &a, 0.0, 25.0)?);
                op.calidad = None;
            }
            "quality" => {
                op.calidad = Some(decimal(&v, &a, -1000.0, 100.0)?);
                op.distancia = None;
            }
            "effort" => op.esfuerzo = Some(entero(&v, &a, 1, 11)? as u8),
            "alpha_distance" => op.distancia_alfa = Some(decimal(&v, &a, 0.0, 25.0)?),
            "progressive" => op.progresivo = true,
            "modular" => op.modular = Some(entero(&v, &a, 0, 1)? as u8),
            "lossless_jpeg" => op.jpeg_sin_perdida = entero(&v, &a, 0, 1)? == 1,
            "photon_noise_iso" => op.ruido_iso = Some(entero(&v, &a, 0, u32::MAX as i64)? as u32),
            "epf" => op.epf = Some(entero(&v, &a, -1, 3)? as i8),
            "gaborish" => op.gaborish = Some(entero(&v, &a, 0, 1)? as u8),
            "faster_decoding" => op.decodificacion_rapida = Some(entero(&v, &a, 0, 4)? as u8),
            "quiet" | "verbose" | "num_reps" => o.ignoradas.push(a),
            "version" | "help" => return Err(mal(format!("{a}: Apolo no lo enseña"))),
            "disable_output" | "streaming_input" => {
                return Err(mal(format!("{a} no está en Apolo")));
            }
            "dec-hints"
                if v.contains("pathname")
                    || v.starts_with("exif=")
                    || v.starts_with("xmp=")
                    || v.starts_with("jumbf=") =>
            {
                return Err(mal(format!(
                    "{a} {v} no está en Apolo: los metadatos salen de la imagen abierta"
                )));
            }
            _ => {
                op.otras.push(a);
                if separado {
                    op.otras.extend(valor);
                }
            }
        }
    }
    Ok(o)
}

/// La orden para enseñar: `cjxl <entrada> <salida> <opciones>`.
pub fn texto(op: &OpcionesJxl, entrada: &str, salida: &str) -> String {
    let mut partes = vec![
        "cjxl".to_string(),
        crate::cwebp::citar(entrada),
        crate::cwebp::citar(salida),
    ];
    partes.extend(op.orden().iter().map(|a| crate::cwebp::citar(a)));
    partes.join(" ")
}

/// Codifica un fichero PNG o JPEG (`ext`) como `cjxl entrada salida <opciones>`.
pub fn codificar(entrada: &[u8], ext: &str, op: &OpcionesJxl) -> Resultado<Vec<u8>> {
    // cjxl también falla, pero solo dice «terminó con el código 1».
    let con_perdida =
        op.distancia.is_some_and(|d| d != 0.0) || op.calidad.is_some_and(|q| q != 100.0);
    if ext == "jpg" && op.jpeg_sin_perdida && con_perdida {
        return Err(mal(
            "con un JPEG, cjxl lo recomprime sin pérdida y no admite calidad: \
             desactiva «JPEG sin pérdida» (--lossless_jpeg=0)",
        ));
    }
    apolo_avifjxl::convertir(Herramienta::Cjxl, entrada, ext, &op.orden(), "jxl")
        .map_err(Error::Codificacion)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn ida_y_vuelta() {
        let o = leer_orden(&[
            "e.png",
            "s.jxl",
            "-q",
            "85",
            "--effort=4",
            "-p",
            "--lossless_jpeg",
            "0",
            "--epf=2",
            "-I",
            "50",
            "--resampling=2",
            "--quiet",
        ])
        .unwrap();
        assert_eq!(o.ficheros, ["e.png", "s.jxl"]);
        let op = &o.opciones;
        assert_eq!(op.calidad, Some(85.0));
        assert_eq!(op.esfuerzo, Some(4));
        assert!(op.progresivo);
        assert!(!op.jpeg_sin_perdida);
        assert_eq!(op.epf, Some(2));
        assert_eq!(op.otras, ["-I", "50", "--resampling=2"]);
        assert_eq!(o.ignoradas, ["--quiet"]);
        let otra = leer_orden(&op.orden()).unwrap();
        assert_eq!(&otra.opciones, op);
    }

    #[test]
    fn distancia_y_calidad_se_excluyen() {
        let o = leer_orden(&["-q", "80", "-d", "1.5"]).unwrap();
        assert_eq!(
            (o.opciones.calidad, o.opciones.distancia),
            (None, Some(1.5))
        );
        assert_eq!(o.opciones.orden(), ["-d", "1.5"]);
    }

    #[test]
    fn rechaza_lo_que_no_esta() {
        assert!(leer_orden(&["--inventada"]).is_err());
        assert!(leer_orden(&["-d"]).is_err());
        assert!(leer_orden(&["-e", "12"]).is_err());
        assert!(leer_orden(&["-p=1"]).is_err());
        assert!(leer_orden(&["--progressive=1"]).is_err());
    }
}
