//! La vista previa: el documento rasterizado con resvg al tamaño del original, sobre blanco. Es lo que
//! enseña el comparador y lo que mide la nota.

use crate::modelo::Documento;
use resvg::{tiny_skia, usvg};

/// RGBA de `doc.ancho` × `doc.alto`.
pub fn rasterizar(doc: &Documento) -> Result<Vec<u8>, String> {
    rasterizar_svg(crate::svg::escribir(doc).as_bytes(), doc.ancho, doc.alto)
}

/// Un SVG cualquiera a `ancho` × `alto`, sobre blanco.
pub fn rasterizar_svg(svg: &[u8], ancho: u32, alto: u32) -> Result<Vec<u8>, String> {
    let arbol = usvg::Tree::from_data(svg, &usvg::Options::default()).map_err(|e| e.to_string())?;
    let t = arbol.size();
    let mut lienzo = tiny_skia::Pixmap::new(ancho, alto).ok_or("lienzo vacío")?;
    lienzo.fill(tiny_skia::Color::WHITE);
    resvg::render(
        &arbol,
        tiny_skia::Transform::from_scale(ancho as f32 / t.width(), alto as f32 / t.height()),
        &mut lienzo.as_mut(),
    );
    Ok(lienzo.take())
}

/// Los contornos y los nodos sobre el dibujo apagado ([`crate::svg::contornos`]), del tamaño del documento.
pub fn rasterizar_contornos(doc: &Documento) -> Result<Vec<u8>, String> {
    rasterizar_svg(crate::svg::contornos(doc).as_bytes(), doc.ancho, doc.alto)
}
