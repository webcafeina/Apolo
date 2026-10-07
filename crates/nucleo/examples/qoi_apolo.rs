fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let d = std::fs::read(&a[0]).unwrap();
    let e = apolo_nucleo::formatos::qoi::leer_png(&d).unwrap();
    eprintln!("{}x{} canales {}", e.ancho, e.alto, e.canales);
    std::fs::write(&a[1], apolo_nucleo::formatos::qoi::codificar(&e).unwrap()).unwrap();
}
