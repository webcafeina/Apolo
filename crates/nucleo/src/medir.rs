//! Medir la pérdida (ADR 0022): cuánto se aleja el resultado de la imagen que
//! recibió el codificador.
//!
//! - **SSIMULACRA 2**, la nota principal: la de libjxl, idéntica a la de su
//!   herramienta `ssimulacra2`. De −∞ a 100; 90 o más no se distingue del
//!   original, 70 es «alta calidad». Es cara: unos 4 s y 1,75 GB con una foto
//!   de 12 MP en el VPS, así que se mide **una a la vez** en todo Apolo.
//! - **PSNR** (dB) y **SSIM** (de 0 a 1), las cifras técnicas de siempre.
//!
//! Con transparencia, las tres se miden sobre un fondo oscuro y uno claro, y
//! vale la peor, como hace `ssimulacra2`: un píxel transparente no se ve, pero
//! lo que haya detrás sí.
//!
//! El PSNR es el de los tres canales juntos (el error cuadrático medio de R, G
//! y B). El SSIM es el de Wang y otros (2004) sobre la luminancia BT.601, con
//! ventana gaussiana de 11 píxeles y σ = 1,5, como el SSIM «de referencia».

use std::sync::Mutex;

use serde::Serialize;

use crate::{Error, Resultado};

/// Las medidas de un resultado frente a su referencia.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Medidas {
    /// `None` si la imagen es menor de 8×8, donde SSIMULACRA 2 no mide.
    pub ssimulacra2: Option<f64>,
    /// `None` si son idénticas (infinito).
    pub psnr: Option<f64>,
    pub ssim: f64,
}

/// SSIMULACRA 2 usa mucha memoria: una medida a la vez en todo el proceso.
static UNA_A_LA_VEZ: Mutex<()> = Mutex::new(());

/// Fondos sobre los que se componen las imágenes con transparencia: los de
/// `ssimulacra2` (0,1 y 0,9).
const FONDOS: [f32; 2] = [0.1 * 255.0, 0.9 * 255.0];

fn comprobar(a: &[u8], b: &[u8], ancho: u32, alto: u32) -> Resultado<()> {
    let n = ancho as usize * alto as usize * 4;
    if a.len() != n || b.len() != n {
        return Err(Error::Configuracion(
            "el resultado no tiene el tamaño de la referencia".into(),
        ));
    }
    Ok(())
}

/// Si alguna de las dos tiene algún píxel no del todo opaco.
fn con_alfa(a: &[u8], b: &[u8]) -> bool {
    let transparente = |v: &[u8]| v.as_chunks::<4>().0.iter().any(|p| p[3] != 255);
    transparente(a) || transparente(b)
}

/// RGB de un RGBA compuesto sobre un fondo gris de intensidad `fondo`.
fn sobre(rgba: &[u8], fondo: f32) -> Vec<f32> {
    let mut v = Vec::with_capacity(rgba.len() / 4 * 3);
    for p in rgba.as_chunks::<4>().0 {
        let a = p[3] as f32 / 255.0;
        for &c in &p[..3] {
            v.push(c as f32 * a + fondo * (1.0 - a));
        }
    }
    v
}

fn sin_alfa(rgba: &[u8]) -> Vec<f32> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[0] as f32, p[1] as f32, p[2] as f32])
        .collect()
}

/// Las tres medidas de `resultado` frente a `referencia`, RGBA de 8 bits del
/// mismo tamaño.
pub fn medir(referencia: &[u8], resultado: &[u8], ancho: u32, alto: u32) -> Resultado<Medidas> {
    comprobar(referencia, resultado, ancho, alto)?;
    let alfa = con_alfa(referencia, resultado);
    let pares: Vec<(Vec<f32>, Vec<f32>)> = if alfa {
        FONDOS
            .iter()
            .map(|&f| (sobre(referencia, f), sobre(resultado, f)))
            .collect()
    } else {
        vec![(sin_alfa(referencia), sin_alfa(resultado))]
    };
    let psnr = pares
        .iter()
        .map(|(a, b)| psnr(a, b))
        .fold(f64::INFINITY, f64::min);
    let ssim = pares
        .iter()
        .map(|(a, b)| ssim_medio(a, b, ancho as usize, alto as usize))
        .fold(f64::INFINITY, f64::min);
    let ssimulacra2 = nota_sin_comprobar(referencia, resultado, ancho, alto, alfa);
    Ok(Medidas {
        ssimulacra2,
        psnr: psnr.is_finite().then_some(psnr),
        ssim,
    })
}

/// Solo la nota SSIMULACRA 2 (para buscar una calidad, donde PSNR y SSIM
/// sobran). `None` si la imagen es menor de 8×8.
pub fn nota(referencia: &[u8], resultado: &[u8], ancho: u32, alto: u32) -> Resultado<Option<f64>> {
    comprobar(referencia, resultado, ancho, alto)?;
    let alfa = con_alfa(referencia, resultado);
    Ok(nota_sin_comprobar(referencia, resultado, ancho, alto, alfa))
}

fn nota_sin_comprobar(
    referencia: &[u8],
    resultado: &[u8],
    ancho: u32,
    alto: u32,
    alfa: bool,
) -> Option<f64> {
    let _turno = UNA_A_LA_VEZ.lock().unwrap_or_else(|e| e.into_inner());
    if alfa {
        apolo_avifjxl::ssimulacra2(referencia, resultado, ancho, alto, 4)
    } else {
        let rgb = |v: &[u8]| -> Vec<u8> {
            v.as_chunks::<4>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect()
        };
        apolo_avifjxl::ssimulacra2(&rgb(referencia), &rgb(resultado), ancho, alto, 3)
    }
}

/// PSNR de dos imágenes RGB en coma flotante (0–255).
fn psnr(a: &[f32], b: &[f32]) -> f64 {
    let suma: f64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| {
            let d = (*x - *y) as f64;
            d * d
        })
        .sum();
    let mse = suma / a.len().max(1) as f64;
    if mse == 0.0 {
        f64::INFINITY
    } else {
        10.0 * (255.0f64 * 255.0 / mse).log10()
    }
}

/// Luminancia BT.601 de RGB en coma flotante.
fn luma(rgb: &[f32]) -> Vec<f32> {
    rgb.as_chunks::<3>()
        .0
        .iter()
        .map(|p| 0.299 * p[0] + 0.587 * p[1] + 0.114 * p[2])
        .collect()
}

/// La ventana gaussiana de 11 con σ = 1,5, normalizada.
fn ventana() -> [f32; 11] {
    let mut k = [0f32; 11];
    let mut suma = 0.0;
    for (i, v) in k.iter_mut().enumerate() {
        let x = i as f32 - 5.0;
        *v = (-(x * x) / (2.0 * 1.5 * 1.5)).exp();
        suma += *v;
    }
    k.map(|v| v / suma)
}

/// Desenfoque gaussiano separable, con los bordes repetidos.
fn desenfocar(v: &[f32], w: usize, h: usize) -> Vec<f32> {
    let k = ventana();
    let mut horizontal = vec![0f32; v.len()];
    for y in 0..h {
        let fila = &v[y * w..(y + 1) * w];
        for x in 0..w {
            let mut s = 0.0;
            for (i, kv) in k.iter().enumerate() {
                let xx = (x as isize + i as isize - 5).clamp(0, w as isize - 1) as usize;
                s += kv * fila[xx];
            }
            horizontal[y * w + x] = s;
        }
    }
    let mut salida = vec![0f32; v.len()];
    for y in 0..h {
        for x in 0..w {
            let mut s = 0.0;
            for (i, kv) in k.iter().enumerate() {
                let yy = (y as isize + i as isize - 5).clamp(0, h as isize - 1) as usize;
                s += kv * horizontal[yy * w + x];
            }
            salida[y * w + x] = s;
        }
    }
    salida
}

/// El mapa SSIM, un valor por píxel (1 es idéntico).
fn mapa_ssim(a: &[f32], b: &[f32], w: usize, h: usize) -> Vec<f32> {
    let (ya, yb) = (luma(a), luma(b));
    let producto = |p: &[f32], q: &[f32]| p.iter().zip(q).map(|(x, y)| x * y).collect::<Vec<f32>>();
    let mu_a = desenfocar(&ya, w, h);
    let mu_b = desenfocar(&yb, w, h);
    let aa = desenfocar(&producto(&ya, &ya), w, h);
    let bb = desenfocar(&producto(&yb, &yb), w, h);
    let ab = desenfocar(&producto(&ya, &yb), w, h);
    let c1 = (0.01f32 * 255.0).powi(2);
    let c2 = (0.03f32 * 255.0).powi(2);
    (0..ya.len())
        .map(|i| {
            let (ma, mb) = (mu_a[i], mu_b[i]);
            let va = aa[i] - ma * ma;
            let vb = bb[i] - mb * mb;
            let cov = ab[i] - ma * mb;
            ((2.0 * ma * mb + c1) * (2.0 * cov + c2)) / ((ma * ma + mb * mb + c1) * (va + vb + c2))
        })
        .collect()
}

fn ssim_medio(a: &[f32], b: &[f32], w: usize, h: usize) -> f64 {
    let m = mapa_ssim(a, b, w, h);
    m.iter().map(|&v| v as f64).sum::<f64>() / m.len().max(1) as f64
}

/// Qué enseña el mapa de diferencias.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoMapa {
    /// Cuánto cambia cada píxel: la mayor diferencia de sus canales (alfa
    /// incluido), multiplicada por 4 para que se vea.
    Diferencia,
    /// Dónde se pierde estructura: 1 − SSIM local, sobre la luminancia, por
    /// 2 (con 4 saturaba: una nota de 80 parecía mala por todas partes).
    Estructura,
}

/// El mapa de calor de dónde se aleja `resultado` de `referencia`, en RGBA
/// para pintar: la referencia apagada en gris debajo, y encima el calor, de
/// rojo oscuro a amarillo y blanco donde la diferencia es mayor.
pub fn mapa(
    referencia: &[u8],
    resultado: &[u8],
    ancho: u32,
    alto: u32,
    tipo: TipoMapa,
) -> Resultado<Vec<u8>> {
    comprobar(referencia, resultado, ancho, alto)?;
    let (w, h) = (ancho as usize, alto as usize);
    let intensidad: Vec<f32> = match tipo {
        TipoMapa::Diferencia => referencia
            .as_chunks::<4>()
            .0
            .iter()
            .zip(resultado.as_chunks::<4>().0)
            .map(|(p, q)| {
                let d = (0..4).map(|i| p[i].abs_diff(q[i])).max().unwrap_or(0);
                (d as f32 * 4.0 / 255.0).min(1.0)
            })
            .collect(),
        TipoMapa::Estructura => {
            // Con transparencia, sobre el fondo claro: es donde más se ve.
            let (a, b) = if con_alfa(referencia, resultado) {
                (sobre(referencia, FONDOS[1]), sobre(resultado, FONDOS[1]))
            } else {
                (sin_alfa(referencia), sin_alfa(resultado))
            };
            mapa_ssim(&a, &b, w, h)
                .into_iter()
                .map(|s| ((1.0 - s) * 2.0).clamp(0.0, 1.0))
                .collect()
        }
    };
    let mut salida = Vec::with_capacity(w * h * 4);
    for (p, &t) in referencia.as_chunks::<4>().0.iter().zip(&intensidad) {
        let a = p[3] as f32 / 255.0;
        let gris = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) * a;
        let base = gris * 0.3 + 20.0;
        let calor = [
            (3.0 * t).min(1.0) * 255.0,
            (3.0 * t - 1.0).clamp(0.0, 1.0) * 255.0,
            (3.0 * t - 2.0).clamp(0.0, 1.0) * 255.0,
        ];
        let peso = (t * 2.0).min(1.0);
        for c in calor {
            salida.push((base * (1.0 - peso) + c * peso).round() as u8);
        }
        salida.push(255);
    }
    Ok(salida)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn degradado(w: u32, h: u32, alfa: u8) -> Vec<u8> {
        (0..w * h)
            .flat_map(|i| {
                let (x, y) = (i % w, i / w);
                [(x * 7) as u8, (y * 5) as u8, ((x + y) * 3) as u8, alfa]
            })
            .collect()
    }

    #[test]
    fn identicas() {
        let a = degradado(32, 24, 255);
        let m = medir(&a, &a, 32, 24).unwrap();
        assert_eq!(m.psnr, None);
        assert!((m.ssim - 1.0).abs() < 1e-6);
        assert!((m.ssimulacra2.unwrap() - 100.0).abs() < 1e-6);
    }

    #[test]
    fn peor_cuanto_mas_ruido() {
        let a = degradado(48, 40, 255);
        let ruido = |n: u8| -> Vec<u8> {
            a.iter()
                .enumerate()
                .map(|(i, &v)| {
                    if i % 4 == 3 {
                        v
                    } else {
                        v.wrapping_add(((i * 7919) % (n as usize + 1)) as u8)
                    }
                })
                .collect()
        };
        let poco = medir(&a, &ruido(4), 48, 40).unwrap();
        let mucho = medir(&a, &ruido(40), 48, 40).unwrap();
        assert!(poco.psnr.unwrap() > mucho.psnr.unwrap());
        assert!(poco.ssim > mucho.ssim);
        assert!(poco.ssimulacra2.unwrap() > mucho.ssimulacra2.unwrap());
    }

    #[test]
    fn la_transparencia_cuenta() {
        // Mismo color, distinto alfa: se nota sobre los fondos.
        let a = degradado(16, 16, 255);
        let b = degradado(16, 16, 128);
        let m = medir(&a, &b, 16, 16).unwrap();
        assert!(m.psnr.is_some() && m.ssim < 1.0);
    }

    #[test]
    fn el_mapa_tiene_el_tamano_y_marca_la_diferencia() {
        let a = degradado(20, 10, 255);
        let mut b = a.clone();
        b[0] = b[0].wrapping_add(200);
        for tipo in [TipoMapa::Diferencia, TipoMapa::Estructura] {
            let m = mapa(&a, &b, 20, 10, tipo).unwrap();
            assert_eq!(m.len(), 20 * 10 * 4);
            // El píxel cambiado es más rojo que uno lejano sin cambios.
            assert!(m[0] > m[(9 * 20 + 19) * 4], "{tipo:?}");
        }
    }

    #[test]
    fn rechaza_tamanos_distintos() {
        let a = degradado(8, 8, 255);
        assert!(medir(&a, &a[..a.len() - 4], 8, 8).is_err());
    }
}
