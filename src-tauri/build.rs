fn main() {
    // En Windows, el manifiesto propio: el de Tauri más la página de códigos UTF-8, para que
    // avifenc y cjxl abran rutas con tildes (empaquetado/windows/apolo.manifest).
    let windows = tauri_build::WindowsAttributes::new()
        .app_manifest(include_str!("../empaquetado/windows/apolo.manifest"));
    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("tauri-build");
}
