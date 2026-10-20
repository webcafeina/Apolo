//! En Windows, el manifiesto con la página de códigos UTF-8 dentro del .exe: avifenc y cjxl
//! abren sus ficheros con fopen, y con la página de siempre una ruta con tildes no se abre
//! (empaquetado/windows/apolo.manifest). En los demás sistemas no hace nada.

use std::path::PathBuf;

fn main() {
    let manifiesto =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../empaquetado/windows/apolo.manifest");
    println!("cargo:rerun-if-changed={}", manifiesto.display());
    // El mismo manifiesto lo copia Tauri en un .rc línea a línea (src-tauri/build.rs): sin
    // declaración XML y en ASCII, o la ventana no arranca. Se mira aquí, en todos los sistemas,
    // porque la ventana no se compila en todas partes.
    let texto = std::fs::read_to_string(&manifiesto).expect("el manifiesto de Windows");
    assert!(
        texto.is_ascii() && !texto.trim_start().starts_with("<?xml"),
        "apolo.manifest: solo ASCII y sin <?xml …?>, o apolo-app.exe no arranca en Windows"
    );
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let manifiesto = manifiesto.canonicalize().expect("el manifiesto de Windows");
    let rc = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("apolo.rc");
    // 1 24: el manifiesto de la aplicación (RT_MANIFEST).
    let ruta = manifiesto.display().to_string().replace('\\', "\\\\");
    std::fs::write(&rc, format!("1 24 \"{ruta}\"\n")).unwrap();
    embed_resource::compile(&rc, embed_resource::NONE)
        .manifest_required()
        .expect("embed-resource");
}
