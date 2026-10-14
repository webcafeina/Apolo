//! `docs/mcp/catalogo.md` está al día con el código. Con `APOLO_REGENERAR=1`
//! (lo que hace `make catalogo-mcp`) lo escribe en vez de comprobarlo.

use std::path::Path;

#[test]
fn el_catalogo_esta_al_dia() {
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/mcp/catalogo.md");
    let nuevo = apolo_mcp::catalogo::catalogo();
    if std::env::var_os("APOLO_REGENERAR").is_some() {
        std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        std::fs::write(&ruta, &nuevo).unwrap();
        return;
    }
    let actual = std::fs::read_to_string(&ruta).unwrap_or_default();
    assert!(
        actual == nuevo,
        "docs/mcp/catalogo.md no está al día: make catalogo-mcp"
    );
}
