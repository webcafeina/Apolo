//! `cargo run -p apolo-tema --bin tokens` (o `make tokens`): escribe
//! `frontend/src/tokens.css` desde la paleta.

use std::path::Path;

fn main() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ruta = raiz.join(apolo_tema::tokens::RUTA);
    std::fs::write(&ruta, apolo_tema::tokens::css())
        .unwrap_or_else(|e| panic!("No se pudo escribir {}: {e}", ruta.display()));
    println!("Escrito {}", apolo_tema::tokens::RUTA);
}
