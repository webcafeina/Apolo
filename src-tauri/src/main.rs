// En Windows, sin esto se abre también una consola detrás de la ventana.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    apolo_app::arrancar()
}
