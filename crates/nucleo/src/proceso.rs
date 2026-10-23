//! El proceso de la imagen antes de codificarla (ADR 0020): recortar,
//! redimensionar y reducir la paleta, en ese orden, como Squoosh. Trabaja
//! sobre RGBA de 8 bits y vale para todos los formatos de salida.
//!
//! Con proceso, la orden de la herramienta oficial ya no da el mismo fichero:
//! ni cwebp ni cjpeg redimensionan como Apolo. La interfaz lo dice.

use crate::{tr, trs};
use fast_image_resize as fr;
use serde::{Deserialize, Serialize};

use crate::{Error, Resultado};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Filtro {
    /// El más nítido al reducir: lo que usa Squoosh por defecto.
    #[default]
    Lanczos3,
    Mitchell,
    CatmullRom,
    Bilineal,
    /// Píxel más cercano: para pixel art.
    Vecino,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recorte {
    pub x: u32,
    pub y: u32,
    pub ancho: u32,
    pub alto: u32,
}

/// A qué tamaño se lleva la imagen (ADR 0032). El lado mayor y el
/// porcentaje se guardan como tales, no en píxeles: un preset «1920 de lado
/// mayor» vale igual para una foto apaisada que para una vertical.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Medida {
    /// Si falta uno de los dos, se calcula para mantener la proporción.
    Medidas {
        ancho: Option<u32>,
        alto: Option<u32>,
    },
    /// El lado más largo: el ancho si es apaisada o cuadrada, el alto si es
    /// vertical. El otro, en proporción.
    LadoMayor(u32),
    /// Los dos lados, al tanto por ciento.
    Porcentaje(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RedimensionJson", into = "RedimensionJson")]
pub struct Redimension {
    pub medida: Medida,
    pub filtro: Filtro,
    /// Mezclar en RGB lineal (más correcto; lo de Squoosh).
    pub lineal: bool,
    /// Las medidas son un máximo: una imagen que ya cabe se queda como está,
    /// nunca se agranda. Apagado, como Squoosh, se lleva siempre a esas
    /// medidas. Lo enciende el agente (ADR 0023), y la ventana por defecto
    /// (ADR 0032): agrandar empeora la imagen.
    pub solo_reducir: bool,
}

impl Default for Redimension {
    fn default() -> Self {
        Redimension {
            medida: Medida::Medidas {
                ancho: None,
                alto: None,
            },
            filtro: Filtro::Lanczos3,
            lineal: true,
            solo_reducir: false,
        }
    }
}

impl Redimension {
    /// Al ancho y al alto dados (cualquiera de los dos puede faltar).
    pub fn medidas(ancho: Option<u32>, alto: Option<u32>) -> Self {
        Redimension {
            medida: Medida::Medidas { ancho, alto },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModoMedida {
    Medidas,
    LadoMayor,
    Porcentaje,
}

/// La redimensión tal como se guarda: plana, con todos los campos. Sin
/// `modo` (los presets de antes de la v0.18), se deduce de lo que haya; sin
/// `solo_reducir`, es `false`, para que un preset viejo haga lo mismo que
/// hacía.
#[derive(Serialize, Deserialize)]
#[serde(default)]
struct RedimensionJson {
    modo: Option<ModoMedida>,
    ancho: Option<u32>,
    alto: Option<u32>,
    lado_mayor: Option<u32>,
    porcentaje: Option<f64>,
    filtro: Filtro,
    lineal: bool,
    solo_reducir: bool,
}

impl Default for RedimensionJson {
    fn default() -> Self {
        RedimensionJson {
            modo: None,
            ancho: None,
            alto: None,
            lado_mayor: None,
            porcentaje: None,
            filtro: Filtro::Lanczos3,
            lineal: true,
            solo_reducir: false,
        }
    }
}

impl TryFrom<RedimensionJson> for Redimension {
    type Error = String;
    fn try_from(j: RedimensionJson) -> Result<Self, String> {
        let modo = j.modo.unwrap_or(if j.lado_mayor.is_some() {
            ModoMedida::LadoMayor
        } else if j.porcentaje.is_some() {
            ModoMedida::Porcentaje
        } else {
            ModoMedida::Medidas
        });
        let medida = match modo {
            ModoMedida::Medidas => Medida::Medidas {
                ancho: j.ancho,
                alto: j.alto,
            },
            ModoMedida::LadoMayor => match j.lado_mayor {
                Some(l) if l > 0 => Medida::LadoMayor(l),
                _ => {
                    return Err(trs!(
                        "el lado mayor tiene que ser de 1 px o más",
                        "the longest side must be 1 px or more"
                    )
                    .into());
                }
            },
            ModoMedida::Porcentaje => match j.porcentaje {
                Some(p) if p.is_finite() && p > 0.0 => Medida::Porcentaje(p),
                _ => {
                    return Err(trs!(
                        "el porcentaje tiene que ser mayor que 0",
                        "the percentage must be greater than 0"
                    )
                    .into());
                }
            },
        };
        Ok(Redimension {
            medida,
            filtro: j.filtro,
            lineal: j.lineal,
            solo_reducir: j.solo_reducir,
        })
    }
}

impl From<Redimension> for RedimensionJson {
    fn from(r: Redimension) -> Self {
        let mut j = RedimensionJson {
            filtro: r.filtro,
            lineal: r.lineal,
            solo_reducir: r.solo_reducir,
            ..Default::default()
        };
        match r.medida {
            Medida::Medidas { ancho, alto } => {
                j.modo = Some(ModoMedida::Medidas);
                (j.ancho, j.alto) = (ancho, alto);
            }
            Medida::LadoMayor(l) => {
                j.modo = Some(ModoMedida::LadoMayor);
                j.lado_mayor = Some(l);
            }
            Medida::Porcentaje(p) => {
                j.modo = Some(ModoMedida::Porcentaje);
                j.porcentaje = Some(p);
            }
        }
        j
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Paleta {
    /// De 2 a 256.
    pub colores: u32,
    /// Tramado, de 0 (nada) a 1 (todo).
    pub tramado: f32,
}

impl Default for Paleta {
    fn default() -> Self {
        Paleta {
            colores: 256,
            tramado: 1.0,
        }
    }
}

/// Lo que se le hace a la imagen antes de codificarla. Vacío por defecto.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Proceso {
    /// Girar según la orientación EXIF, antes que todo lo demás. No cuenta
    /// para [`Proceso::vacio`]: tiene su propio aviso.
    pub enderezar: bool,
    pub recorte: Option<Recorte>,
    pub redimension: Option<Redimension>,
    pub paleta: Option<Paleta>,
}

impl Proceso {
    /// Si no recorta, ni redimensiona, ni reduce la paleta.
    pub fn vacio(&self) -> bool {
        self.recorte.is_none() && self.redimension.is_none() && self.paleta.is_none()
    }

    /// Las medidas que saldrán, sin hacer nada todavía.
    pub fn medidas(&self, ancho: u32, alto: u32) -> (u32, u32) {
        let (mut w, mut h) = (ancho, alto);
        if let Some(r) = self.recorte {
            w = r.ancho.min(ancho.saturating_sub(r.x));
            h = r.alto.min(alto.saturating_sub(r.y));
        }
        if let Some(r) = self.redimension {
            (w, h) = destino(w, h, r);
        }
        (w, h)
    }

    /// El proceso sin lo que no cambia nada en una imagen de `ancho` × `alto`
    /// (ya enderezada): un recorte que la coge entera y una redimensión que
    /// la deja como está (pedir 1920 sin agrandar en una de 1200). Así no se
    /// pierde la orden equivalente por un proceso que no hace nada.
    pub fn efectivo(&self, ancho: u32, alto: u32) -> Proceso {
        let mut p = *self;
        let (mut w, mut h) = (ancho, alto);
        if let Some(r) = p.recorte {
            if r.x == 0 && r.y == 0 && r.ancho >= ancho && r.alto >= alto {
                p.recorte = None;
            } else {
                w = r.ancho.min(ancho.saturating_sub(r.x));
                h = r.alto.min(alto.saturating_sub(r.y));
            }
        }
        if let Some(r) = p.redimension
            && destino(w, h, r) == (w, h)
        {
            p.redimension = None;
        }
        p
    }
}

fn destino(w: u32, h: u32, r: Redimension) -> (u32, u32) {
    let prop =
        |a: u32, de: u32, a_de: u32| ((a as f64 * a_de as f64 / de as f64).round() as u32).max(1);
    let (nw, nh) = match r.medida {
        Medida::Medidas { ancho, alto } => match (ancho, alto) {
            (Some(a), Some(b)) => (a.max(1), b.max(1)),
            (Some(a), None) => (a.max(1), prop(a, w, h)),
            (None, Some(b)) => (prop(b, h, w), b.max(1)),
            (None, None) => (w, h),
        },
        Medida::LadoMayor(l) if w >= h => (l.max(1), prop(l, w, h)),
        Medida::LadoMayor(l) => (prop(l, h, w), l.max(1)),
        Medida::Porcentaje(p) => {
            let a = |v: u32| ((v as f64 * p / 100.0).round() as u32).max(1);
            (a(w), a(h))
        }
    };
    if r.solo_reducir && (nw > w || nh > h) {
        return (w, h);
    }
    (nw, nh)
}

/// Aplica el proceso a píxeles RGBA. Devuelve las medidas nuevas y los píxeles.
pub fn aplicar(p: &Proceso, ancho: u32, alto: u32, rgba: &[u8]) -> Resultado<(u32, u32, Vec<u8>)> {
    let (mut w, mut h, mut px) = (ancho, alto, rgba.to_vec());
    if let Some(r) = p.recorte {
        (w, h, px) = recortar(w, h, &px, r)?;
    }
    if let Some(r) = p.redimension {
        (w, h, px) = redimensionar(w, h, &px, r)?;
    }
    if let Some(pal) = p.paleta {
        px = reducir_paleta(w, h, &px, pal)?;
    }
    Ok((w, h, px))
}

fn recortar(w: u32, h: u32, px: &[u8], r: Recorte) -> Resultado<(u32, u32, Vec<u8>)> {
    if r.x >= w || r.y >= h || r.ancho == 0 || r.alto == 0 {
        return Err(Error::Configuracion(
            trs!(
                "el recorte queda fuera de la imagen",
                "the crop is outside the image"
            )
            .into(),
        ));
    }
    let (nw, nh) = (r.ancho.min(w - r.x), r.alto.min(h - r.y));
    let mut v = Vec::with_capacity(nw as usize * nh as usize * 4);
    for y in r.y..r.y + nh {
        let ini = (y as usize * w as usize + r.x as usize) * 4;
        v.extend_from_slice(&px[ini..ini + nw as usize * 4]);
    }
    Ok((nw, nh, v))
}

fn redimensionar(w: u32, h: u32, px: &[u8], r: Redimension) -> Resultado<(u32, u32, Vec<u8>)> {
    let (nw, nh) = destino(w, h, r);
    if (nw, nh) == (w, h) {
        return Ok((w, h, px.to_vec()));
    }
    let mal = |e: &dyn std::fmt::Display| {
        Error::Codificacion(tr!("al redimensionar: {e}", "while resizing: {e}"))
    };
    let alg = match r.filtro {
        Filtro::Vecino => fr::ResizeAlg::Nearest,
        Filtro::Lanczos3 => fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3),
        Filtro::Mitchell => fr::ResizeAlg::Convolution(fr::FilterType::Mitchell),
        Filtro::CatmullRom => fr::ResizeAlg::Convolution(fr::FilterType::CatmullRom),
        Filtro::Bilineal => fr::ResizeAlg::Convolution(fr::FilterType::Bilinear),
    };
    // La transparencia se premultiplica (use_alpha) para que no salgan halos
    // del color que hubiera debajo de lo transparente.
    let opciones = fr::ResizeOptions::new().resize_alg(alg).use_alpha(true);
    let mut redim = fr::Resizer::new();
    let origen = fr::images::Image::from_vec_u8(w, h, px.to_vec(), fr::PixelType::U8x4)
        .map_err(|e| mal(&e))?;
    if r.lineal && r.filtro != Filtro::Vecino {
        let mapa = fr::create_srgb_mapper();
        let mut lin = fr::images::Image::new(w, h, fr::PixelType::U16x4);
        mapa.forward_map(&origen, &mut lin).map_err(|e| mal(&e))?;
        let mut dst = fr::images::Image::new(nw, nh, fr::PixelType::U16x4);
        redim
            .resize(&lin, &mut dst, &opciones)
            .map_err(|e| mal(&e))?;
        let mut fin = fr::images::Image::new(nw, nh, fr::PixelType::U8x4);
        mapa.backward_map(&dst, &mut fin).map_err(|e| mal(&e))?;
        Ok((nw, nh, fin.into_vec()))
    } else {
        let mut dst = fr::images::Image::new(nw, nh, fr::PixelType::U8x4);
        redim
            .resize(&origen, &mut dst, &opciones)
            .map_err(|e| mal(&e))?;
        Ok((nw, nh, dst.into_vec()))
    }
}

fn reducir_paleta(w: u32, h: u32, px: &[u8], p: Paleta) -> Resultado<Vec<u8>> {
    let mal = |e: imagequant::Error| {
        Error::Codificacion(tr!(
            "al reducir la paleta: {e}",
            "while reducing the palette: {e}"
        ))
    };
    let mut liq = imagequant::new();
    liq.set_max_colors(p.colores.clamp(2, 256)).map_err(mal)?;
    let pixeles: Vec<imagequant::RGBA> = px
        .as_chunks::<4>()
        .0
        .iter()
        .map(|c| imagequant::RGBA::new(c[0], c[1], c[2], c[3]))
        .collect();
    let mut img = liq
        .new_image(pixeles, w as usize, h as usize, 0.0)
        .map_err(mal)?;
    let mut res = liq.quantize(&mut img).map_err(mal)?;
    res.set_dithering_level(p.tramado.clamp(0.0, 1.0))
        .map_err(mal)?;
    let (paleta, indices) = res.remapped(&mut img).map_err(mal)?;
    Ok(indices
        .iter()
        .flat_map(|&i| {
            let c = paleta[i as usize];
            [c.r, c.g, c.b, c.a]
        })
        .collect())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn degradado(w: u32, h: u32) -> Vec<u8> {
        (0..w * h)
            .flat_map(|i| [(i % w * 255 / w) as u8, (i / w * 255 / h) as u8, 128, 255])
            .collect()
    }

    #[test]
    fn solo_reducir_no_agranda() {
        let r = |ancho, solo_reducir| Redimension {
            solo_reducir,
            ..Redimension::medidas(Some(ancho), None)
        };
        // Mayor que la imagen: con solo_reducir se queda; sin él, como Squoosh, agranda.
        assert_eq!(destino(400, 300, r(800, true)), (400, 300));
        assert_eq!(destino(400, 300, r(800, false)), (800, 600));
        // Menor: reduce igual.
        assert_eq!(destino(400, 300, r(200, true)), (200, 150));
    }

    #[test]
    fn medidas_con_proporcion() {
        let p = Proceso {
            recorte: Some(Recorte {
                x: 10,
                y: 0,
                ancho: 200,
                alto: 100,
            }),
            redimension: Some(Redimension::medidas(Some(50), None)),
            ..Default::default()
        };
        assert_eq!(p.medidas(400, 300), (50, 25));
        let (w, h, px) = aplicar(&p, 400, 300, &degradado(400, 300)).unwrap();
        assert_eq!((w, h, px.len()), (50, 25, 50 * 25 * 4));
    }

    #[test]
    fn la_paleta_deja_pocos_colores() {
        let p = Proceso {
            paleta: Some(Paleta {
                colores: 8,
                tramado: 0.0,
            }),
            ..Default::default()
        };
        let (_, _, px) = aplicar(&p, 64, 64, &degradado(64, 64)).unwrap();
        let distintos: std::collections::HashSet<[u8; 4]> =
            px.as_chunks::<4>().0.iter().copied().collect();
        assert!(distintos.len() <= 8, "{}", distintos.len());
    }

    #[test]
    fn un_recorte_fuera_no_vale() {
        let p = Proceso {
            recorte: Some(Recorte {
                x: 500,
                y: 0,
                ancho: 10,
                alto: 10,
            }),
            ..Default::default()
        };
        assert!(aplicar(&p, 100, 100, &degradado(100, 100)).is_err());
    }

    fn con(medida: Medida, solo_reducir: bool) -> Redimension {
        Redimension {
            medida,
            solo_reducir,
            ..Default::default()
        }
    }

    // La misma tabla que frontend/src/estudio/tamano.test.ts: las dos cuentas
    // tienen que dar lo mismo.
    #[test]
    fn lado_mayor_y_porcentaje() {
        let lm = |l| con(Medida::LadoMayor(l), false);
        let pc = |p| con(Medida::Porcentaje(p), false);
        assert_eq!(destino(4000, 3000, lm(1920)), (1920, 1440));
        assert_eq!(destino(3000, 4000, lm(1920)), (1440, 1920));
        assert_eq!(destino(1000, 1000, lm(800)), (800, 800));
        assert_eq!(destino(4000, 3000, pc(50.0)), (2000, 1500));
        assert_eq!(destino(100, 75, pc(33.3)), (33, 25));
        assert_eq!(destino(3, 3, pc(10.0)), (1, 1));
        assert_eq!(destino(128, 128, pc(150.0)), (192, 192));
        // Sin agrandar: lo que ya cabe se queda.
        assert_eq!(
            destino(1200, 800, con(Medida::LadoMayor(1920), true)),
            (1200, 800)
        );
        assert_eq!(
            destino(128, 128, con(Medida::Porcentaje(150.0), true)),
            (128, 128)
        );
        assert_eq!(
            destino(4000, 3000, con(Medida::LadoMayor(1920), true)),
            (1920, 1440)
        );
    }

    #[test]
    fn el_json_de_antes_se_lee_igual() {
        let r: Redimension =
            serde_json::from_str(r#"{"ancho":800,"alto":null,"filtro":"mitchell","lineal":false}"#)
                .unwrap();
        assert_eq!(
            r.medida,
            Medida::Medidas {
                ancho: Some(800),
                alto: None
            }
        );
        assert_eq!(
            (r.filtro, r.lineal, r.solo_reducir),
            (Filtro::Mitchell, false, false)
        );
        let r: Redimension = serde_json::from_str("{}").unwrap();
        assert_eq!(r, Redimension::default());
        // Sin modo, se deduce.
        let r: Redimension = serde_json::from_str(r#"{"lado_mayor":1920}"#).unwrap();
        assert_eq!(r.medida, Medida::LadoMayor(1920));
        let r: Redimension = serde_json::from_str(r#"{"porcentaje":50}"#).unwrap();
        assert_eq!(r.medida, Medida::Porcentaje(50.0));
    }

    #[test]
    fn ida_y_vuelta_de_los_tres_modos() {
        for m in [
            Medida::Medidas {
                ancho: Some(10),
                alto: None,
            },
            Medida::LadoMayor(1920),
            Medida::Porcentaje(33.5),
        ] {
            let r = con(m, true);
            let texto = serde_json::to_string(&r).unwrap();
            assert!(texto.contains("\"modo\""), "{texto}");
            assert_eq!(serde_json::from_str::<Redimension>(&texto).unwrap(), r);
        }
    }

    #[test]
    fn un_tamano_que_no_vale_se_rechaza() {
        for mal in [
            r#"{"modo":"porcentaje","porcentaje":0}"#,
            r#"{"modo":"porcentaje"}"#,
            r#"{"modo":"lado_mayor","lado_mayor":0}"#,
            r#"{"porcentaje":-5}"#,
        ] {
            assert!(serde_json::from_str::<Redimension>(mal).is_err(), "{mal}");
        }
    }

    #[test]
    fn efectivo_quita_lo_que_no_cambia_nada() {
        let p = Proceso {
            recorte: Some(Recorte {
                x: 0,
                y: 0,
                ancho: 128,
                alto: 128,
            }),
            redimension: Some(con(Medida::LadoMayor(1920), true)),
            ..Default::default()
        };
        assert!(p.efectivo(128, 128).vacio());
        // Lo que sí cambia, se queda.
        let q = Proceso {
            recorte: Some(Recorte {
                x: 0,
                y: 16,
                ancho: 128,
                alto: 96,
            }),
            redimension: Some(con(Medida::Porcentaje(50.0), true)),
            ..Default::default()
        };
        assert_eq!(q.efectivo(128, 128), q);
        // La redimensión se mira sobre lo recortado: 64 de lado mayor en un
        // recorte de 64 × 32 no hace nada.
        let r = Proceso {
            recorte: Some(Recorte {
                x: 0,
                y: 0,
                ancho: 64,
                alto: 32,
            }),
            redimension: Some(con(Medida::LadoMayor(64), false)),
            ..Default::default()
        };
        assert_eq!(r.efectivo(128, 128).redimension, None);
    }
}
