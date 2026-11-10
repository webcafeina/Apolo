//! Vectorizar (ADR 0038 y 0039): de un mapa de bits a SVG, PDF, EPS o DXF, con el motor de `apolo-vector`.
//!
//! Es una operación aparte de [`crate::salida`]: el resultado no es un mapa de bits del tamaño del original.
//! Lo que sí lo es, la **vista previa** (el dibujo rasterizado con resvg), sirve para el comparador y para
//! medir con la SSIMULACRA 2 de siempre, frente al original aplanado sobre blanco (como lo ve el motor).
//!
//! **Modo por nota**: con un objetivo, se parte en dos el intervalo del detalle (de un mínimo a 100), como
//! [`crate::salida`] con la calidad, y de lo probado se queda el dibujo **más ligero** (menos nodos) que llega
//! a la nota; si ninguno llega, el de mejor nota. Más detalle no siempre da más nota, así que no basta con el
//! detalle más bajo que llega.

use serde::{Deserialize, Serialize};

pub use apolo_vector::{
    Documento, Estadisticas, Estilo, FormatoVector, NivelDxf, OpcionesExportar, TipoImagen,
};

use crate::entrada::Imagen;
use crate::error::{Error, Resultado};
use crate::{orientacion, proceso, trs, vista};

/// Las imágenes de más píxeles que esto se reducen antes de vectorizar (como Vectorizer.ai): el dibujo no
/// gana nada y el tiempo crece con el área.
pub const MAX_PIXELES: u64 = 2_000_000;

/// Lo que se pide al vectorizar y al exportar. Es lo que guardan la ventana, la CLI y el MCP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AjusteVector {
    pub tipo: TipoImagen,
    /// Colores como mucho (`None`: los que tenga). Con un máximo o una paleta, sin degradados.
    pub colores: Option<u32>,
    /// Paleta fija, en `#rrggbb`.
    pub paleta: Vec<String>,
    /// Detalle, de 0 a 100 (75 por defecto). Con objetivo, es el **mínimo**: la búsqueda no baja de aquí.
    pub detalle: u8,
    /// Nota SSIMULACRA 2 que buscar (el dibujo más ligero que llega).
    pub objetivo: Option<f32>,
    pub degradados: bool,
    /// Las formas de menos área (en píxeles cuadrados) se descartan.
    pub area_minima: f64,
    pub estilo: Estilo,
    pub grosor: f64,
    /// Color del trazo en `#rrggbb` (`None`: el de cada capa; en «trazar bordes», negro).
    pub color_trazo: Option<String>,
    pub nivel_dxf: NivelDxf,
}

impl Default for AjusteVector {
    fn default() -> Self {
        let v = apolo_vector::OpcionesVector::default();
        let e = OpcionesExportar::default();
        AjusteVector {
            tipo: v.tipo,
            colores: v.colores,
            paleta: Vec::new(),
            detalle: v.detalle,
            objetivo: None,
            degradados: v.degradados,
            area_minima: v.area_minima,
            estilo: e.estilo,
            grosor: e.grosor,
            color_trazo: None,
            nivel_dxf: e.nivel_dxf,
        }
    }
}

/// `#rrggbb` (o `rrggbb`) a RGB.
pub fn color(texto: &str) -> Resultado<[u8; 3]> {
    let t = texto.trim().trim_start_matches('#');
    let malo = || {
        Error::Configuracion(format!(
            "{} «{texto}» ({})",
            trs!("color no válido", "invalid color"),
            trs!("se escribe #rrggbb", "write it as #rrggbb")
        ))
    };
    if t.len() != 6 || !t.is_ascii() {
        return Err(malo());
    }
    let c = |i: usize| u8::from_str_radix(&t[i..i + 2], 16).map_err(|_| malo());
    Ok([c(0)?, c(2)?, c(4)?])
}

impl AjusteVector {
    fn opciones(&self, detalle: u8) -> Resultado<apolo_vector::OpcionesVector> {
        Ok(apolo_vector::OpcionesVector {
            tipo: self.tipo,
            colores: self.colores,
            paleta: self
                .paleta
                .iter()
                .map(|c| color(c))
                .collect::<Resultado<_>>()?,
            detalle: detalle.min(100),
            area_minima: self.area_minima.max(0.0),
            degradados: self.degradados,
        })
    }

    /// Si dos ajustes dan el mismo dibujo (solo cambian las opciones de exportar): entonces no hace falta
    /// volver a vectorizar.
    pub fn mismo_dibujo(&self, otro: &AjusteVector) -> bool {
        self.tipo == otro.tipo
            && self.colores == otro.colores
            && self.paleta == otro.paleta
            && self.detalle == otro.detalle
            && self.objetivo == otro.objetivo
            && self.degradados == otro.degradados
            && self.area_minima == otro.area_minima
    }

    /// Las opciones de exportar.
    pub fn exportar(&self) -> Resultado<OpcionesExportar> {
        Ok(OpcionesExportar {
            estilo: self.estilo,
            grosor: self.grosor.clamp(0.01, 1000.0),
            color_trazo: self.color_trazo.as_deref().map(color).transpose()?,
            nivel_dxf: self.nivel_dxf,
        })
    }
}

/// Lo que encontró la búsqueda de una nota.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct HalladoVector {
    /// El detalle del dibujo elegido: el más ligero que llega o, si ninguno llega, el de mejor nota.
    pub detalle: u8,
    pub nota: f64,
    pub alcanzada: bool,
    pub pruebas: u32,
}

/// El resultado de vectorizar.
#[derive(Debug, Clone)]
pub struct Vectorizado {
    pub documento: Documento,
    /// La vista previa, RGBA del tamaño del documento.
    pub vista: Vec<u8>,
    /// La imagen que recibió el motor (enderezada, reducida y aplanada sobre blanco), RGBA del mismo tamaño:
    /// la referencia de las medidas.
    pub referencia: Vec<u8>,
    /// El tipo que se usó (resuelto si era automático).
    pub tipo: TipoImagen,
    /// El detalle con el que se hizo.
    pub detalle: u8,
    /// La nota SSIMULACRA 2 de la vista previa (`None` si mide menos de 8 × 8 o no se midió).
    pub nota: Option<f64>,
    pub hallado: Option<HalladoVector>,
    /// Si se redujo la imagen (pasaba de [`MAX_PIXELES`]): de qué tamaño era.
    pub reducida_desde: Option<(u32, u32)>,
    pub estadisticas: Estadisticas,
}

/// Progreso de 0 a 100; si devuelve `false`, se cancela.
pub type Progreso<'a> = &'a mut dyn FnMut(i32) -> bool;

/// La imagen que recibe el motor: enderezada según su EXIF, reducida si pasa de [`MAX_PIXELES`] y
/// aplanada sobre blanco. Devuelve también el tamaño de antes de reducir, si se redujo.
pub fn preparar(img: &Imagen) -> Resultado<(apolo_vector::Rgba, Option<(u32, u32)>)> {
    let girada = if orientacion::leer(img.metadatos.exif.as_deref()) != 1 {
        orientacion::enderezar(img)?
    } else {
        None
    };
    let (w, h, rgba) = vista::rgba(girada.as_ref().unwrap_or(img))?;
    let area = w as u64 * h as u64;
    let (w2, h2, rgba, antes) = if area > MAX_PIXELES {
        let k = (MAX_PIXELES as f64 / area as f64).sqrt();
        let (nw, nh) = (
            ((w as f64 * k).floor() as u32).max(1),
            ((h as f64 * k).floor() as u32).max(1),
        );
        let p = proceso::Proceso {
            redimension: Some(proceso::Redimension {
                solo_reducir: true,
                ..proceso::Redimension::medidas(Some(nw), Some(nh))
            }),
            ..Default::default()
        };
        let (a, b, v) = proceso::aplicar(&p, w, h, &rgba)?;
        (a, b, v, Some((w, h)))
    } else {
        (w, h, rgba, None)
    };
    Ok((apolo_vector::Rgba::nueva(w2, h2, rgba), antes))
}

/// Vectoriza. Con `objetivo`, busca el detalle más bajo que llega a la nota (de `detalle` a 100).
pub fn vectorizar(
    img: &Imagen,
    a: &AjusteVector,
    mut progreso: Option<Progreso>,
) -> Resultado<Vectorizado> {
    let (im, reducida_desde) = preparar(img)?;
    vectorizar_rgba(&im, reducida_desde, a, &mut progreso)
}

/// Como [`vectorizar`], con la imagen ya preparada.
pub fn vectorizar_rgba(
    im: &apolo_vector::Rgba,
    reducida_desde: Option<(u32, u32)>,
    a: &AjusteVector,
    progreso: &mut Option<Progreso>,
) -> Resultado<Vectorizado> {
    let tipo = apolo_vector::tipo_efectivo(im, a.tipo);
    let probar = |detalle: u8, medir_nota: bool| -> Resultado<Vectorizado> {
        let doc =
            apolo_vector::vectorizar(im, &a.opciones(detalle)?).map_err(Error::Configuracion)?;
        let vista = apolo_vector::vista::rasterizar(&doc).map_err(Error::Configuracion)?;
        let nota = if medir_nota {
            crate::medir::nota(&im.px, &vista, im.ancho, im.alto)?
        } else {
            None
        };
        Ok(Vectorizado {
            estadisticas: doc.estadisticas(),
            documento: doc,
            vista,
            referencia: Vec::new(),
            tipo,
            detalle,
            nota,
            hallado: None,
            reducida_desde,
        })
    };
    let mut seguir = |hechas: usize| -> Resultado<()> {
        // Como mucho ocho pruebas.
        let p = (hechas * 100 / 8).min(99) as i32;
        if progreso.as_mut().is_none_or(|f| f(p)) {
            Ok(())
        } else {
            Err(Error::Cancelado)
        }
    };
    let mut v = match a.objetivo {
        None => {
            seguir(0)?;
            probar(a.detalle, true)?
        }
        Some(objetivo) => {
            let objetivo = objetivo as f64;
            let mut probadas: Vec<Vectorizado> = Vec::new();
            let mut nota_de = |d: u8, probadas: &mut Vec<Vectorizado>| -> Resultado<f64> {
                if let Some(v) = probadas.iter().find(|v| v.detalle == d) {
                    return Ok(v.nota.unwrap_or(f64::NEG_INFINITY));
                }
                seguir(probadas.len())?;
                let v = probar(d, true)?;
                let n = v.nota.ok_or_else(|| {
                    Error::Configuracion(
                        trs!(
                            "la imagen es demasiado pequeña para medirla (8×8)",
                            "the image is too small to measure (8×8)"
                        )
                        .into(),
                    )
                })?;
                probadas.push(v);
                Ok(n)
            };
            // `bajo` no llega (o es el mínimo pedido, que se prueba si hace falta) y `alto` llega (o es 100).
            let minimo = a.detalle.min(100);
            let (mut bajo, mut alto) = (minimo, 100u8);
            if nota_de(minimo, &mut probadas)? >= objetivo {
                alto = minimo;
            }
            while alto - bajo > 1 {
                let medio = (bajo + alto) / 2;
                if nota_de(medio, &mut probadas)? >= objetivo {
                    alto = medio;
                } else {
                    bajo = medio;
                }
            }
            nota_de(alto, &mut probadas)?;
            // Más detalle no siempre da más nota (en la bandera de EE. UU., el detalle 0 da 76,9 y el 100,
            // 74,2): de lo probado, el más ligero (menos nodos) que llega; si ninguno llega, el de mejor nota.
            let pruebas = probadas.len() as u32;
            let nota_v = |v: &Vectorizado| v.nota.unwrap_or(f64::NEG_INFINITY);
            let llegan = probadas.iter().any(|v| nota_v(v) >= objetivo);
            let i = (0..probadas.len())
                .filter(|&i| !llegan || nota_v(&probadas[i]) >= objetivo)
                .min_by(|&i, &j| {
                    let (a, b) = (&probadas[i], &probadas[j]);
                    if llegan {
                        a.estadisticas
                            .nodos
                            .cmp(&b.estadisticas.nodos)
                            .then(a.detalle.cmp(&b.detalle))
                    } else {
                        nota_v(b)
                            .total_cmp(&nota_v(a))
                            .then(a.detalle.cmp(&b.detalle))
                    }
                })
                .unwrap();
            let mut v = probadas.swap_remove(i);
            v.hallado = Some(HalladoVector {
                detalle: v.detalle,
                nota: nota_v(&v),
                alcanzada: llegan,
                pruebas,
            });
            v
        }
    };
    v.referencia = im.px.clone();
    Ok(v)
}

/// Los contornos y los nodos sobre el dibujo apagado, RGBA del tamaño del documento.
pub fn contornos(doc: &Documento) -> Resultado<Vec<u8>> {
    apolo_vector::vista::rasterizar_contornos(doc).map_err(Error::Configuracion)
}

/// El fichero, en el formato pedido.
pub fn exportar(doc: &Documento, formato: FormatoVector, a: &AjusteVector) -> Resultado<Vec<u8>> {
    Ok(apolo_vector::exportar(doc, formato, &a.exportar()?))
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::entrada::{self, Lectura};

    /// Un PNG con un círculo rojo sobre blanco.
    fn png(lado: u32) -> Vec<u8> {
        let mut px = Vec::new();
        for y in 0..lado {
            for x in 0..lado {
                let d = ((x as f64 + 0.5 - lado as f64 / 2.0).powi(2)
                    + (y as f64 + 0.5 - lado as f64 / 2.0).powi(2))
                .sqrt();
                let t = (lado as f64 / 3.0 - d + 0.5).clamp(0.0, 1.0);
                let m = |a: f64, b: f64| (a * t + b * (1.0 - t)).round() as u8;
                px.extend([m(220.0, 255.0), m(30.0, 255.0), m(30.0, 255.0), 255]);
            }
        }
        let mut out = Vec::new();
        {
            let mut e = png::Encoder::new(&mut out, lado, lado);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header().unwrap().write_image_data(&px).unwrap();
        }
        out
    }

    /// Un PNG con un círculo rojo sobre transparente.
    fn png_transparente(lado: u32) -> Vec<u8> {
        let mut px = Vec::new();
        for y in 0..lado {
            for x in 0..lado {
                let d = ((x as f64 + 0.5 - lado as f64 / 2.0).powi(2)
                    + (y as f64 + 0.5 - lado as f64 / 2.0).powi(2))
                .sqrt();
                let a = ((lado as f64 / 3.0 - d + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
                px.extend([220, 30, 30, a]);
            }
        }
        let mut out = Vec::new();
        {
            let mut e = png::Encoder::new(&mut out, lado, lado);
            e.set_color(png::ColorType::Rgba);
            e.set_depth(png::BitDepth::Eight);
            e.write_header().unwrap().write_image_data(&px).unwrap();
        }
        out
    }

    #[test]
    fn transparente_sin_fondo() {
        let v = vectorizar(&leer(&png_transparente(96)), &AjusteVector::default(), None).unwrap();
        assert!(v.documento.fondo.is_none());
        let svg = String::from_utf8(
            exportar(&v.documento, FormatoVector::Svg, &AjusteVector::default()).unwrap(),
        )
        .unwrap();
        assert!(!svg.contains("<rect"), "{svg}");
        assert!(v.nota.unwrap() > 80.0, "{:?}", v.nota);
    }

    fn leer(datos: &[u8]) -> Imagen {
        entrada::leer(datos, Lectura::default()).unwrap()
    }

    #[test]
    fn un_circulo() {
        let v = vectorizar(&leer(&png(96)), &AjusteVector::default(), None).unwrap();
        assert_eq!((v.documento.ancho, v.documento.alto), (96, 96));
        assert_eq!(v.vista.len(), 96 * 96 * 4);
        assert!(v.nota.unwrap() > 80.0, "{:?}", v.nota);
        assert!(v.reducida_desde.is_none());
        let svg = String::from_utf8(
            exportar(&v.documento, FormatoVector::Svg, &AjusteVector::default()).unwrap(),
        )
        .unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn por_nota() {
        let a = AjusteVector {
            detalle: 0,
            objetivo: Some(70.0),
            ..Default::default()
        };
        let v = vectorizar(&leer(&png(96)), &a, None).unwrap();
        let h = v.hallado.unwrap();
        assert!(h.alcanzada && h.nota >= 70.0, "{h:?}");
        assert!(h.pruebas <= 9, "{h:?}");
        assert_eq!(v.detalle, h.detalle);
    }

    #[test]
    fn grande_se_reduce() {
        let v = vectorizar(&leer(&png(1600)), &AjusteVector::default(), None).unwrap();
        assert_eq!(v.reducida_desde, Some((1600, 1600)));
        let (w, h) = (v.documento.ancho as u64, v.documento.alto as u64);
        assert!(w * h <= MAX_PIXELES && w >= 1400, "{w} × {h}");
    }

    #[test]
    fn cancelar() {
        let mut no = |_| false;
        let r = vectorizar(&leer(&png(64)), &AjusteVector::default(), Some(&mut no));
        assert!(matches!(r, Err(Error::Cancelado)));
    }

    #[test]
    fn colores() {
        assert_eq!(color("#ff8000").unwrap(), [255, 128, 0]);
        assert_eq!(color("00FF00").unwrap(), [0, 255, 0]);
        assert!(color("#f80").is_err());
        assert!(color("#zzzzzz").is_err());
    }

    #[test]
    fn ajuste_en_json() {
        let a: AjusteVector = serde_json::from_str(
            r#"{"tipo":"pixel_art","estilo":"trazar_bordes","nivel_dxf":"arcos"}"#,
        )
        .unwrap();
        assert_eq!(a.tipo, TipoImagen::PixelArt);
        assert_eq!(a.estilo, Estilo::TrazarBordes);
        assert_eq!(a.nivel_dxf, NivelDxf::Arcos);
        assert_eq!(a.detalle, 75);
    }
}
