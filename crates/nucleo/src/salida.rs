//! Codificar a cualquier formato de salida (ADR 0020 y 0021): WebP, JPEG, PNG,
//! QOI, AVIF o JPEG XL, con el proceso delante si lo hay.
//!
//! Sin proceso, cada formato va por su camino exacto, el que da el mismo
//! fichero que su herramienta oficial: cwebp (`webp::codificar`), cjpeg (la
//! imagen leída como cjpeg, desde los bytes originales), oxipng (el PNG tal
//! cual), qoiconv (el PNG leído como stb_image), y avifenc y cjxl (el PNG o
//! el JPEG tal cual, a la herramienta compilada dentro). Con proceso (o enderezando
//! en JPEG, PNG y QOI), se codifican los píxeles ya procesados, y la orden
//! de la herramienta ya no da ese fichero: [`motivo`] dice por qué.

use serde::{Deserialize, Serialize};

use crate::entrada::{Formato, Imagen, Pixeles};
use crate::formatos::{avif, jxl, png, qoi};
use crate::jpeg::{self, OpcionesJpeg};
use crate::proceso::{self, Proceso};
use crate::webp::{self, Estadisticas, Extras, OpcionesWebp, Progreso};
use crate::{Resultado, cwebp, orientacion, vista};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatoSalida {
    #[default]
    Webp,
    Jpeg,
    Png,
    Qoi,
    Avif,
    Jxl,
}

impl FormatoSalida {
    pub const TODOS: [FormatoSalida; 6] = [
        FormatoSalida::Webp,
        FormatoSalida::Jpeg,
        FormatoSalida::Png,
        FormatoSalida::Qoi,
        FormatoSalida::Avif,
        FormatoSalida::Jxl,
    ];

    pub fn extension(self) -> &'static str {
        match self {
            FormatoSalida::Webp => "webp",
            FormatoSalida::Jpeg => "jpg",
            FormatoSalida::Png => "png",
            FormatoSalida::Qoi => "qoi",
            FormatoSalida::Avif => "avif",
            FormatoSalida::Jxl => "jxl",
        }
    }

    pub fn nombre(self) -> &'static str {
        match self {
            FormatoSalida::Webp => "WebP",
            FormatoSalida::Jpeg => "JPEG",
            FormatoSalida::Png => "PNG",
            FormatoSalida::Qoi => "QOI",
            FormatoSalida::Avif => "AVIF",
            FormatoSalida::Jxl => "JPEG XL",
        }
    }

    /// La herramienta oficial cuya orden se enseña.
    pub fn herramienta(self) -> &'static str {
        match self {
            FormatoSalida::Webp => "cwebp",
            FormatoSalida::Jpeg => "cjpeg",
            FormatoSalida::Png => "oxipng",
            FormatoSalida::Qoi => "qoiconv",
            FormatoSalida::Avif => "avifenc",
            FormatoSalida::Jxl => "cjxl",
        }
    }

    /// Si la herramienta lee un fichero de este formato **y está comprobado**
    /// que lo lee igual que Apolo.
    pub fn herramienta_lee(self, f: Formato) -> bool {
        match self {
            FormatoSalida::Webp => matches!(
                f,
                Formato::Png
                    | Formato::Jpeg
                    | Formato::Tiff
                    | Formato::WebP
                    | Formato::Pnm
                    | Formato::Yuv
            ),
            FormatoSalida::Jpeg => matches!(f, Formato::Png | Formato::Jpeg | Formato::Pnm),
            FormatoSalida::Png | FormatoSalida::Qoi => f == Formato::Png,
            FormatoSalida::Avif | FormatoSalida::Jxl => matches!(f, Formato::Png | Formato::Jpeg),
        }
    }
}

/// Todo lo que decide el fichero de salida: el formato, las opciones de cada
/// formato (se guardan todas, para no perderlas al cambiar) y el proceso.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Ajuste {
    pub formato: FormatoSalida,
    pub webp: OpcionesWebp,
    pub jpeg: OpcionesJpeg,
    pub png: png::OpcionesPng,
    pub avif: avif::OpcionesAvif,
    pub jxl: jxl::OpcionesJxl,
    pub proceso: Proceso,
    /// Buscar la calidad más baja que da esta nota SSIMULACRA 2 (ADR 0022),
    /// en los formatos con pérdida. La orden lleva la calidad encontrada.
    pub objetivo: Option<f32>,
}

impl Ajuste {
    /// Enderezar está en el proceso; las opciones de cada formato lo llevan
    /// también (por la CLI: `-apolo_enderezar`).
    pub fn enderezar(&self) -> bool {
        self.proceso.enderezar
            || match self.formato {
                FormatoSalida::Webp => self.webp.enderezar,
                FormatoSalida::Jpeg => self.jpeg.enderezar,
                FormatoSalida::Png => self.png.enderezar,
                FormatoSalida::Qoi => false,
                FormatoSalida::Avif => self.avif.enderezar,
                FormatoSalida::Jxl => self.jxl.enderezar,
            }
    }
}

/// Por qué la orden de la herramienta no da exactamente el fichero de Apolo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Motivo {
    /// Apolo la giró según el EXIF; la herramienta no gira.
    Enderezada,
    /// Recortada, redimensionada o con la paleta reducida por Apolo.
    Procesada,
    /// La herramienta no lee este formato de entrada.
    FormatoSinHerramienta,
}

pub fn motivo(entrada: Formato, a: &Ajuste) -> Option<Motivo> {
    if !a.proceso.vacio() {
        Some(Motivo::Procesada)
    } else if a.enderezar() {
        Some(Motivo::Enderezada)
    } else if !a.formato.herramienta_lee(entrada) {
        Some(Motivo::FormatoSinHerramienta)
    } else {
        None
    }
}

/// La orden de la herramienta, para enseñarla o copiarla. `formato` es el de
/// la imagen de entrada: con un JPEG, cjxl no lleva lo que no cuenta.
pub fn orden(a: &Ajuste, formato: Formato, entrada: &str, salida: &str) -> String {
    match a.formato {
        FormatoSalida::Webp => cwebp::texto(&a.webp, entrada, salida),
        FormatoSalida::Jpeg => jpeg::opciones::texto(&a.jpeg, entrada, salida),
        FormatoSalida::Png => png::texto(&a.png, entrada, salida),
        FormatoSalida::Qoi => qoi::texto(entrada, salida),
        FormatoSalida::Avif => avif::texto(&a.avif, entrada, salida),
        FormatoSalida::Jxl => jxl::texto(&jxl_efectivas(a, formato), entrada, salida),
    }
}

/// Las opciones de cjxl que cuentan: si recibe el JPEG original tal cual (sin
/// proceso ni enderezar) y lo recomprime sin pérdida, sin calidad ni grano.
fn jxl_efectivas(a: &Ajuste, formato: Formato) -> jxl::OpcionesJxl {
    if formato == Formato::Jpeg && motivo(formato, a).is_none() {
        a.jxl.para_jpeg()
    } else {
        a.jxl.clone()
    }
}

/// Los argumentos, sin ficheros (para los presets y las órdenes pegadas).
pub fn argumentos(a: &Ajuste) -> Vec<String> {
    match a.formato {
        FormatoSalida::Webp => cwebp::escribir_apolo(&a.webp),
        FormatoSalida::Jpeg => a.jpeg.orden_apolo(),
        FormatoSalida::Png => a.png.orden_apolo(),
        FormatoSalida::Qoi => Vec::new(),
        FormatoSalida::Avif => a.avif.orden_apolo(),
        FormatoSalida::Jxl => a.jxl.orden_apolo(),
    }
}

pub struct Codificado {
    pub datos: Vec<u8>,
    pub ancho: u32,
    pub alto: u32,
    /// Solo WebP las da.
    pub estadisticas: Option<Estadisticas>,
    /// Con una nota objetivo, la calidad que se encontró.
    pub hallada: Option<Hallada>,
}

/// Lo que encontró la búsqueda de una nota objetivo.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Hallada {
    /// La calidad más baja que llega a la nota (o 100, si ni así llega).
    pub calidad: u8,
    /// La nota SSIMULACRA 2 de esa calidad.
    pub nota: f64,
    /// Si se llegó a la nota.
    pub alcanzada: bool,
    /// Cuántas calidades se probaron.
    pub pruebas: u32,
}

/// Si el formato admite buscar una nota: los que tienen una calidad. Con un
/// JPEG que cjxl recomprime sin pérdida, no.
pub fn admite_objetivo(a: &Ajuste, entrada: Formato) -> bool {
    match a.formato {
        FormatoSalida::Webp => !a.webp.sin_perdida,
        FormatoSalida::Jpeg => true,
        FormatoSalida::Avif => !a.avif.sin_perdida,
        FormatoSalida::Jxl => {
            !(entrada == Formato::Jpeg && motivo(entrada, a).is_none() && a.jxl.jpeg_sin_perdida)
        }
        FormatoSalida::Png | FormatoSalida::Qoi => false,
    }
}

/// El ajuste con la calidad `q` (0–100) en el formato elegido, sin objetivo.
pub fn con_calidad(a: &Ajuste, q: u8) -> Ajuste {
    let mut b = a.clone();
    b.objetivo = None;
    match b.formato {
        FormatoSalida::Webp => b.webp.calidad = q as f32,
        FormatoSalida::Jpeg => b.jpeg.calidad = vec![q as f32],
        FormatoSalida::Avif => b.avif.calidad = Some(q),
        FormatoSalida::Jxl => {
            b.jxl.calidad = Some(q as f32);
            b.jxl.distancia = None;
        }
        FormatoSalida::Png | FormatoSalida::Qoi => {}
    }
    b
}

/// La imagen que recibe el codificador, en RGBA: enderezada y procesada si
/// hace falta. Es la referencia de las medidas (ADR 0022).
pub fn referencia(img: &Imagen, a: &Ajuste) -> Resultado<(u32, u32, Vec<u8>)> {
    let enderezar = a.enderezar() && orientacion::leer(img.metadatos.exif.as_deref()) != 1;
    let girada = if enderezar {
        orientacion::enderezar(img)?
    } else {
        None
    };
    let (w, h, rgba) = vista::rgba(girada.as_ref().unwrap_or(img))?;
    if a.proceso.vacio() {
        Ok((w, h, rgba))
    } else {
        proceso::aplicar(&a.proceso, w, h, &rgba)
    }
}

/// Codifica `img` (y sus bytes originales, `datos`) según el ajuste.
/// `progreso` lo pregunta libwebp; en los demás, solo se mira al empezar (y
/// entre prueba y prueba al buscar una nota).
pub fn codificar(
    datos: &[u8],
    img: &Imagen,
    a: &Ajuste,
    progreso: Option<Progreso>,
) -> Resultado<Codificado> {
    match a.objetivo {
        Some(nota) if admite_objetivo(a, img.formato) => {
            buscar_nota(datos, img, a, nota as f64, progreso)
        }
        _ => codificar_tal_cual(datos, img, a, progreso),
    }
}

/// Busca la calidad más baja con la que la nota SSIMULACRA 2 llega a
/// `objetivo`, partiendo en dos el intervalo 0–100: siete pruebas, y una más
/// si ni la 99 llega. Se
/// supone que más calidad da más nota, que es lo normal; si en algún tramo no
/// lo es, la calidad que sale llega a la nota igual, aunque quizá no sea la
/// más baja posible.
fn buscar_nota(
    datos: &[u8],
    img: &Imagen,
    a: &Ajuste,
    objetivo: f64,
    mut progreso: Option<Progreso>,
) -> Resultado<Codificado> {
    let (w, h, ref_rgba) = referencia(img, a)?;
    let mut probadas: Vec<(u8, Codificado, f64)> = Vec::new();
    let mut probar = |q: u8, progreso: &mut Option<Progreso>| -> Resultado<f64> {
        if let Some((_, _, n)) = probadas.iter().find(|(c, _, _)| *c == q) {
            return Ok(*n);
        }
        if !progreso.as_mut().is_none_or(|f| f(0)) {
            return Err(crate::Error::Cancelado);
        }
        let c = codificar_tal_cual(datos, img, &con_calidad(a, q), None)?;
        let (rw, rh, rgba) = vista::decodificar(&c.datos)?;
        if (rw, rh) != (w, h) {
            return Err(crate::Error::Configuracion(
                "no se puede buscar una nota si el formato cambia el tamaño (como -resize de cwebp)"
                    .into(),
            ));
        }
        let n = crate::medir::nota(&ref_rgba, &rgba, w, h)?.ok_or_else(|| {
            crate::Error::Configuracion("la imagen es demasiado pequeña para medirla (8×8)".into())
        })?;
        probadas.push((q, c, n));
        Ok(n)
    };
    // `alto` llega (o es 100, que se da por bueno hasta probarlo al final) y
    // `bajo` no llega. La 100 se prueba solo si hace falta: es la más lenta, y
    // en avifenc es sin pérdida.
    let (mut bajo, mut alto) = (0u8, 100u8);
    while alto - bajo > 1 {
        let medio = (bajo + alto) / 2;
        if probar(medio, &mut progreso)? >= objetivo {
            alto = medio;
        } else {
            bajo = medio;
        }
    }
    let alcanzada = probar(alto, &mut progreso)? >= objetivo;
    let pruebas = probadas.len() as u32;
    let i = probadas.iter().position(|(c, _, _)| *c == alto).unwrap();
    let (calidad, mut c, nota) = probadas.swap_remove(i);
    c.hallada = Some(Hallada {
        calidad,
        nota,
        alcanzada,
        pruebas,
    });
    Ok(c)
}

fn codificar_tal_cual(
    datos: &[u8],
    img: &Imagen,
    a: &Ajuste,
    mut progreso: Option<Progreso>,
) -> Resultado<Codificado> {
    let seguir = |p: &mut Option<Progreso>| p.as_mut().is_none_or(|f| f(0));
    if !seguir(&mut progreso) {
        return Err(crate::Error::Cancelado);
    }
    let enderezar = a.enderezar() && orientacion::leer(img.metadatos.exif.as_deref()) != 1;

    // Con proceso, todos los formatos parten de los píxeles procesados.
    let procesada: Option<(u32, u32, Vec<u8>)> = if a.proceso.vacio() {
        None
    } else {
        let girada = if enderezar {
            orientacion::enderezar(img)?
        } else {
            None
        };
        let base = girada.as_ref().unwrap_or(img);
        let (w, h, rgba) = vista::rgba(base)?;
        Some(proceso::aplicar(&a.proceso, w, h, &rgba)?)
    };

    match a.formato {
        FormatoSalida::Webp => {
            let mut op = a.webp.clone();
            let r = match procesada {
                None => {
                    op.enderezar = enderezar || op.enderezar;
                    webp::codificar(img, &op, Extras::default(), progreso)?
                }
                Some((w, h, rgba)) => {
                    op.enderezar = false;
                    let img2 = Imagen {
                        ancho: w,
                        alto: h,
                        formato: img.formato,
                        pixeles: Pixeles::Rgba(rgba),
                        metadatos: img.metadatos.clone(),
                    };
                    webp::codificar(&img2, &op, Extras::default(), progreso)?
                }
            };
            Ok(Codificado {
                ancho: r.ancho,
                alto: r.alto,
                datos: r.datos,
                estadisticas: Some(r.estadisticas),
                hallada: None,
            })
        }
        FormatoSalida::Jpeg => {
            let e = match rgba_si_hace_falta(img, procesada, enderezar)? {
                None => jpeg::leer(datos)?,
                Some((w, h, rgba)) => jpeg::EntradaJpeg::desde_rgba(w, h, &rgba),
            };
            let extra = jpeg::cjpeg::Extra {
                // Al reconstruir los píxeles, el perfil de color no se pierde.
                icc: if e.de_cjpeg {
                    None
                } else {
                    img.metadatos.icc.clone()
                },
                estricto: false,
            };
            let datos = jpeg::cjpeg::ejecutar(&e, &a.jpeg.orden(), &extra)?;
            Ok(Codificado {
                ancho: e.ancho,
                alto: e.alto,
                datos,
                estadisticas: None,
                hallada: None,
            })
        }
        FormatoSalida::Png => {
            let (png_origen, w, h) = match rgba_si_hace_falta(img, procesada, enderezar)? {
                None if img.formato == Formato::Png => (datos.to_vec(), img.ancho, img.alto),
                None => {
                    let (w, h, rgba) = vista::rgba(img)?;
                    (
                        png::png_de_pixeles(w, h, &rgba, img.metadatos.icc.as_deref())?,
                        w,
                        h,
                    )
                }
                Some((w, h, rgba)) => (
                    png::png_de_pixeles(w, h, &rgba, img.metadatos.icc.as_deref())?,
                    w,
                    h,
                ),
            };
            Ok(Codificado {
                datos: png::optimizar(&png_origen, &a.png)?,
                ancho: w,
                alto: h,
                estadisticas: None,
                hallada: None,
            })
        }
        FormatoSalida::Qoi => {
            let e = match rgba_si_hace_falta(img, procesada, enderezar)? {
                None if img.formato == Formato::Png => qoi::leer_png(datos)?,
                None => {
                    let (w, h, rgba) = vista::rgba(img)?;
                    qoi::desde_rgba(w, h, &rgba)
                }
                Some((w, h, rgba)) => qoi::desde_rgba(w, h, &rgba),
            };
            Ok(Codificado {
                datos: qoi::codificar(&e)?,
                ancho: e.ancho,
                alto: e.alto,
                estadisticas: None,
                hallada: None,
            })
        }
        FormatoSalida::Avif | FormatoSalida::Jxl => {
            // El PNG o el JPEG original tal cual; si no, un PNG con los píxeles
            // y el perfil.
            let (fichero, ext, w, h) = match rgba_si_hace_falta(img, procesada, enderezar)? {
                None if img.formato == Formato::Png => (datos.to_vec(), "png", img.ancho, img.alto),
                None if img.formato == Formato::Jpeg => {
                    (datos.to_vec(), "jpg", img.ancho, img.alto)
                }
                None => {
                    let (w, h, rgba) = vista::rgba(img)?;
                    let p = png::png_de_pixeles(w, h, &rgba, img.metadatos.icc.as_deref())?;
                    (p, "png", w, h)
                }
                Some((w, h, rgba)) => {
                    let p = png::png_de_pixeles(w, h, &rgba, img.metadatos.icc.as_deref())?;
                    (p, "png", w, h)
                }
            };
            let datos = if a.formato == FormatoSalida::Avif {
                avif::codificar(&fichero, ext, &a.avif)?
            } else {
                let op = if ext == "jpg" {
                    a.jxl.para_jpeg()
                } else {
                    a.jxl.clone()
                };
                jxl::codificar(&fichero, ext, &op)?
            };
            Ok(Codificado {
                datos,
                ancho: w,
                alto: h,
                estadisticas: None,
                hallada: None,
            })
        }
    }
}

/// Para todos menos WebP: los píxeles procesados, o los enderezados, o nada
/// (y entonces se va por el camino exacto de la herramienta).
fn rgba_si_hace_falta(
    img: &Imagen,
    procesada: Option<(u32, u32, Vec<u8>)>,
    enderezar: bool,
) -> Resultado<Option<(u32, u32, Vec<u8>)>> {
    if procesada.is_some() {
        return Ok(procesada);
    }
    if enderezar && let Some(girada) = orientacion::enderezar(img)? {
        return Ok(Some(vista::rgba(&girada)?));
    }
    Ok(None)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::entrada::{self, Lectura};

    fn foto_png() -> Vec<u8> {
        let webp = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus/foto.webp"),
        )
        .unwrap();
        let (w, h, rgba) = vista::decodificar_webp(&webp).unwrap();
        png::png_de_pixeles(w, h, &rgba, None).unwrap()
    }

    #[test]
    fn todos_los_formatos_salen_y_se_leen() {
        let datos = foto_png();
        let img = entrada::leer(&datos, Lectura::default()).unwrap();
        for f in FormatoSalida::TODOS {
            let a = Ajuste {
                formato: f,
                ..Default::default()
            };
            let r = codificar(&datos, &img, &a, None).unwrap();
            let (w, h, _) = vista::decodificar(&r.datos).unwrap();
            assert_eq!((w, h), (img.ancho, img.alto), "{f:?}");
            assert_eq!(motivo(Formato::Png, &a), None, "{f:?}");
        }
    }

    #[test]
    fn sin_proceso_jpeg_es_el_camino_de_cjpeg() {
        let datos = foto_png();
        let img = entrada::leer(&datos, Lectura::default()).unwrap();
        let mut a = Ajuste {
            formato: FormatoSalida::Jpeg,
            ..Default::default()
        };
        a.jpeg.calidad = vec![60.0];
        let r = codificar(&datos, &img, &a, None).unwrap();
        let directo = jpeg::cjpeg::ejecutar(
            &jpeg::leer(&datos).unwrap(),
            &a.jpeg.orden(),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(r.datos, directo);
    }

    #[test]
    fn la_nota_objetivo_da_la_calidad_mas_baja_que_llega() {
        let datos = foto_png();
        let img = entrada::leer(&datos, Lectura::default()).unwrap();
        let (w, h, referencia) = referencia(&img, &Ajuste::default()).unwrap();
        let nota_de = |a: &Ajuste| {
            let c = codificar(&datos, &img, a, None).unwrap();
            let (_, _, rgba) = vista::decodificar(&c.datos).unwrap();
            crate::medir::nota(&referencia, &rgba, w, h)
                .unwrap()
                .unwrap()
        };
        for formato in [
            FormatoSalida::Webp,
            FormatoSalida::Jpeg,
            FormatoSalida::Avif,
            FormatoSalida::Jxl,
        ] {
            let a = Ajuste {
                formato,
                objetivo: Some(75.0),
                ..Default::default()
            };
            let c = codificar(&datos, &img, &a, None).unwrap();
            let h_ = c.hallada.expect("con objetivo hay calidad hallada");
            assert!(h_.alcanzada && h_.nota >= 75.0, "{formato:?}: {h_:?}");
            assert!(h_.pruebas <= 9, "{formato:?}: {h_:?}");
            // La misma calidad, codificada aparte, da el mismo fichero.
            assert_eq!(
                codificar(&datos, &img, &con_calidad(&a, h_.calidad), None)
                    .unwrap()
                    .datos,
                c.datos
            );
            if h_.calidad > 0 {
                assert!(
                    nota_de(&con_calidad(&a, h_.calidad - 1)) < 75.0,
                    "{formato:?}: {h_:?}"
                );
            }
        }
        let png = Ajuste {
            formato: FormatoSalida::Png,
            objetivo: Some(75.0),
            ..Default::default()
        };
        assert!(!admite_objetivo(&png, Formato::Png));
        assert!(
            codificar(&datos, &img, &png, None)
                .unwrap()
                .hallada
                .is_none()
        );
    }

    #[test]
    fn con_proceso_cambian_las_medidas_y_el_motivo() {
        let datos = foto_png();
        let img = entrada::leer(&datos, Lectura::default()).unwrap();
        let mut a = Ajuste {
            formato: FormatoSalida::Qoi,
            ..Default::default()
        };
        a.proceso.redimension = Some(proceso::Redimension {
            ancho: Some(64),
            ..Default::default()
        });
        let r = codificar(&datos, &img, &a, None).unwrap();
        assert_eq!(r.ancho, 64);
        assert_eq!(motivo(Formato::Png, &a), Some(Motivo::Procesada));
        assert_eq!(
            motivo(
                Formato::Heic,
                &Ajuste {
                    formato: FormatoSalida::Png,
                    ..Default::default()
                }
            ),
            Some(Motivo::FormatoSinHerramienta)
        );
    }
}
