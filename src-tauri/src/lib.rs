//! La aplicación de ventana: órdenes de Tauri sobre el núcleo.
//!
//! Aquí no se trabaja con imágenes. Cada orden traduce lo que pide la interfaz
//! a una llamada a `apolo_nucleo` y devuelve lo que ésta conteste.

use apolo_nucleo::Motor;

/// Los motores enlazados y sus versiones, para «Acerca de».
#[tauri::command]
fn motores() -> Vec<Motor> {
    apolo_nucleo::motores()
}

pub fn arrancar() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![motores])
        .run(tauri::generate_context!())
        .expect("No se pudo arrancar Apolo");
}
