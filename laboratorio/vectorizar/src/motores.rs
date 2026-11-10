//! Los motores que se comparan. Cada uno recibe el original (opaco, sobre blanco) y devuelve un SVG del
//! mismo tamaño.

use crate::corpus::Grupo;
use crate::imagen::{self, Rgba};
use crate::{ajuste, paleta, potrace, subpixel};
use vtracer::{ColorImage, Config, FitMode, Hierarchical, Preset};

pub struct Motor {
    pub id: &'static str,
    pub descripcion: &'static str,
    /// Grupos en los que se prueba (vacío: todos).
    pub grupos: &'static [Grupo],
    pub correr: fn(&Rgba, Grupo) -> Result<String, String>,
}

impl Motor {
    pub fn aplica(&self, g: Grupo) -> bool {
        self.grupos.is_empty() || self.grupos.contains(&g)
    }
}

pub const MOTORES: &[Motor] = &[
    Motor { id: "apolo", descripcion: "Nuestro motor: detecta pixel art (lo traza a píxeles) e imágenes sin suavizado; si no, contorno por debajo del píxel, esquinas exactas (los lados rectos se prolongan hasta cruzarse), rectas y Bézier con error de 0,35 px y círculos de verdad.", grupos: &[], correr: apolo },
    Motor { id: "apolo-poli", descripcion: "Nuestro motor, versión polígono primero: el contorno por debajo del píxel se afina a polígono; los lados largos pasan a rectas exactas con la esquina en su cruce, las rachas de lados cortos a Bézier si ahorran nodos, y las puntas romas entre rectas se afilan.", grupos: &[], correr: apolo_poli },
    Motor { id: "apolo-crate", descripcion: "El motor de la aplicación (crates/vector), en automático: lo que vectorizará Apolo.", grupos: &[], correr: apolo_crate },
    Motor { id: "vtracer", descripcion: "VTracer 1.0 con sus opciones por defecto: agrupa colores, apila capas y ajusta Bézier.", grupos: &[], correr: vtracer_defecto },
    Motor { id: "vtracer-poster", descripcion: "VTracer con su preset «poster», pensado para colores planos.", grupos: &[], correr: vtracer_poster },
    Motor { id: "vtracer-mosaico", descripcion: "VTracer en mosaico (cada borde se ajusta una vez y lo comparten las dos regiones) y simplificación de 1 px.", grupos: &[], correr: vtracer_mosaico },
    Motor { id: "vtracer-byn", descripcion: "VTracer con su preset de blanco y negro.", grupos: &[Grupo::ByN], correr: vtracer_byn },
    Motor { id: "vtracer-pixel", descripcion: "VTracer trazando el contorno exacto de los píxeles, sin suavizar.", grupos: &[Grupo::PixelArt], correr: vtracer_pixel },
    Motor { id: "potrace", descripcion: "Potrace 1.16 sobre la máscara de oscuros, con sus ajustes por defecto.", grupos: &[Grupo::ByN], correr: potrace_byn },
    Motor { id: "potrace-color", descripcion: "Potrace a color: paleta nuestra y una capa por color, cada una cubriendo también las de encima, para que no queden huecos.", grupos: &[], correr: potrace_color },
    Motor { id: "paleta", descripcion: "Nuestra paleta automática (sin los colores del suavizado) y luego VTracer.", grupos: &[], correr: paleta_vtracer },
    Motor { id: "local", descripcion: "Paleta automática, pero cada píxel de borde solo puede ir a un color que tenga al lado (sin halos), y Potrace por capas.", grupos: &[Grupo::Logotipos, Grupo::Ilustraciones, Grupo::ByN], correr: local_potrace },
    Motor { id: "local-x3", descripcion: "Como «local», ampliando 3× antes, para que el borde caiga entre píxeles del original.", grupos: &[Grupo::Logotipos, Grupo::Ilustraciones, Grupo::ByN], correr: local_x3 },
    Motor { id: "subpixel", descripcion: "Nuestro: cada píxel de borde como mezcla de sus dos colores vecinos y el contorno donde la mezcla cruza la mitad, con precisión por debajo del píxel. Polígonos simplificados a 0,15 px, sin curvas todavía.", grupos: &[Grupo::Logotipos, Grupo::Ilustraciones, Grupo::ByN], correr: subpixel_poligonos },
    Motor { id: "paleta-x3", descripcion: "Como «paleta», pero ampliando la imagen 3× antes de trazar, para acercarse a los bordes por debajo del píxel.", grupos: &[], correr: paleta_x3 },
];

fn color_image(im: &Rgba) -> ColorImage {
    ColorImage { pixels: im.px.clone(), width: im.ancho as usize, height: im.alto as usize }
}

fn vt(im: &Rgba, cfg: &Config) -> Result<String, String> {
    cfg.build().map_err(|e| e.to_string())?.to_svg(&color_image(im)).map_err(|e| e.to_string())
}

fn vtracer_defecto(im: &Rgba, _: Grupo) -> Result<String, String> {
    vt(im, &Config::default())
}

fn vtracer_poster(im: &Rgba, _: Grupo) -> Result<String, String> {
    vt(im, &Config::from_preset(Preset::Poster))
}

fn vtracer_mosaico(im: &Rgba, _: Grupo) -> Result<String, String> {
    let mut c = Config::default();
    c.hierarchical = Hierarchical::Cutout;
    c.simplify = Some(1.0);
    vt(im, &c)
}

fn vtracer_byn(im: &Rgba, _: Grupo) -> Result<String, String> {
    vt(im, &Config::from_preset(Preset::Bw))
}

fn vtracer_pixel(im: &Rgba, _: Grupo) -> Result<String, String> {
    let mut c = Config::default();
    c.mode = FitMode::Pixel;
    c.filter_speckle = 0;
    vt(im, &c)
}

/// La paleta para cada grupo: la nuestra en lo plano, libimagequant en fotos, exacta en pixel art.
pub fn paleta_de(im: &Rgba, g: Grupo) -> Result<(Rgba, Vec<[u8; 3]>), String> {
    match g {
        Grupo::Fotos => paleta::cuantizar(im, 24),
        Grupo::PixelArt => {
            let p = paleta::exactos(im);
            if p.len() <= 64 { Ok((im.clone(), p)) } else { paleta::cuantizar(im, 64) }
        }
        _ => {
            let p = paleta::detectar(im, 48);
            if p.is_empty() { paleta::cuantizar(im, 24) } else { Ok((paleta::aplicar(im, &p), p)) }
        }
    }
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn potrace_byn(im: &Rgba, _: Grupo) -> Result<String, String> {
    let m: Vec<bool> = im.px.chunks_exact(4).map(|p| (p[0] as u32 * 299 + p[1] as u32 * 587 + p[2] as u32 * 114) < 128_000).collect();
    let (w, h) = (im.ancho, im.alto);
    let mut s = format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><rect width="{w}" height="{h}" fill="#ffffff"/>"##);
    if let Some(d) = potrace::trazar(&m, w as usize, h as usize, potrace::Ajustes::default()) {
        s.push_str(&format!(r##"<path fill="#000000" fill-rule="evenodd" d="{d}"/>"##));
    }
    s.push_str("</svg>");
    Ok(s)
}

fn potrace_color(im: &Rgba, g: Grupo) -> Result<String, String> {
    let (q, pal) = paleta_de(im, g)?;
    potrace_capas(&q, &pal, 1)
}

/// Una capa por color, de más a menos área; cada máscara incluye los colores que van encima, así los
/// bordes de dos colores no dejan una rendija.
fn potrace_capas(q: &Rgba, pal: &[[u8; 3]], escala: u32) -> Result<String, String> {
    potrace_capas_con(q, pal, escala, potrace::Ajustes::default())
}

fn potrace_capas_con(q: &Rgba, pal: &[[u8; 3]], escala: u32, ajustes: potrace::Ajustes) -> Result<String, String> {
    let n = (q.ancho * q.alto) as usize;
    let indice: std::collections::HashMap<[u8; 3], usize> = pal.iter().enumerate().map(|(i, c)| (*c, i)).collect();
    let mut idx = vec![0usize; n];
    let mut area = vec![0usize; pal.len()];
    for (i, p) in q.px.chunks_exact(4).enumerate() {
        let k = *indice.get(&[p[0], p[1], p[2]]).ok_or("color fuera de la paleta")?;
        idx[i] = k;
        area[k] += 1;
    }
    let mut orden: Vec<usize> = (0..pal.len()).filter(|&k| area[k] > 0).collect();
    orden.sort_by(|a, b| area[*b].cmp(&area[*a]).then(a.cmp(b)));
    let mut rango = vec![0usize; pal.len()];
    for (r, &k) in orden.iter().enumerate() {
        rango[k] = r;
    }
    let (w, h) = (q.ancho / escala, q.alto / escala);
    let mut s = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {} {}">"#, q.ancho, q.alto);
    if let Some(&fondo) = orden.first() {
        s.push_str(&format!(r#"<rect width="{}" height="{}" fill="{}"/>"#, q.ancho, q.alto, hex(pal[fondo])));
    }
    for (r, &k) in orden.iter().enumerate().skip(1) {
        let m: Vec<bool> = idx.iter().map(|&i| rango[i] >= r).collect();
        if let Some(d) = potrace::trazar(&m, q.ancho as usize, q.alto as usize, ajustes) {
            s.push_str(&format!(r#"<path fill="{}" fill-rule="evenodd" d="{d}"/>"#, hex(pal[k])));
        }
    }
    s.push_str("</svg>");
    Ok(s)
}

fn paleta_vtracer(im: &Rgba, g: Grupo) -> Result<String, String> {
    let (q, _) = paleta_de(im, g)?;
    let mut c = Config::default();
    c.color_precision = 8;
    c.layer_difference = 1;
    vt(&q, &c)
}

fn paleta_x3(im: &Rgba, g: Grupo) -> Result<String, String> {
    let (_, pal) = paleta_de(im, g)?;
    let grande = imagen::ampliar_suave(im, 3);
    let q = paleta::aplicar(&grande, &pal);
    let mut c = Config::default();
    c.color_precision = 8;
    c.layer_difference = 1;
    c.filter_speckle = 8;
    let svg = vt(&q, &c)?;
    Ok(reescalar(&svg, im.ancho, im.alto))
}

/// Deja un SVG trazado sobre una imagen ampliada con el tamaño del original, sin tocar las coordenadas.
pub fn reescalar(svg: &str, ancho: u32, alto: u32) -> String {
    let Some(ini) = svg.find("<svg") else { return svg.to_string() };
    let Some(fin) = svg[ini..].find('>').map(|f| ini + f) else { return svg.to_string() };
    let etiqueta = &svg[ini..fin];
    let atributo = |nombre: &str| -> Option<String> {
        let clave = format!(" {nombre}=\"");
        let p = etiqueta.find(&clave)? + clave.len();
        Some(etiqueta[p..p + etiqueta[p..].find('"')?].to_string())
    };
    let (Some(wg), Some(hg)) = (atributo("width"), atributo("height")) else { return svg.to_string() };
    let mut nueva = etiqueta.replace(&format!(" width=\"{wg}\""), &format!(" width=\"{ancho}\"")).replace(&format!(" height=\"{hg}\""), &format!(" height=\"{alto}\""));
    if atributo("viewBox").is_none() {
        nueva.push_str(&format!(" viewBox=\"0 0 {wg} {hg}\""));
    }
    format!("{}{}{}", &svg[..ini], nueva, &svg[fin..])
}

fn local_potrace(im: &Rgba, g: Grupo) -> Result<String, String> {
    let (_, pal) = paleta_de(im, g)?;
    let q = paleta::aplicar_local(im, &pal, 2);
    potrace_capas(&q, &pal, 1)
}

fn local_x3(im: &Rgba, g: Grupo) -> Result<String, String> {
    let (_, pal) = paleta_de(im, g)?;
    let grande = imagen::ampliar_suave(im, 3);
    let q = paleta::aplicar_local(&grande, &pal, 6);
    let mut a = potrace::Ajustes::default();
    a.turdsize = 18;
    potrace_capas_con(&q, &pal, 3, a)
}

fn subpixel_poligonos(im: &Rgba, g: Grupo) -> Result<String, String> {
    use std::fmt::Write;
    let (_, pal) = paleta_de(im, g)?;
    let mezcla = subpixel::mezclas(im, &pal, 2);
    // Orden de capas por área, como en Potrace por capas: cada capa incluye las de encima.
    let mut area = vec![0f64; pal.len()];
    for m in &mezcla {
        area[m.a as usize] += m.t as f64;
        area[m.b as usize] += 1.0 - m.t as f64;
    }
    let mut orden: Vec<usize> = (0..pal.len()).filter(|&k| area[k] > 0.5).collect();
    orden.sort_by(|a, b| area[*b].total_cmp(&area[*a]).then(a.cmp(b)));
    let (w, h) = (im.ancho, im.alto);
    let mut s = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#);
    if let Some(&fondo) = orden.first() {
        let _ = write!(s, r#"<rect width="{w}" height="{h}" fill="{}"/>"#, hex(pal[fondo]));
    }
    for (r, &k) in orden.iter().enumerate().skip(1) {
        let mut dentro = vec![false; pal.len()];
        for &q in &orden[r..] {
            dentro[q] = true;
        }
        let cob = subpixel::cobertura(&mezcla, &dentro);
        let mut d = String::new();
        for lazo in subpixel::contornos(&cob, w as usize, h as usize) {
            let p = subpixel::simplificar(&lazo, 0.15);
            if p.len() < 3 { continue; }
            let _ = write!(d, "M{:.2} {:.2}", p[0].0, p[0].1);
            for q in &p[1..] {
                let _ = write!(d, "L{:.2} {:.2}", q.0, q.1);
            }
            d.push('Z');
        }
        if !d.is_empty() {
            let _ = write!(s, r#"<path fill="{}" fill-rule="evenodd" d="{d}"/>"#, hex(pal[k]));
        }
    }
    s.push_str("</svg>");
    Ok(s)
}

/// Pixel art: pocos colores exactos y, o es pequeño, o todos los tramos de color miden un múltiplo de un
/// bloque de 2 o más píxeles (pixel art ampliado).
pub fn es_pixel_art(im: &Rgba) -> bool {
    if paleta::exactos(im).len() > 64 { return false; }
    if im.ancho.max(im.alto) <= 64 { return true; }
    if !paleta::sin_suavizado(im) { return false; }
    fn mcd(a: u32, b: u32) -> u32 { if b == 0 { a } else { mcd(b, a % b) } }
    let mut g = 0;
    for y in 0..im.alto {
        let mut run = 1;
        for x in 1..im.ancho {
            if im.color(x, y) == im.color(x - 1, y) { run += 1 } else { g = mcd(g, run); run = 1; if g == 1 { return false; } }
        }
    }
    for x in 0..im.ancho {
        let mut run = 1;
        for y in 1..im.alto {
            if im.color(x, y) == im.color(x, y - 1) { run += 1 } else { g = mcd(g, run); run = 1; if g == 1 { return false; } }
        }
    }
    g >= 2
}

fn apolo(im: &Rgba, g: Grupo) -> Result<String, String> {
    apolo_con(im, g, false)
}

fn apolo_poli(im: &Rgba, g: Grupo) -> Result<String, String> {
    apolo_con(im, g, true)
}

fn apolo_con(im: &Rgba, g: Grupo, poligono: bool) -> Result<String, String> {
    if es_pixel_art(im) {
        return vtracer_pixel(im, g);
    }
    let duro = paleta::sin_suavizado(im);
    let (_, pal) = paleta_de(im, g)?;
    let mut mezcla = subpixel::mezclas(im, &pal, 2);
    let degradados = poligono && g != Grupo::Fotos && std::env::var("APOLO_DEGRADADOS").map(|v| v != "0").unwrap_or(true);
    let rellenos: Vec<Option<crate::degradado::Relleno>> = if degradados { crate::degradado::fusionar(im, &mut mezcla, pal.len()) } else { (0..pal.len()).map(|_| None).collect() };
    let mut area = vec![0f64; pal.len()];
    for m in &mezcla {
        area[m.a as usize] += m.t as f64;
        area[m.b as usize] += 1.0 - m.t as f64;
    }
    let mut por_area: Vec<usize> = (0..pal.len()).filter(|&k| area[k] > 0.5).collect();
    por_area.sort_by(|a, b| area[*b].total_cmp(&area[*a]).then(a.cmp(b)));
    // La otra opción: arriba los colores que forman islas cerradas dentro de otros (estrellas, ojos,
    // letras), que así se dibujan una vez, como figura, y no como agujero en todas las capas de debajo.
    // Ninguna gana siempre: se pintan las dos y se queda la que más se parece al original.
    let islas = islas_por_color(&mezcla, im.ancho as usize, im.alto as usize, pal.len());
    let mut por_islas = por_area.clone();
    por_islas.sort_by(|a, b| islas[*a].cmp(&islas[*b]).then(area[*b].total_cmp(&area[*a])).then(a.cmp(b)));
    let modo = std::env::var("APOLO_ORDEN").unwrap_or_default();
    let candidatos: Vec<&Vec<usize>> = match modo.as_str() {
        "area" => vec![&por_area],
        "islas" => vec![&por_islas],
        _ if por_islas == por_area => vec![&por_area],
        _ => vec![&por_area, &por_islas],
    };
    let varios = candidatos.len() > 1;
    let mut mejor: Option<(f64, String)> = None;
    for orden in candidatos {
        let svg = pintar(im, g, poligono, duro, &pal, &mezcla, orden, &rellenos)?;
        let e = if varios { error_cuadratico(im, &svg) } else { 0.0 };
        if mejor.as_ref().map(|m| e < m.0).unwrap_or(true) { mejor = Some((e, svg)); }
    }
    mejor.map(|m| m.1).ok_or_else(|| "sin capas".to_string())
}

/// Error cuadrático medio entre el original y el SVG rasterizado.
fn error_cuadratico(im: &Rgba, svg: &str) -> f64 {
    match imagen::rasterizar(svg.as_bytes(), im.ancho, im.alto, true) {
        Ok(r) => im.px.iter().zip(&r.px).map(|(a, b)| (*a as f64 - *b as f64).powi(2)).sum::<f64>() / im.px.len() as f64,
        Err(_) => f64::MAX,
    }
}

#[allow(clippy::too_many_arguments)]
fn pintar(im: &Rgba, g: Grupo, poligono: bool, duro: bool, pal: &[[u8; 3]], mezcla: &[subpixel::Mezcla], orden: &[usize], rellenos: &[Option<crate::degradado::Relleno>]) -> Result<String, String> {
    use std::fmt::Write;
    let var = |n: &str, d: f64| std::env::var(n).ok().and_then(|v| v.parse::<f64>().ok()).unwrap_or(d);
    let o = ajuste::Opciones {
        tol: if duro { var("APOLO_TOL_DURO", 0.75) } else { var("APOLO_TOL", 0.35) },
        angulo: var("APOLO_ANGULO", 50.0),
        circulos: var("APOLO_CIRCULOS", 1.0) > 0.0,
        esquinas_exactas: var("APOLO_AFILAR", 1.0) > 0.0,
    };
    let minima = if g == Grupo::Fotos { 6.0 } else { 0.8 };
    let (w, h) = (im.ancho, im.alto);
    let mut s = format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">"#);
    if let Some(&fondo) = orden.first() {
        let _ = write!(s, r#"<rect width="{w}" height="{h}" fill="{}"/>"#, hex(pal[fondo]));
    }
    // Degradados: por cada color, los píxeles de su interior (sin mezcla), y si cambian suavemente, un
    // degradado en vez del color plano (degradado.rs).
    let mut relleno: Vec<String> = pal.iter().map(|c| hex(*c)).collect();
    {
        let mut defs = String::new();
        for (k, r) in rellenos.iter().enumerate() {
            if let Some(r) = r {
                let id = format!("d{k}");
                defs.push_str(&crate::degradado::defs(&id, r));
                relleno[k] = format!("url(#{id})");
            }
        }
        if !defs.is_empty() {
            // El fondo también puede ser un degradado: se reescribe su rect.
            if let Some(&fondo) = orden.first() {
                s = s.replacen(&format!(r#"fill="{}"/>"#, hex(pal[fondo])), &format!(r#"fill="{}"/>"#, relleno[fondo]), 1);
            }
            let ini = s.find('>').map(|x| x + 1).unwrap_or(0);
            s.insert_str(ini, &format!("<defs>{defs}</defs>"));
        }
    }
    for (r, &k) in orden.iter().enumerate().skip(1) {
        let mut dentro = vec![false; pal.len()];
        for &q in &orden[r..] { dentro[q] = true; }
        let cob = subpixel::cobertura(&mezcla, &dentro);
        let mut d = String::new();
        for lazo in subpixel::contornos(&cob, w as usize, h as usize) {
            if area_lazo(&lazo).abs() < minima { continue; }
            let f = if poligono {
                let muestra = |x: f64, y: f64| -> f32 {
                    // Bilineal sobre los centros de los píxeles; fuera de la imagen, 0.
                    let (fx, fy) = (x - 0.5, y - 0.5);
                    let (x0, y0) = (fx.floor(), fy.floor());
                    let (tx, ty) = ((fx - x0) as f32, (fy - y0) as f32);
                    let v = |i: f64, j: f64| -> f32 {
                        if i < 0.0 || j < 0.0 || i >= w as f64 || j >= h as f64 { 0.0 } else { cob[j as usize * w as usize + i as usize] }
                    };
                    let a = v(x0, y0) * (1.0 - tx) + v(x0 + 1.0, y0) * tx;
                    let b = v(x0, y0 + 1.0) * (1.0 - tx) + v(x0 + 1.0, y0 + 1.0) * tx;
                    a * (1.0 - ty) + b * ty
                };
                crate::poli::ajustar(&lazo, var("APOLO_TOL_POLI", if duro { 0.5 } else { 0.3 }), var("APOLO_TOL_CURVA", 0.15), &muestra, (w, h), g != Grupo::Fotos)
            } else {
                ajuste::ajustar(&lazo, o)
            };
            if let Some(f) = f { ajuste::d(&f, &mut d); }
        }
        if !d.is_empty() {
            let _ = write!(s, r#"<path fill="{}" fill-rule="evenodd" d="{d}"/>"#, relleno[k]);
        }
    }
    s.push_str("</svg>");
    Ok(s)
}

fn area_lazo(p: &[(f64, f64)]) -> f64 {
    let n = p.len();
    (0..n).map(|i| { let (a, b) = (p[i], p[(i + 1) % n]); a.0 * b.1 - b.0 * a.1 }).sum::<f64>() / 2.0
}

/// Cuántas islas tiene cada color: componentes conexas (de 4 px o más) que no tocan el borde de la imagen.
fn islas_por_color(m: &[subpixel::Mezcla], w: usize, h: usize, n: usize) -> Vec<usize> {
    let etiqueta: Vec<u16> = m.iter().map(|x| if x.t >= 0.5 { x.a } else { x.b }).collect();
    let mut visto = vec![false; w * h];
    let mut islas = vec![0usize; n];
    let mut pila = Vec::new();
    for ini in 0..w * h {
        if visto[ini] { continue; }
        let c = etiqueta[ini];
        let (mut tam, mut borde) = (0usize, false);
        visto[ini] = true;
        pila.push(ini);
        while let Some(i) = pila.pop() {
            tam += 1;
            let (x, y) = (i % w, i / w);
            if x == 0 || y == 0 || x == w - 1 || y == h - 1 { borde = true; }
            let mut mira = |j: usize| { if !visto[j] && etiqueta[j] == c { visto[j] = true; pila.push(j); } };
            if x > 0 { mira(i - 1); }
            if x + 1 < w { mira(i + 1); }
            if y > 0 { mira(i - w); }
            if y + 1 < h { mira(i + w); }
        }
        if !borde && tam >= 4 { islas[c as usize] += 1; }
    }
    islas
}

fn apolo_crate(im: &Rgba, _: Grupo) -> Result<String, String> {
    let doc = apolo_vector::vectorizar(&apolo_vector::Rgba::nueva(im.ancho, im.alto, im.px.clone()), &apolo_vector::OpcionesVector::default())?;
    Ok(apolo_vector::svg::escribir(&doc))
}
