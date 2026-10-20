fn main() {
    // En Windows, el manifiesto propio: el de Tauri (Common-Controls 6, para los diálogos) más la
    // página de códigos UTF-8, para que avifenc y cjxl, que abren sus ficheros con fopen, abran
    // rutas con tildes (empaquetado/windows/apolo.manifest).
    //
    // Tauri no lo mete como fichero: lo copia línea a línea en un .rc, con un espacio delante, y
    // rc.exe lo lee en cp1252. Por eso el manifiesto va sin `<?xml …?>` y solo en ASCII: con la
    // declaración o con una tilde, Windows no abre Apolo («la configuración en paralelo no es
    // correcta», docs/trampas.md). Lo vigila crates/cli/build.rs.
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("../empaquetado/windows/apolo.manifest"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("tauri-build");
}
