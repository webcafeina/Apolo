//! Mapas de bits: preparar los originales, rasterizar los SVG con resvg y medir con `apolo medir`.

use resvg::{tiny_skia, usvg};
use std::path::Path;
use std::process::Command;

/// RGBA de 8 bits, opaco (todo se aplana sobre blanco).
#[derive(Clone)]
pub struct Rgba {
    pub ancho: u32,
    pub alto: u32,
    pub px: Vec<u8>,
}

impl Rgba {
    pub fn guardar_png(&self, ruta: &Path) -> Result<(), String> {
        image::save_buffer(ruta, &self.px, self.ancho, self.alto, image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())
    }
    pub fn color(&self, x: u32, y: u32) -> [u8; 3] {
        let i = ((y * self.ancho + x) * 4) as usize;
        [self.px[i], self.px[i + 1], self.px[i + 2]]
    }
}

fn opciones(suavizado: bool) -> usvg::Options<'static> {
    let mut o = usvg::Options::default();
    if !suavizado {
        o.shape_rendering = usvg::ShapeRendering::CrispEdges;
    }
    o
}

/// Rasteriza un SVG a `ancho`×`alto` exactos, sobre blanco.
pub fn rasterizar(svg: &[u8], ancho: u32, alto: u32, suavizado: bool) -> Result<Rgba, String> {
    let arbol = usvg::Tree::from_data(svg, &opciones(suavizado)).map_err(|e| e.to_string())?;
    let t = arbol.size();
    let mut lienzo = tiny_skia::Pixmap::new(ancho, alto).ok_or("lienzo vacío")?;
    lienzo.fill(tiny_skia::Color::WHITE);
    let escala = tiny_skia::Transform::from_scale(ancho as f32 / t.width(), alto as f32 / t.height());
    resvg::render(&arbol, escala, &mut lienzo.as_mut());
    Ok(Rgba { ancho, alto, px: lienzo.take() })
}

/// El tamaño natural de un SVG, con el lado mayor escalado a `lado`.
pub fn tamano_svg(svg: &[u8], lado: u32) -> Result<(u32, u32), String> {
    let arbol = usvg::Tree::from_data(svg, &usvg::Options::default()).map_err(|e| e.to_string())?;
    let t = arbol.size();
    let k = lado as f32 / t.width().max(t.height());
    Ok(((t.width() * k).round().max(1.0) as u32, (t.height() * k).round().max(1.0) as u32))
}

/// Una foto, aplanada sobre blanco y reducida a `lado_max`.
pub fn foto(ruta: &Path, lado_max: u32) -> Result<Rgba, String> {
    let mut im = image::open(ruta).map_err(|e| format!("{}: {e}", ruta.display()))?.to_rgba8();
    let (w, h) = im.dimensions();
    if w.max(h) > lado_max {
        let k = lado_max as f32 / w.max(h) as f32;
        im = image::imageops::resize(&im, (w as f32 * k).round() as u32, (h as f32 * k).round() as u32, image::imageops::FilterType::Lanczos3);
    }
    let (w, h) = im.dimensions();
    let mut px = im.into_raw();
    for p in px.chunks_exact_mut(4) {
        let a = p[3] as u32;
        for c in &mut p[..3] {
            *c = ((*c as u32 * a + 255 * (255 - a) + 127) / 255) as u8;
        }
        p[3] = 255;
    }
    Ok(Rgba { ancho: w, alto: h, px })
}

/// Amplía sin filtrar (pixel art).
pub fn ampliar_vecino(im: &Rgba, k: u32) -> Rgba {
    let (w, h) = (im.ancho * k, im.alto * k);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let o = (((y / k) * im.ancho + x / k) * 4) as usize;
            let d = ((y * w + x) * 4) as usize;
            px[d..d + 4].copy_from_slice(&im.px[o..o + 4]);
        }
    }
    Rgba { ancho: w, alto: h, px }
}

/// Amplía con Catmull-Rom (para trazar con más resolución que la del original).
pub fn ampliar_suave(im: &Rgba, k: u32) -> Rgba {
    let b = image::RgbaImage::from_raw(im.ancho, im.alto, im.px.clone()).expect("tamaño");
    let r = image::imageops::resize(&b, im.ancho * k, im.alto * k, image::imageops::FilterType::CatmullRom);
    Rgba { ancho: im.ancho * k, alto: im.alto * k, px: r.into_raw() }
}

/// La nota SSIMULACRA 2 de `apolo medir` (la misma de libjxl).
pub fn nota(apolo: &Path, original: &Path, resultado: &Path) -> Result<f64, String> {
    let s = Command::new(apolo).args(["medir", "--idioma", "en"]).arg(original).arg(resultado).output().map_err(|e| e.to_string())?;
    let texto = String::from_utf8_lossy(&s.stdout);
    let linea = texto.lines().find(|l| l.starts_with("SSIMULACRA 2")).ok_or_else(|| format!("sin nota: {texto}{}", String::from_utf8_lossy(&s.stderr)))?;
    linea.trim_start_matches("SSIMULACRA 2").trim().replace(',', ".").parse::<f64>().map_err(|e| e.to_string())
}
