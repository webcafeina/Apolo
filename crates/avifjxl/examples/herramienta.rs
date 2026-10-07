//! `cargo run --example herramienta -- avifenc|avifdec|cjxl|djxl ARGUMENTOS…`:
//! la herramienta compilada dentro, para compararla con la oficial a mano.
use apolo_avifjxl::{Herramienta, ejecutar};

fn main() {
    let mut args = std::env::args().skip(1);
    let herramienta = match args.next().as_deref() {
        Some("avifenc") => Herramienta::Avifenc,
        Some("avifdec") => Herramienta::Avifdec,
        Some("cjxl") => Herramienta::Cjxl,
        Some("djxl") => Herramienta::Djxl,
        _ => {
            eprintln!("uso: herramienta avifenc|avifdec|cjxl|djxl ARGUMENTOS…");
            std::process::exit(2);
        }
    };
    let resto: Vec<String> = args.collect();
    std::process::exit(ejecutar(herramienta, &resto));
}
