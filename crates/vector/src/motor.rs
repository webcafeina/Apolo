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

/// Más colores distintos por píxel que esto (en la muestra de `paleta::colores_por_pixel`): foto. Las fotos del
/// banco, a su tamaño, dan de 0,25 a 0,62, y los dibujos (también con degradados, de 256 a 1.400 px) 0,031 como
/// mucho: el límite deja un margen de unas tres veces a cada lado (ADR 0039).
const LIMITE_FOTO: f64 = 0.08;

/// Y, además, más píxeles con textura que esto (`paleta::textura`): un logo con degradados tiene tantos colores
/// distintos como una foto, pero es liso. Las fotos del banco dan de 0,41 a 0,66; un logo real con degradados en
/// las letras (Aldea, del cliente), 0,056, y guardado en JPEG de calidad 85, 0,14; los dibujos, 0,033 como mucho.
const LIMITE_TEXTURA: f64 = 0.25;

/// El tipo efectivo, resolviendo `Auto`.
pub fn tipo_efectivo(im: &Rgba, tipo: TipoImagen) -> TipoImagen {
    match tipo {
        TipoImagen::Auto if es_pixel_art(im) => TipoImagen::PixelArt,
        TipoImagen::Auto
            if paleta::colores_por_pixel(im) > LIMITE_FOTO
                && paleta::textura(im) > LIMITE_TEXTURA =>
        {
            TipoImagen::Foto
        }
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
    // Una foto, antes, suavizada: sin el grano, cada mancha de color sale en menos formas y con menos nodos.
    let suave;
    let im = match radio_suavizado(im) {
        r if tipo == TipoImagen::Foto && r > 0 => {
            suave = mediana(im, r);
            &suave
        }
        _ => im,
    };
    if tipo == TipoImagen::PixelArt && o.paleta.is_empty() {
        let p = paleta_de(im, tipo, o)?;
        return Ok(pixelart::trazar(&paleta::aplicar(im, &p), &p));
    }
    // Lo transparente, si lo hay, es un color más (el último índice) que va siempre abajo y no se dibuja.
    let transparente = im.con_transparencia();
    let foto = tipo == TipoImagen::Foto;
    let duro = paleta::sin_suavizado(im);
    let pal = paleta_de(im, tipo, o)?;
    let mut mezcla = subpixel::mezclas(im, &pal, 2);
    let t_idx = pal.len();
    if transparente {
        subpixel::transparencia(im, &pal, &mut mezcla, t_idx as u16);
    }
    // Todos los «colores»: los de la paleta y, si lo hay, el transparente.
    let n = pal.len() + transparente as usize;
    // Con los colores limitados (un máximo o una paleta fija) no hay degradados: quien los limita quiere
    // colores planos (imprimir, vinilo), y un degradado se saltaría el límite (con dos colores, el azul de un
    // cuadrado y el rojo de un círculo acababan en un degradado de azul a rojo).
    let limitados = o.colores.is_some() || !o.paleta.is_empty();
    let rellenos: Vec<Option<Relleno>> = if o.degradados && !foto && !limitados {
        degradado::fusionar(im, &mut mezcla, n, transparente.then_some(t_idx))
    } else {
        (0..n).map(|_| None).collect()
    };
    // Los bordes mal explicados, con los colores de los trazos finos de alrededor (subpixel::afinar).
    subpixel::afinar(im, &pal, &mut mezcla, 2, &rellenos);
    let mut area_color = vec![0f64; n];
    for m in &mezcla {
        area_color[m.a as usize] += m.t as f64;
        area_color[m.b as usize] += 1.0 - m.t as f64;
    }
    // Lo transparente, siempre lo primero (abajo): ninguna capa puede destapar lo que hay debajo.
    let abajo = |k: usize| !(transparente && k == t_idx);
    let mut por_area: Vec<usize> = (0..n).filter(|&k| area_color[k] > 0.5).collect();
    por_area.sort_by(|a, b| {
        abajo(*a)
            .cmp(&abajo(*b))
            .then(area_color[*b].total_cmp(&area_color[*a]))
            .then(a.cmp(b))
    });
    // La otra opción: arriba los colores que forman islas cerradas dentro de otros (estrellas, ojos,
    // letras). Ninguna gana siempre: se pintan las dos y se queda la que más se parece al original.
    let islas = islas_por_color(&mezcla, im.ancho as usize, im.alto as usize, n);
    let mut por_islas = por_area.clone();
    por_islas.sort_by(|a, b| {
        abajo(*a)
            .cmp(&abajo(*b))
            .then(islas[*a].cmp(&islas[*b]))
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
            let doc = pintar(im, o, foto, duro, &pal, n, &mezcla, orden, &rellenos);
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
    n: usize,
    mezcla: &[subpixel::Mezcla],
    orden: &[usize],
    rellenos: &[Option<Relleno>],
) -> Documento {
    let (w, h) = (im.ancho, im.alto);
    let f = factor_detalle(o.detalle);
    let (tol_poli, tol_curva) = (if duro { 0.5 } else { 0.3 } * f, 0.15 * f);
    let minima = if foto { 6.0 } else { o.area_minima } * f.max(1.0);
    // El índice `pal.len()` es lo transparente: va siempre el primero y no tiene relleno (sin fondo).
    let relleno = |k: usize| rellenos[k].clone().unwrap_or(Relleno::Color(pal[k]));
    // Cada capa es independiente (su cobertura incluye las de encima): se calculan en paralelo y se
    // apilan en orden.
    let capas: Vec<Capa> = (1..orden.len())
        .into_par_iter()
        .filter_map(|r| {
            let k = orden[r];
            let mut dentro = vec![false; n];
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
        fondo: orden
            .first()
            .filter(|&&k| k < pal.len())
            .map(|&k| relleno(k)),
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

/// El radio con que se suaviza una foto antes de vectorizarla: nada por debajo de 500 px de lado corto, 1 px
/// hasta 1.000 y 2 px por encima (con la entrada reducida a 2 Mpx, no pasa de ahí). En las fotos del banco, a
/// su tamaño, el SVG pesa la mitad (de 13–21 MB a 6–13), sale de 2 a 4 veces antes y la nota es parecida (unas
/// suben, otras bajan); en una de 128 px, suavizar la hundía (de 26 a −6).
fn radio_suavizado(im: &Rgba) -> usize {
    (im.ancho.min(im.alto) / 500).min(2) as usize
}

/// Mediana por canal en una ventana de (2r+1)²: quita el grano y respeta los bordes.
fn mediana(im: &Rgba, r: usize) -> Rgba {
    let (w, h) = (im.ancho as usize, im.alto as usize);
    let mut out = im.px.clone();
    out.par_chunks_mut(w * 4).enumerate().for_each(|(y, fila)| {
        let mut v = Vec::with_capacity((2 * r + 1).pow(2));
        for x in 0..w {
            for c in 0..3 {
                v.clear();
                for j in y.saturating_sub(r)..=(y + r).min(h - 1) {
                    for i in x.saturating_sub(r)..=(x + r).min(w - 1) {
                        v.push(im.px[(j * w + i) * 4 + c]);
                    }
                }
                let m = v.len() / 2;
                fila[x * 4 + c] = *v.select_nth_unstable(m).1;
            }
        }
    });
    Rgba {
        ancho: im.ancho,
        alto: im.alto,
        px: out,
        alfa: im.alfa.clone(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn la_mediana_quita_el_grano_y_respeta_los_bordes() {
        // Mitad negra y mitad blanca, con un píxel rojo suelto en la negra.
        let (w, h) = (20u32, 10u32);
        let mut px = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let c = if x < 10 { 0 } else { 255 };
                if (x, y) == (4, 5) {
                    px.extend([255, 0, 0, 255]);
                } else {
                    px.extend([c, c, c, 255]);
                }
            }
        }
        let m = mediana(&Rgba::nueva(w, h, px), 2);
        assert_eq!(m.color(4, 5), [0, 0, 0], "el píxel suelto se va");
        for y in 0..h {
            assert_eq!(m.color(9, y), [0, 0, 0], "el borde se queda donde estaba");
            assert_eq!(m.color(10, y), [255, 255, 255]);
        }
    }

    #[test]
    fn radio_segun_el_tamano() {
        let im = |w, h| Rgba::nueva(w, h, vec![255; (w * h * 4) as usize]);
        assert_eq!(radio_suavizado(&im(128, 128)), 0);
        assert_eq!(radio_suavizado(&im(800, 600)), 1);
        assert_eq!(radio_suavizado(&im(1280, 1104)), 2);
        assert_eq!(radio_suavizado(&im(1414, 1414)), 2);
    }
}
