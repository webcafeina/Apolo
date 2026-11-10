//! Un SVG a PNG con transparencia, con resvg: `cargo run -p apolo-vector --example rasterizar -- dibujo.svg 512
//! dibujo.png`. Para preparar imágenes de prueba y de la web (pruebas/vector/).

use resvg::{tiny_skia, usvg};

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let [_, entrada, lado, salida] = a.as_slice() else {
        eprintln!("Uso: rasterizar ENTRADA.svg LADO SALIDA.png");
        std::process::exit(2);
    };
    let lado: u32 = lado.parse().expect("LADO, en píxeles");
    let datos = std::fs::read(entrada).expect("leer el SVG");
    let arbol = usvg::Tree::from_data(&datos, &usvg::Options::default()).expect("SVG válido");
    let t = arbol.size();
    let k = lado as f32 / t.width().max(t.height());
    let (w, h) = (
        (t.width() * k).round() as u32,
        (t.height() * k).round() as u32,
    );
    let mut lienzo = tiny_skia::Pixmap::new(w, h).expect("tamaño");
    resvg::render(
        &arbol,
        tiny_skia::Transform::from_scale(k, k),
        &mut lienzo.as_mut(),
    );
    lienzo.save_png(salida).expect("escribir el PNG");
}
