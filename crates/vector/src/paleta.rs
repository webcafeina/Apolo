//! Paleta automática: cuántos colores tiene de verdad una imagen plana, sin contar los del suavizado.
//!
//! Hipótesis (variante «a» del plan): en un logotipo o una ilustración, los colores de verdad son los
//! de los píxeles «planos», rodeados de píxeles iguales; los de los bordes son mezclas del suavizado y
//! no cuentan. Se agrupan los colores planos por cercanía y cada píxel se lleva al más cercano.

use crate::imagen::Rgba;
use std::collections::HashMap;

fn dist2(a: [u8; 3], b: [u8; 3]) -> u32 {
    let d = |i: usize| (a[i] as i32 - b[i] as i32).pow(2) as u32;
    d(0) + d(1) + d(2)
}

/// Los colores planos agrupados, de más a menos frecuentes. Vacío si no hay casi nada plano (fotos).
pub fn detectar(im: &Rgba, maximo: usize) -> Vec<[u8; 3]> {
    let (w, h) = (im.ancho as i64, im.alto as i64);
    let mut cuenta: HashMap<[u8; 3], u32> = HashMap::new();
    let mut planos = 0u32;
    let mut opacos = 0u32;
    for y in 0..h {
        for x in 0..w {
            // Lo que no es opaco no cuenta: su color, aplanado sobre blanco, no es suyo.
            if im.alfa((y * w + x) as usize) < 255 {
                continue;
            }
            opacos += 1;
            let c = im.color(x as u32, y as u32);
            let mut plano = true;
            'v: for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let n = im.color(nx as u32, ny as u32);
                    if im.alfa((ny * w + nx) as usize) < 255
                        || (0..3).any(|i| (n[i] as i32 - c[i] as i32).abs() > 10)
                    {
                        plano = false;
                        break 'v;
                    }
                }
            }
            if plano {
                *cuenta.entry(c).or_default() += 1;
                planos += 1;
            }
        }
    }
    if planos < opacos / 5 {
        return Vec::new();
    }
    let mut colores: Vec<([u8; 3], u32)> = cuenta.into_iter().collect();
    colores.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut grupos: Vec<([u8; 3], u32)> = Vec::new();
    for (c, n) in colores {
        match grupos.iter_mut().find(|g| dist2(g.0, c) <= 28 * 28) {
            Some(g) => g.1 += n,
            None => grupos.push((c, n)),
        }
    }
    let minimo = (planos / 2000).max(6);
    grupos.retain(|g| g.1 >= minimo);
    grupos.truncate(maximo);
    grupos.into_iter().map(|g| g.0).collect()
}

/// Lleva cada píxel al color más cercano de la paleta.
pub fn aplicar(im: &Rgba, paleta: &[[u8; 3]]) -> Rgba {
    let mut px = im.px.clone();
    let mut cache: HashMap<[u8; 3], [u8; 3]> = HashMap::new();
    for p in px.as_chunks_mut::<4>().0 {
        let c = [p[0], p[1], p[2]];
        let r = *cache.entry(c).or_insert_with(|| {
            *paleta
                .iter()
                .min_by_key(|q| dist2(**q, c))
                .expect("paleta vacía")
        });
        p[..3].copy_from_slice(&r);
    }
    Rgba {
        ancho: im.ancho,
        alto: im.alto,
        px,
        alfa: im.alfa.clone(),
    }
}

/// Cuantiza con libimagequant (la de Apolo) a `n` colores, sin tramado. Para fotos.
pub fn cuantizar(im: &Rgba, n: u32) -> Result<(Rgba, Vec<[u8; 3]>), String> {
    let mut liq = imagequant::new();
    liq.set_max_colors(n).map_err(|e| e.to_string())?;
    let px: Vec<imagequant::RGBA> = im
        .px
        .as_chunks::<4>()
        .0
        .iter()
        .enumerate()
        .map(|(i, p)| imagequant::RGBA::new(p[0], p[1], p[2], im.alfa(i)))
        .collect();
    let mut img = liq
        .new_image(px, im.ancho as usize, im.alto as usize, 0.0)
        .map_err(|e| e.to_string())?;
    let mut res = liq.quantize(&mut img).map_err(|e| e.to_string())?;
    res.set_dithering_level(0.0).map_err(|e| e.to_string())?;
    let (pal, idx) = res.remapped(&mut img).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(im.px.len());
    for i in idx {
        let c = pal[i as usize];
        out.extend_from_slice(&[c.r, c.g, c.b, 255]);
    }
    Ok((
        Rgba {
            ancho: im.ancho,
            alto: im.alto,
            px: out,
            alfa: im.alfa.clone(),
        },
        // Los colores transparentes de la paleta, fuera: lo transparente va aparte.
        pal.iter()
            .filter(|c| c.a >= 128)
            .map(|c| [c.r, c.g, c.b])
            .collect(),
    ))
}

/// Todos los colores distintos (pixel art).
pub fn exactos(im: &Rgba) -> Vec<[u8; 3]> {
    let mut cuenta: HashMap<[u8; 3], u32> = HashMap::new();
    for (i, p) in im.px.as_chunks::<4>().0.iter().enumerate() {
        if im.alfa(i) >= 128 {
            *cuenta.entry([p[0], p[1], p[2]]).or_default() += 1;
        }
    }
    let mut v: Vec<_> = cuenta.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.into_iter().map(|x| x.0).collect()
}

/// ¿Sin suavizado? Casi ningún píxel (menos del 0,3 %) tiene un color intermedio, lejos de todos los de
/// su paleta. Contar colores no sirve: un dibujo en blanco y negro suavizado tiene pocos grises distintos.
pub fn sin_suavizado(im: &Rgba) -> bool {
    let p = detectar(im, 64);
    if p.is_empty() {
        return false;
    }
    // Solo los opacos: el borde de una figura sobre transparente tiene alfa intermedio, no color.
    let (mut opacos, mut intermedios) = (0usize, 0usize);
    for (i, q) in im.px.as_chunks::<4>().0.iter().enumerate() {
        if im.alfa(i) < 255 {
            continue;
        }
        opacos += 1;
        let c = [q[0], q[1], q[2]];
        if p.iter().all(|k| dist2(*k, c) > 20 * 20) {
            intermedios += 1;
        }
    }
    (intermedios as f64) < 0.003 * opacos.max(1) as f64
}

/// La parte de los píxeles con textura: los que se apartan de su vecino de la derecha o de abajo más de lo que
/// cambia un degradado (6 en algún canal) y menos que un borde (40). En una foto hay textura en casi todas partes
/// (grano, piel, hojas); en un dibujo, aunque tenga degradados, casi todo es liso o borde. Se mira la misma
/// muestra que `colores_por_pixel`, y solo donde los tres píxeles son opacos.
pub fn textura(im: &Rgba) -> f64 {
    let (w, h) = (im.ancho as usize, im.alto as usize);
    if w < 2 || h < 2 {
        return 0.0;
    }
    let paso = ((w * h) as f64 / MUESTRA_COLORES as f64)
        .sqrt()
        .ceil()
        .max(1.0) as usize;
    let (mut con, mut total) = (0usize, 0usize);
    for y in (0..h - 1).step_by(paso) {
        for x in (0..w - 1).step_by(paso) {
            let i = y * w + x;
            if im.alfa(i) < 255 || im.alfa(i + 1) < 255 || im.alfa(i + w) < 255 {
                continue;
            }
            let c = |j: usize| &im.px[j * 4..j * 4 + 3];
            let (a, b, d) = (c(i), c(i + 1), c(i + w));
            let m = (0..3)
                .map(|k| a[k].abs_diff(b[k]).max(a[k].abs_diff(d[k])))
                .max()
                .unwrap_or(0);
            total += 1;
            if (7..=40).contains(&m) {
                con += 1;
            }
        }
    }
    con as f64 / total.max(1) as f64
}

/// Cuántos píxeles mira `colores_por_pixel`, como mucho.
const MUESTRA_COLORES: usize = 65_536;

/// Colores distintos por píxel: en una foto, cada píxel casi tiene el suyo. Se mira una muestra repartida por
/// la imagen de unos 65.000 píxeles, para que no dependa del tamaño: en una foto grande los colores se repiten
/// más, y los arándanos del banco a 1.280 px daban 0,047 y pasaban por dibujo (75 s y 10 MB).
pub fn colores_por_pixel(im: &Rgba) -> f64 {
    let (w, h) = (im.ancho as usize, im.alto as usize);
    let paso = ((w * h) as f64 / MUESTRA_COLORES as f64)
        .sqrt()
        .ceil()
        .max(1.0) as usize;
    let mut distintos = std::collections::HashSet::new();
    let mut opacos = 0usize;
    for y in (0..h).step_by(paso) {
        for x in (0..w).step_by(paso) {
            let i = y * w + x;
            if im.alfa(i) == 255 {
                let q = &im.px[i * 4..i * 4 + 3];
                distintos.insert([q[0], q[1], q[2]]);
                opacos += 1;
            }
        }
    }
    distintos.len() as f64 / opacos.max(1) as f64
}
