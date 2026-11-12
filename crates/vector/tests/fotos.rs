//! Qué es una foto: la detección no depende del tamaño de la imagen (ADR 0039).

use apolo_vector::{Rgba, TipoImagen, tipo_efectivo};
use std::path::Path;

/// Una «foto» suave de `w` × `h`: dos degradados lentos y un grano de ±1 en el azul. A su tamaño repite mucho
/// los colores (0,03 colores distintos por píxel, como los arándanos del banco a 1.280 px, que pasaban por
/// dibujo), pero en cualquier trozo cada píxel casi tiene el suyo.
fn foto_suave(w: u32, h: u32) -> Rgba {
    let mut px = Vec::with_capacity((w * h * 4) as usize);
    let mut s: u32 = 12345;
    for y in 0..h {
        for x in 0..w {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            let grano = (s >> 16) % 3;
            px.extend([(x / 10) as u8, (y / 10) as u8, 120 + grano as u8, 255]);
        }
    }
    Rgba::nueva(w, h, px)
}

#[test]
fn una_foto_grande_es_foto() {
    for (w, h) in [(1280, 1104), (640, 552)] {
        assert_eq!(
            tipo_efectivo(&foto_suave(w, h), TipoImagen::Auto),
            TipoImagen::Foto,
            "{w}×{h}"
        );
    }
}

#[test]
fn un_dibujo_no_es_foto() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/vector");
    for f in ["logo.png", "apolo-plano.png"] {
        let p = resvg::tiny_skia::Pixmap::load_png(dir.join(f)).unwrap();
        let im = Rgba::nueva(p.width(), p.height(), p.data().to_vec());
        assert_ne!(
            tipo_efectivo(&im, TipoImagen::Auto),
            TipoImagen::Foto,
            "{f}"
        );
    }
}
