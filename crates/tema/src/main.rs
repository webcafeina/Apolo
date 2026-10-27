//! `cargo run -p apolo-tema --bin tokens` (o `make tokens`): escribe
//! `frontend/src/tokens.css` y `web/src/estilos/tokens.css`
//! desde la paleta.

use std::path::Path;

fn main() {
    let raiz = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (r, css) in apolo_tema::tokens::FICHEROS {
        let ruta = raiz.join(r);
        if let Some(carpeta) = ruta.parent() {
            let _ = std::fs::create_dir_all(carpeta);
        }
        std::fs::write(&ruta, css())
            .unwrap_or_else(|e| panic!("No se pudo escribir {}: {e}", ruta.display()));
        println!("Escrito {r}");
    }
}
