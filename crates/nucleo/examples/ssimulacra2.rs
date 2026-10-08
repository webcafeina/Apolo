//! `cargo run --example ssimulacra2 -- original distorsionada`: la nota de
//! Apolo, para compararla a mano con la herramienta `ssimulacra2` de libjxl.
fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let leer = |r: &str| apolo_nucleo::vista::decodificar(&std::fs::read(r).unwrap()).unwrap();
    let (w, h, x) = leer(&a[0]);
    let (_, _, y) = leer(&a[1]);
    let alfa = x.as_chunks::<4>().0.iter().any(|p| p[3] != 255);
    let (x, y, c) = if alfa {
        (x, y, 4)
    } else {
        let rgb = |v: &[u8]| {
            v.as_chunks::<4>()
                .0
                .iter()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect::<Vec<u8>>()
        };
        (rgb(&x), rgb(&y), 3)
    };
    println!(
        "{:.8}",
        apolo_avifjxl::ssimulacra2(&x, &y, w, h, c).unwrap()
    );
}
