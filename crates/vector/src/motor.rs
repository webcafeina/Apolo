//! El motor: de la imagen al [`Documento`].

use crate::geometria::{P, area};
use crate::imagen::Rgba;
use crate::modelo::{Capa, Documento, Forma, Relleno};
use crate::{degradado, paleta, pixelart, poli, subpixel};
use rayon::prelude::*;

/// Qué tipo de imagen es. `Auto` lo detecta: pixel art si tiene pocos colores sin suavizar y en bloques;
/// foto si casi no tiene zonas de color plano; si no, ilustración.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TipoImagen {
    #[default]
    Auto,
    Logotipo,
    Ilustracion,
    /// Blanco y negro: dos colores.
    ByN,
    PixelArt,
    Foto,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OpcionesVector {
    pub tipo: TipoImagen,
    /// Colores como mucho (`None`: los que tenga la imagen).
    pub colores: Option<u32>,
    /// Paleta fija: cada píxel se lleva al color más cercano de esta lista.
    pub paleta: Vec<[u8; 3]>,
    /// Detalle, de 0 a 100: cuánto se ciñe al original. 75 es el equilibrio medido en el banco; más,
    /// más fiel y más nodos; menos, más ligero. El modo por nota busca el menor que llega a la nota.
    pub detalle: u8,
    /// Las formas de menos área (en píxeles cuadrados) se descartan.
    pub area_minima: f64,
    /// Degradados donde el color cambia suavemente. Con `colores` o `paleta`, no: colores planos.
    pub degradados: bool,
}

impl Default for OpcionesVector {
    fn default() -> Self {
        Self {
            tipo: TipoImagen::Auto,
            colores: None,
            paleta: Vec::new(),
            detalle: 75,
            area_minima: 0.8,
            degradados: true,
        }
    }
}

/// Pixel art: pocos colores exactos y, o es pequeño, o todos los tramos de color miden un múltiplo de un
/// bloque de 2 o más píxeles (pixel art ampliado).
pub(crate) fn es_pixel_art(im: &Rgba) -> bool {
    if paleta::exactos(im).len() > 64 {
        return false;
    }
    if im.ancho.max(im.alto) <= 64 {
        return true;
    }
    if !paleta::sin_suavizado(im) {
        return false;
    }
    fn mcd(a: u32, b: u32) -> u32 {
        if b == 0 { a } else { mcd(b, a % b) }
    }
    let mut g = 0;
    for y in 0..im.alto {
        let mut run = 1;
        for x in 1..im.ancho {
            if im.color(x, y) == im.color(x - 1, y) {
                run += 1
            } else {
                g = mcd(g, run);
                run = 1;
                if g == 1 {
                    return false;
                }
            }
        }
    }
    for x in 0..im.ancho {
        let mut run = 1;
        for y in 1..im.alto {
            if im.color(x, y) == im.color(x, y - 1) {
                run += 1
            } else {
                g = mcd(g, run);
                run = 1;
                if g == 1 {
                    return false;
                }
            }
        }
    }
    g >= 2
}

/// Más colores distintos por píxel que esto: foto. En el banco, las fotos dan de 0,13 a 0,25 y los
/// dibujos (también los suavizados y con degradados) 0,007 como mucho (ADR 0039).
const LIMITE_FOTO: f64 = 0.05;

/// El tipo efectivo, resolviendo `Auto`.
pub fn tipo_efectivo(im: &Rgba, tipo: TipoImagen) -> TipoImagen {
    match tipo {
        TipoImagen::Auto if es_pixel_art(im) => TipoImagen::PixelArt,
        TipoImagen::Auto if paleta::colores_por_pixel(im) > LIMITE_FOTO => TipoImagen::Foto,
        TipoImagen::Auto => TipoImagen::Ilustracion,
        t => t,
    }
}

fn paleta_de(im: &Rgba, tipo: TipoImagen, o: &OpcionesVector) -> Result<Vec<[u8; 3]>, String> {
    if !o.paleta.is_empty() {
        return Ok(o.paleta.clone());
    }
    let maximo = o.colores.map(|c| c.clamp(2, 256) as usize);
    Ok(match tipo {
        TipoImagen::Foto => paleta::cuantizar(im, maximo.unwrap_or(24) as u32)?.1,
        TipoImagen::PixelArt => {
            let p = paleta::exactos(im);
            if p.len() <= maximo.unwrap_or(64) {
                p
            } else {
                paleta::cuantizar(im, maximo.unwrap_or(64) as u32)?.1
            }
        }
        TipoImagen::ByN => {
            let p = paleta::detectar(im, 2);
            if p.len() == 2 {
                p
            } else {
                paleta::cuantizar(im, 2)?.1
            }
        }
        _ => {
            let p = paleta::detectar(im, maximo.unwrap_or(48));
            if p.is_empty() {
                paleta::cuantizar(im, maximo.unwrap_or(24) as u32)?.1
            } else {
                p
            }
        }
    })
}

/// Vectoriza una imagen.
pub fn vectorizar(im: &Rgba, o: &OpcionesVector) -> Result<Documento, String> {
    let tipo = tipo_efectivo(im, o.tipo);
    if tipo == TipoImagen::PixelArt && o.paleta.is_empty() {
        let p = paleta_de(im, tipo, o)?;
        return Ok(pixelart::trazar(&paleta::aplicar(im, &p), &p));
    }
    let foto = tipo == TipoImagen::Foto;
    let duro = paleta::sin_suavizado(im);
    let pal = paleta_de(im, tipo, o)?;
    let mut mezcla = subpixel::mezclas(im, &pal, 2);
    // Con los colores limitados (un máximo o una paleta fija) no hay degradados: quien los limita quiere
    // colores planos (imprimir, vinilo), y un degradado se saltaría el límite (con dos colores, el azul de un
    // cuadrado y el rojo de un círculo acababan en un degradado de azul a rojo).
    let limitados = o.colores.is_some() || !o.paleta.is_empty();
    let rellenos: Vec<Option<Relleno>> = if o.degradados && !foto && !limitados {
        degradado::fusionar(im, &mut mezcla, pal.len())
    } else {
        (0..pal.len()).map(|_| None).collect()
    };
    let mut area_color = vec![0f64; pal.len()];
    for m in &mezcla {
        area_color[m.a as usize] += m.t as f64;
        area_color[m.b as usize] += 1.0 - m.t as f64;
    }
    let mut por_area: Vec<usize> = (0..pal.len()).filter(|&k| area_color[k] > 0.5).collect();
    por_area.sort_by(|a, b| area_color[*b].total_cmp(&area_color[*a]).then(a.cmp(b)));
    // La otra opción: arriba los colores que forman islas cerradas dentro de otros (estrellas, ojos,
    // letras). Ninguna gana siempre: se pintan las dos y se queda la que más se parece al original.
    let islas = islas_por_color(&mezcla, im.ancho as usize, im.alto as usize, pal.len());
    let mut por_islas = por_area.clone();
    por_islas.sort_by(|a, b| {
        islas[*a]
            .cmp(&islas[*b])
            .then(area_color[*b].total_cmp(&area_color[*a]))
            .then(a.cmp(b))
    });
    let candidatos = if foto || por_islas == por_area {
        vec![por_area]
    } else {
        vec![por_area, por_islas]
    };
    let varios = candidatos.len() > 1;
    let mejor = candidatos
        .par_iter()
        .map(|orden| {
            let doc = pintar(im, o, foto, duro, &pal, &mezcla, orden, &rellenos);
            let e = if varios {
                error_cuadratico(im, &doc)
            } else {
                0.0
            };
            (e, doc)
        })
        .collect::<Vec<_>>()
        .into_iter()
        // El primero gana los empates, como antes.
        .reduce(|a, b| if b.0 < a.0 { b } else { a });
    mejor
        .map(|m| m.1)
        .ok_or_else(|| "La imagen no tiene colores que vectorizar.".to_string())
}

/// Las tolerancias según el detalle: 75 son las del banco; cada 25 por encima, la mitad; por debajo, el doble.
fn factor_detalle(detalle: u8) -> f64 {
    2f64.powf((75.0 - detalle.min(100) as f64) / 25.0)
}

#[allow(clippy::too_many_arguments)]
fn pintar(
    im: &Rgba,
    o: &OpcionesVector,
    foto: bool,
    duro: bool,
    pal: &[[u8; 3]],
    mezcla: &[subpixel::Mezcla],
    orden: &[usize],
    rellenos: &[Option<Relleno>],
) -> Documento {
    let (w, h) = (im.ancho, im.alto);
    let f = factor_detalle(o.detalle);
    let (tol_poli, tol_curva) = (if duro { 0.5 } else { 0.3 } * f, 0.15 * f);
    let minima = if foto { 6.0 } else { o.area_minima } * f.max(1.0);
    let relleno = |k: usize| rellenos[k].clone().unwrap_or(Relleno::Color(pal[k]));
    // Cada capa es independiente (su cobertura incluye las de encima): se calculan en paralelo y se
    // apilan en orden.
    let capas: Vec<Capa> = (1..orden.len())
        .into_par_iter()
        .filter_map(|r| {
            let k = orden[r];
            let mut dentro = vec![false; pal.len()];
            for &q in &orden[r..] {
                dentro[q] = true;
            }
            let cob = subpixel::cobertura(mezcla, &dentro);
            let muestra = |x: f64, y: f64| -> f32 {
                // Bilineal sobre los centros de los píxeles; fuera de la imagen, 0.
                let (fx, fy) = (x - 0.5, y - 0.5);
                let (x0, y0) = (fx.floor(), fy.floor());
                let (tx, ty) = ((fx - x0) as f32, (fy - y0) as f32);
                let v = |i: f64, j: f64| -> f32 {
                    if i < 0.0 || j < 0.0 || i >= w as f64 || j >= h as f64 {
                        0.0
                    } else {
                        cob[j as usize * w as usize + i as usize]
                    }
                };
                let a = v(x0, y0) * (1.0 - tx) + v(x0 + 1.0, y0) * tx;
                let b = v(x0, y0 + 1.0) * (1.0 - tx) + v(x0 + 1.0, y0 + 1.0) * tx;
                a * (1.0 - ty) + b * ty
            };
            let formas: Vec<Forma> = subpixel::contornos(&cob, w as usize, h as usize)
                .into_par_iter()
                .filter(|lazo: &Vec<P>| area(lazo).abs() >= minima)
                .filter_map(|lazo| {
                    poli::ajustar(&lazo, tol_poli, tol_curva, &muestra, (w, h), !foto)
                })
                .collect();
            (!formas.is_empty()).then(|| Capa {
                relleno: relleno(k),
                formas,
            })
        })
        .collect();
    Documento {
        ancho: w,
        alto: h,
        fondo: orden.first().map(|&k| relleno(k)),
        capas,
    }
}

/// Error cuadrático medio entre el original y el documento rasterizado.
fn error_cuadratico(im: &Rgba, doc: &Documento) -> f64 {
    match crate::vista::rasterizar(doc) {
        Ok(r) => {
            im.px
                .iter()
                .zip(&r)
                .map(|(a, b)| (*a as f64 - *b as f64).powi(2))
                .sum::<f64>()
                / im.px.len() as f64
        }
        Err(_) => f64::MAX,
    }
}

/// Cuántas islas tiene cada color: componentes conexas (de 4 px o más) que no tocan el borde de la imagen.
fn islas_por_color(m: &[subpixel::Mezcla], w: usize, h: usize, n: usize) -> Vec<usize> {
    let etiqueta: Vec<u16> = m
        .iter()
        .map(|x| if x.t >= 0.5 { x.a } else { x.b })
        .collect();
    let mut visto = vec![false; w * h];
    let mut islas = vec![0usize; n];
    let mut pila = Vec::new();
    for ini in 0..w * h {
        if visto[ini] {
            continue;
        }
        let c = etiqueta[ini];
        let (mut tam, mut borde) = (0usize, false);
        visto[ini] = true;
        pila.push(ini);
        while let Some(i) = pila.pop() {
            tam += 1;
            let (x, y) = (i % w, i / w);
            if x == 0 || y == 0 || x == w - 1 || y == h - 1 {
                borde = true;
            }
            let mut mira = |j: usize| {
                if !visto[j] && etiqueta[j] == c {
                    visto[j] = true;
                    pila.push(j);
                }
            };
            if x > 0 {
                mira(i - 1);
            }
            if x + 1 < w {
                mira(i + 1);
            }
            if y > 0 {
                mira(i - w);
            }
            if y + 1 < h {
                mira(i + w);
            }
        }
        if !borde && tam >= 4 {
            islas[c as usize] += 1;
        }
    }
    islas
}
