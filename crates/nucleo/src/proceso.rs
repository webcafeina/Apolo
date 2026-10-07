//! El proceso de la imagen antes de codificarla (ADR 0020): recortar,
//! redimensionar y reducir la paleta, en ese orden, como Squoosh. Trabaja
//! sobre RGBA de 8 bits y vale para todos los formatos de salida.
//!
//! Con proceso, la orden de la herramienta oficial ya no da el mismo fichero:
//! ni cwebp ni cjpeg redimensionan como Apolo. La interfaz lo dice.

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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Redimension {
    /// Si falta uno de los dos, se calcula para mantener la proporción.
    pub ancho: Option<u32>,
    pub alto: Option<u32>,
    pub filtro: Filtro,
    /// Mezclar en RGB lineal (más correcto; lo de Squoosh).
    pub lineal: bool,
}

impl Default for Redimension {
    fn default() -> Self {
        Redimension {
            ancho: None,
            alto: None,
            filtro: Filtro::Lanczos3,
            lineal: true,
        }
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
            (w, h) = destino(w, h, r.ancho, r.alto);
        }
        (w, h)
    }
}

fn destino(w: u32, h: u32, ancho: Option<u32>, alto: Option<u32>) -> (u32, u32) {
    let prop =
        |a: u32, de: u32, a_de: u32| ((a as f64 * a_de as f64 / de as f64).round() as u32).max(1);
    match (ancho, alto) {
        (Some(a), Some(b)) => (a.max(1), b.max(1)),
        (Some(a), None) => (a.max(1), prop(a, w, h)),
        (None, Some(b)) => (prop(b, h, w), b.max(1)),
        (None, None) => (w, h),
    }
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
            "el recorte queda fuera de la imagen".into(),
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
    let (nw, nh) = destino(w, h, r.ancho, r.alto);
    if (nw, nh) == (w, h) {
        return Ok((w, h, px.to_vec()));
    }
    let mal = |e: &dyn std::fmt::Display| Error::Codificacion(format!("al redimensionar: {e}"));
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
    let mal = |e: imagequant::Error| Error::Codificacion(format!("al reducir la paleta: {e}"));
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
    fn medidas_con_proporcion() {
        let p = Proceso {
            recorte: Some(Recorte {
                x: 10,
                y: 0,
                ancho: 200,
                alto: 100,
            }),
            redimension: Some(Redimension {
                ancho: Some(50),
                ..Default::default()
            }),
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
}
