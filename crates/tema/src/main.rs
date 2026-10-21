//! `cargo run -p apolo-tema --bin tokens` (o `make tokens`): escribe
//! `frontend/src/tokens.css` y `web/tokens.css` desde la paleta.

use std::path::Path;

fn main() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for r in apolo_tema::tokens::RUTAS {
        let ruta = raiz.join(r);
        std::fs::write(&ruta, apolo_tema::tokens::css())
            .unwrap_or_else(|e| panic!("No se pudo escribir {}: {e}", ruta.display()));
        println!("Escrito {r}");
    }
}
