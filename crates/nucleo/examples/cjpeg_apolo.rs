//! `cargo run --example cjpeg_apolo -- [opciones de cjpeg] -outfile salida entrada`:
//! el cjpeg de Apolo, para comparar a mano con el oficial.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = apolo_nucleo::jpeg::opciones::leer_orden(&args).expect("orden");
    let datos = std::fs::read(o.entrada.expect("entrada")).expect("leer");
    let e = apolo_nucleo::jpeg::leer(&datos).expect("decodificar");
    let extra = apolo_nucleo::jpeg::cjpeg::Extra {
        icc: o.icc.map(|r| std::fs::read(r).expect("icc")),
        estricto: o.estricto,
    };
    let r =
        apolo_nucleo::jpeg::cjpeg::ejecutar(&e, &o.opciones.orden(), &extra).expect("codificar");
    std::fs::write(o.salida.expect("salida"), r).unwrap();
}
