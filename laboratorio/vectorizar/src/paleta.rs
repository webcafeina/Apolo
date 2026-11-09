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
    for y in 0..h {
        for x in 0..w {
            let c = im.color(x as u32, y as u32);
            let mut plano = true;
            'v: for dy in -1..=1 {
                for dx in -1..=1 {
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let n = im.color(nx as u32, ny as u32);
                    if (0..3).any(|i| (n[i] as i32 - c[i] as i32).abs() > 10) {
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
    let total = (w * h) as u32;
    if planos < total / 5 {
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
    for p in px.chunks_exact_mut(4) {
        let c = [p[0], p[1], p[2]];
        let r = *cache.entry(c).or_insert_with(|| *paleta.iter().min_by_key(|q| dist2(**q, c)).expect("paleta vacía"));
        p[..3].copy_from_slice(&r);
    }
    Rgba { ancho: im.ancho, alto: im.alto, px }
}

/// Cuantiza con libimagequant (la de Apolo) a `n` colores, sin tramado. Para fotos.
pub fn cuantizar(im: &Rgba, n: u32) -> Result<(Rgba, Vec<[u8; 3]>), String> {
    let mut liq = imagequant::new();
    liq.set_max_colors(n).map_err(|e| e.to_string())?;
    let px: Vec<imagequant::RGBA> = im.px.chunks_exact(4).map(|p| imagequant::RGBA::new(p[0], p[1], p[2], p[3])).collect();
    let mut img = liq.new_image(px, im.ancho as usize, im.alto as usize, 0.0).map_err(|e| e.to_string())?;
    let mut res = liq.quantize(&mut img).map_err(|e| e.to_string())?;
    res.set_dithering_level(0.0).map_err(|e| e.to_string())?;
    let (pal, idx) = res.remapped(&mut img).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(im.px.len());
    for i in idx {
        let c = pal[i as usize];
        out.extend_from_slice(&[c.r, c.g, c.b, 255]);
    }
    Ok((Rgba { ancho: im.ancho, alto: im.alto, px: out }, pal.iter().map(|c| [c.r, c.g, c.b]).collect()))
}

/// Todos los colores distintos (pixel art).
pub fn exactos(im: &Rgba) -> Vec<[u8; 3]> {
    let mut cuenta: HashMap<[u8; 3], u32> = HashMap::new();
    for p in im.px.chunks_exact(4) {
        *cuenta.entry([p[0], p[1], p[2]]).or_default() += 1;
    }
    let mut v: Vec<_> = cuenta.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v.into_iter().map(|x| x.0).collect()
}

/// Como `aplicar`, pero los píxeles de borde (los que no son planos) solo pueden ir a un color que
/// tengan cerca, a menos de `radio` píxeles, entre los píxeles planos. Así un borde azul-blanco no acaba
/// en rojo por ser el color global más parecido a la mezcla (variante «e»: sin halos).
pub fn aplicar_local(im: &Rgba, paleta: &[[u8; 3]], radio: i64) -> Rgba {
    let (w, h) = (im.ancho as i64, im.alto as i64);
    let cercano = |c: [u8; 3], cand: &mut dyn Iterator<Item = usize>| -> Option<usize> { cand.min_by_key(|&k| dist2(paleta[k], c)) };
    // 1. Índice de paleta de cada píxel plano; los de borde, sin asignar.
    let mut idx: Vec<Option<usize>> = vec![None; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let c = im.color(x as u32, y as u32);
            let plano = (-1..=1).all(|dy| (-1..=1).all(|dx| {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h { return true; }
                let n = im.color(nx as u32, ny as u32);
                (0..3).all(|i| (n[i] as i32 - c[i] as i32).abs() <= 10)
            }));
            if plano {
                idx[(y * w + x) as usize] = cercano(c, &mut (0..paleta.len()));
            }
        }
    }
    // 2. Los de borde, al más parecido de los colores planos de alrededor.
    let mut px = im.px.clone();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let k = match idx[i] {
                Some(k) => k,
                None => {
                    let mut vecinos = Vec::new();
                    for dy in -radio..=radio {
                        for dx in -radio..=radio {
                            let (nx, ny) = (x + dx, y + dy);
                            if nx >= 0 && ny >= 0 && nx < w && ny < h {
                                if let Some(k) = idx[(ny * w + nx) as usize] {
                                    if !vecinos.contains(&k) { vecinos.push(k); }
                                }
                            }
                        }
                    }
                    let c = im.color(x as u32, y as u32);
                    if vecinos.is_empty() { cercano(c, &mut (0..paleta.len())).unwrap_or(0) } else { cercano(c, &mut vecinos.into_iter()).unwrap_or(0) }
                }
            };
            px[i * 4..i * 4 + 3].copy_from_slice(&paleta[k]);
        }
    }
    Rgba { ancho: im.ancho, alto: im.alto, px }
}

/// ¿Sin suavizado? Casi ningún píxel (menos del 0,3 %) tiene un color intermedio, lejos de todos los de
/// su paleta. Contar colores no sirve: un dibujo en blanco y negro suavizado tiene pocos grises distintos.
pub fn sin_suavizado(im: &Rgba) -> bool {
    let p = detectar(im, 64);
    if p.is_empty() { return false; }
    let intermedios = im.px.chunks_exact(4).filter(|q| { let c = [q[0], q[1], q[2]]; p.iter().all(|k| dist2(*k, c) > 20 * 20) }).count();
    (intermedios as f64) < 0.003 * (im.ancho * im.alto) as f64
}
