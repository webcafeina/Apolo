//! AVIF con el `avifenc` de libavif 1.4.2 y aom 3.14.1, compilado dentro de
//! Apolo (ADR 0021).
//!
//! A diferencia de cjpeg u oxipng, aquí no hay nada trasladado a Rust: Apolo
//! llama al `main` de avifenc con los argumentos de la orden, así que lee las
//! opciones él mismo. Apolo solo reconoce las que enseña en el panel; las
//! demás que avifenc admite van en [`OpcionesAvif::otras`], tal cual y en su
//! orden.

use serde::{Deserialize, Serialize};

use apolo_avifjxl::Herramienta;

use crate::{Error, Resultado};

/// Las opciones de avifenc. Lo que no está puesto es lo de avifenc.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct OpcionesAvif {
    /// `-q` (0 a 100; 100 es sin pérdida). Por defecto, 60.
    pub calidad: Option<u8>,
    /// `--qalpha` (0 a 100). Por defecto, la de `-q`.
    pub calidad_alfa: Option<u8>,
    /// `-s` (0 a 10; 0 es la más lenta). Por defecto, 6.
    pub velocidad: Option<u8>,
    /// `-l`: todo sin pérdida.
    pub sin_perdida: bool,
    /// `-y`: 444, 422, 420 o 400. Por defecto, lo de la entrada.
    pub submuestreo: Option<String>,
    /// `-d`: 8, 10 o 12 bits.
    pub profundidad: Option<u8>,
    /// `--sharpyuv`: la conversión a YUV 4:2:0 de libwebp.
    pub sharpyuv: bool,
    /// `-r limited`: rango limitado en vez de completo.
    pub rango_limitado: bool,
    /// `-a tune=…`: psnr, ssim o iq.
    pub afinado: Option<String>,
    /// `-a sharpness=…` (0 a 7).
    pub nitidez: Option<u8>,
    /// `-p`: premultiplicar el color por el alfa.
    pub premultiplicar: bool,
    /// `--progressive`: por capas, para pintarlo poco a poco.
    pub progresivo: bool,
    /// `--ignore-exif`, `--ignore-xmp` e `--ignore-icc`.
    pub sin_exif: bool,
    pub sin_xmp: bool,
    pub sin_icc: bool,
    /// Las demás opciones de avifenc, tal cual.
    pub otras: Vec<String>,
    /// De Apolo: girar según la orientación EXIF.
    pub enderezar: bool,
}

pub const ENDEREZAR: &str = "-apolo_enderezar";

fn mal(t: impl Into<String>) -> Error {
    Error::Configuracion(t.into())
}

impl OpcionesAvif {
    /// Los argumentos de avifenc, sin los ficheros.
    pub fn orden(&self) -> Vec<String> {
        let mut a: Vec<String> = Vec::new();
        let mut par = |k: &str, v: String| {
            a.push(k.into());
            a.push(v);
        };
        if let Some(q) = self.calidad {
            par("-q", q.to_string());
        }
        if let Some(q) = self.calidad_alfa {
            par("--qalpha", q.to_string());
        }
        if let Some(s) = self.velocidad {
            par("-s", s.to_string());
        }
        if let Some(y) = &self.submuestreo {
            par("-y", y.clone());
        }
        if let Some(d) = self.profundidad {
            par("-d", d.to_string());
        }
        if self.rango_limitado {
            par("-r", "limited".into());
        }
        if let Some(t) = &self.afinado {
            par("-a", format!("tune={t}"));
        }
        if let Some(n) = self.nitidez {
            par("-a", format!("sharpness={n}"));
        }
        if self.sin_perdida {
            a.push("-l".into());
        }
        if self.sharpyuv {
            a.push("--sharpyuv".into());
        }
        if self.premultiplicar {
            a.push("-p".into());
        }
        if self.progresivo {
            a.push("--progressive".into());
        }
        if self.sin_exif {
            a.push("--ignore-exif".into());
        }
        if self.sin_xmp {
            a.push("--ignore-xmp".into());
        }
        if self.sin_icc {
            a.push("--ignore-icc".into());
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

/// Una orden de avifenc entera.
#[derive(Debug, Clone, Default)]
pub struct OrdenAvifenc {
    pub opciones: OpcionesAvif,
    /// Los ficheros: la entrada y, el último, la salida.
    pub ficheros: Vec<String>,
    /// Opciones aceptadas que no cambian el fichero, para avisar.
    pub ignoradas: Vec<String>,
}

/// Las opciones de avifenc que llevan un valor detrás. Las «actualizables»
/// (`-q:u`) se miran sin el sufijo.
const CON_VALOR: &[&str] = &[
    "-a",
    "--advanced",
    "--cicp",
    "--nclx",
    "--clap",
    "--clli",
    "-c",
    "--codec",
    "--creation-time",
    "--crop",
    "-d",
    "--depth",
    "--duration",
    "--exif",
    "--fps",
    "--timescale",
    "-g",
    "--grid",
    "--icc",
    "--imir",
    "--input-format",
    "--irot",
    "-j",
    "--jobs",
    "-k",
    "--keyframe",
    "--max",
    "--maxalpha",
    "--min",
    "--minalpha",
    "--modification-time",
    "--pasp",
    "--qalpha",
    "-q",
    "--qcolor",
    "--qgain-map",
    "-r",
    "--range",
    "--repetition-count",
    "--scaling-mode",
    "-s",
    "--speed",
    "--target-size",
    "--tilecolslog2",
    "--tilerowslog2",
    "--xmp",
    "-y",
    "--yuv",
    "-o",
    "--output",
];

const SIN_VALOR: &[&str] = &[
    "--autotiling",
    "--ignore-exif",
    "--ignore-gain-map",
    "--ignore-icc",
    "--ignore-profile",
    "--ignore-xmp",
    "--layered",
    "-l",
    "--lossless",
    "--mini",
    "-p",
    "--premultiply",
    "--progressive",
    "--sharpyuv",
    "--no-overwrite",
];

fn lleva_valor(nombre: &str) -> Option<bool> {
    let base = nombre
        .strip_suffix(":u")
        .or_else(|| nombre.strip_suffix(":update"))
        .unwrap_or(nombre);
    if CON_VALOR.contains(&base) {
        Some(true)
    } else if SIN_VALOR.contains(&base) {
        Some(false)
    } else {
        None
    }
}

fn numero(v: &str, que: &str, max: u8) -> Resultado<u8> {
    v.parse::<u8>()
        .ok()
        .filter(|n| *n <= max)
        .ok_or_else(|| mal(format!("{que} {v}: va de 0 a {max}")))
}

/// Lee una orden de avifenc: opciones y ficheros.
pub fn leer_orden<S: AsRef<str>>(args: &[S]) -> Resultado<OrdenAvifenc> {
    leer_orden_desde(&OpcionesAvif::default(), args)
}

/// Lo mismo, encima de unas opciones de partida (un preset).
pub fn leer_orden_desde<S: AsRef<str>>(base: &OpcionesAvif, args: &[S]) -> Resultado<OrdenAvifenc> {
    let mut o = OrdenAvifenc {
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
        let Some(con_valor) = lleva_valor(&a) else {
            return Err(match a.as_str() {
                "-h" | "--help" | "-V" | "--version" => mal(format!("{a}: Apolo no lo enseña")),
                "--stdin" => mal("--stdin no está en Apolo: la entrada es un fichero"),
                _ => mal(format!("avifenc no tiene la opción {a}")),
            });
        };
        let valor = if con_valor {
            Some(
                cola.next()
                    .ok_or_else(|| mal(format!("falta el valor de {a}")))?,
            )
        } else {
            None
        };
        let v = valor.clone().unwrap_or_default();
        match a.as_str() {
            "-q" | "--qcolor" => op.calidad = Some(numero(&v, &a, 100)?),
            "--qalpha" => op.calidad_alfa = Some(numero(&v, &a, 100)?),
            "-s" | "--speed" if v != "d" && v != "default" => {
                op.velocidad = Some(numero(&v, &a, 10)?)
            }
            "-y" | "--yuv" if v != "auto" => {
                if !matches!(v.as_str(), "444" | "422" | "420" | "400") {
                    return Err(mal(format!("{a} {v}: tiene que ser 444, 422, 420 o 400")));
                }
                op.submuestreo = Some(v);
            }
            "-y" | "--yuv" => op.submuestreo = None,
            "-d" | "--depth" if matches!(v.as_str(), "8" | "10" | "12") => {
                op.profundidad = v.parse().ok()
            }
            "-r" | "--range" => match v.as_str() {
                "limited" | "l" => op.rango_limitado = true,
                "full" | "f" => op.rango_limitado = false,
                _ => return Err(mal(format!("{a} {v}: tiene que ser limited o full"))),
            },
            "-a" | "--advanced" if v.starts_with("tune=") => {
                op.afinado = Some(v["tune=".len()..].to_string())
            }
            "-a" | "--advanced" if v.starts_with("sharpness=") => {
                op.nitidez = Some(numero(&v["sharpness=".len()..], "sharpness", 7)?)
            }
            "-l" | "--lossless" => op.sin_perdida = true,
            "--sharpyuv" => op.sharpyuv = true,
            "-p" | "--premultiply" => op.premultiplicar = true,
            "--progressive" => op.progresivo = true,
            "--ignore-exif" => op.sin_exif = true,
            "--ignore-xmp" => op.sin_xmp = true,
            "--ignore-icc" | "--ignore-profile" => op.sin_icc = true,
            "--no-overwrite" => o.ignoradas.push(a),
            "-o" | "--output" | "--exif" | "--xmp" | "--icc" => {
                return Err(mal(format!(
                    "{a} no está en Apolo: la salida y los metadatos salen de la imagen abierta"
                )));
            }
            "-g" | "--grid" | "--layered" => {
                return Err(mal(format!(
                    "{a} no está en Apolo: es para varias imágenes"
                )));
            }
            _ => {
                op.otras.push(a);
                op.otras.extend(valor);
            }
        }
    }
    Ok(o)
}

/// La orden para enseñar: `avifenc <opciones> <entrada> <salida>`.
pub fn texto(op: &OpcionesAvif, entrada: &str, salida: &str) -> String {
    let mut partes = vec!["avifenc".to_string()];
    partes.extend(op.orden().iter().map(|a| crate::cwebp::citar(a)));
    partes.push(crate::cwebp::citar(entrada));
    partes.push(crate::cwebp::citar(salida));
    partes.join(" ")
}

/// Codifica un fichero PNG o JPEG (`ext`) como `avifenc <opciones>`.
pub fn codificar(entrada: &[u8], ext: &str, op: &OpcionesAvif) -> Resultado<Vec<u8>> {
    apolo_avifjxl::convertir(Herramienta::Avifenc, entrada, ext, &op.orden(), "avif")
        .map_err(Error::Codificacion)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn ida_y_vuelta() {
        let o = leer_orden(&[
            "-q",
            "70",
            "-s",
            "8",
            "-y",
            "420",
            "--sharpyuv",
            "-a",
            "tune=ssim",
            "-a",
            "sharpness=2",
            "--ignore-exif",
            "-a",
            "color:enable-chroma-deltaq=1",
            "-j",
            "2",
            "--autotiling",
            "e.png",
            "s.avif",
        ])
        .unwrap();
        assert_eq!(o.ficheros, ["e.png", "s.avif"]);
        let op = &o.opciones;
        assert_eq!(op.calidad, Some(70));
        assert_eq!(op.velocidad, Some(8));
        assert_eq!(op.submuestreo.as_deref(), Some("420"));
        assert_eq!(op.afinado.as_deref(), Some("ssim"));
        assert_eq!(op.nitidez, Some(2));
        assert_eq!(
            op.otras,
            [
                "-a",
                "color:enable-chroma-deltaq=1",
                "-j",
                "2",
                "--autotiling"
            ]
        );
        let otra = leer_orden(&op.orden()).unwrap();
        assert_eq!(&otra.opciones, op);
        assert!(otra.ficheros.is_empty());
    }

    #[test]
    fn rechaza_lo_que_no_esta() {
        assert!(leer_orden(&["--grid", "2x2"]).is_err());
        assert!(leer_orden(&["--inventada"]).is_err());
        assert!(leer_orden(&["-q"]).is_err());
        assert!(leer_orden(&["-q", "101"]).is_err());
    }
}
