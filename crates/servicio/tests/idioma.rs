//! El idioma de los ajustes (ADR 0025). En un fichero aparte, que corre en su
//! propio proceso: `poner_idioma` cambia el idioma de todo el proceso, y las
//! demás pruebas del servicio comprueban textos en español a la vez.

use apolo_nucleo::idioma::{self, Idioma};
use apolo_servicio::{PreferenciaIdioma, Servicio};

#[test]
fn el_idioma_se_guarda_se_aplica_y_sale_en_el_inicio() {
    let carpeta =
        std::env::temp_dir().join(format!("apolo-servicio-idioma-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&carpeta);
    let s = Servicio::nuevo(carpeta.clone());
    assert_eq!(s.ajustes().idioma, PreferenciaIdioma::Sistema);
    // Crear el servicio no toca el idioma del proceso.
    assert_eq!(idioma::actual(), Idioma::Es);
    assert_eq!(s.poner_idioma(PreferenciaIdioma::En), Idioma::En);
    assert_eq!(idioma::actual(), Idioma::En);
    assert_eq!(s.inicio().idioma, Idioma::En);
    // Otra sesión lo encuentra guardado, y lo aplica al pedírselo.
    idioma::poner(Idioma::Es);
    let otra = Servicio::nuevo(carpeta.clone());
    assert_eq!(otra.ajustes().idioma, PreferenciaIdioma::En);
    assert_eq!(idioma::actual(), Idioma::Es);
    assert_eq!(otra.aplicar_idioma(), Idioma::En);
    assert_eq!(idioma::actual(), Idioma::En);
    let _ = std::fs::remove_dir_all(&carpeta);
}
