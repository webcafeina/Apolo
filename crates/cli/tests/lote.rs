//! `apolo lote` de punta a punta, con el binario de verdad.

use std::path::{Path, PathBuf};
use std::process::Command;

/// El binario, en español: sin `--idioma`, la CLI usa el idioma del sistema, y
/// estas pruebas comparan textos en español.
fn apolo() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_apolo"));
    c.env("LC_ALL", "es_ES.UTF-8");
    c
}

/// El binario, en inglés por el sistema (sin `--idioma`).
fn apolo_en() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_apolo"));
    c.env("LC_ALL", "en_US.UTF-8");
    c
}

fn carpeta(nombre: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("apolo-cli-lote-{nombre}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pruebas/corpus");
    std::fs::create_dir_all(d.join("fotos/sub")).unwrap();
    std::fs::copy(corpus.join("foto.webp"), d.join("fotos/foto.webp")).unwrap();
    std::fs::copy(corpus.join("orientacion-6.png"), d.join("fotos/sub/b.png")).unwrap();
    d
}

#[test]
fn convierte_repite_subcarpetas_y_no_pisa() {
    let d = carpeta("bien");
    let salida = d.join("fotos-webp");
    let r = apolo()
        .args(["lote", "-q"])
        .arg(d.join("fotos"))
        .args(["--", "-q", "50"])
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert!(String::from_utf8_lossy(&r.stdout).starts_with("2 convertidas"));
    assert!(salida.join("foto.webp").exists());
    assert!(salida.join("sub/b.webp").exists());

    // Otra vez, a la misma salida (ahora explícita): nada se pisa.
    let r = apolo()
        .args(["lote", "-q"])
        .arg(d.join("fotos"))
        .arg("--salida")
        .arg(&salida)
        .output()
        .unwrap();
    assert!(r.status.success());
    assert!(salida.join("foto-2.webp").exists());
    assert!(salida.join("sub/b-2.webp").exists());
}

#[test]
fn una_imagen_rota_falla_sola_y_la_salida_lo_dice() {
    let d = carpeta("rota");
    std::fs::write(d.join("fotos/rota.png"), b"no soy un png").unwrap();
    let r = apolo()
        .args(["lote", "-q"])
        .arg(d.join("fotos"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    let texto = String::from_utf8_lossy(&r.stdout);
    assert!(texto.contains("2 convertidas"), "{texto}");
    assert!(texto.contains("1 no se pudo convertir"), "{texto}");
}

#[test]
fn un_preset_que_no_existe_no_empieza() {
    let d = carpeta("preset");
    let r = apolo()
        .args(["lote", "--preset", "No existe"])
        .arg(d.join("fotos"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("No hay ningún preset"));
    assert!(!d.join("fotos-webp").exists());
}

#[test]
fn la_ayuda_sale_en_ingles_con_idioma_en() {
    let r = apolo().args(["--idioma", "en", "--help"]).output().unwrap();
    assert!(r.status.success());
    let texto = String::from_utf8_lossy(&r.stdout);
    assert!(texto.contains("Optimizes images"), "{texto}");
    assert!(!texto.contains("Optimiza imágenes"), "{texto}");
    // Y con el sistema en inglés, sin pedirlo.
    let r = apolo_en().args(["lote", "--help"]).output().unwrap();
    assert!(String::from_utf8_lossy(&r.stdout).contains("Converts whole folders"));
}

#[test]
fn un_error_en_ingles() {
    // `--idioma` detrás de `webp`: se quita antes de que lo lea cwebp.
    let r = apolo()
        .args(["webp", "--idioma", "en", "no-existe.png", "-o", "x.webp"])
        .output()
        .unwrap();
    assert!(!r.status.success());
    let texto = String::from_utf8_lossy(&r.stderr);
    assert!(texto.contains("Can't read “no-existe.png”"), "{texto}");
}

#[test]
fn un_idioma_que_no_hay_no_empieza() {
    let r = apolo()
        .args(["--idioma", "fr", "motores"])
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&r.stderr).contains("--idioma fr"));
}

#[test]
fn el_resumen_del_lote_en_ingles() {
    let d = carpeta("ingles");
    let r = apolo()
        .args(["--idioma=en", "lote", "-q"])
        .arg(d.join("fotos"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    let texto = String::from_utf8_lossy(&r.stdout);
    assert!(texto.starts_with("2 converted in "), "{texto}");
    // Con punto decimal, y sin espacio antes del «%».
    assert!(texto.contains(" KB") && !texto.contains(','), "{texto}");
    assert!(texto.contains("%)") && !texto.contains(" %"), "{texto}");
}

/// El ancho y el alto de un PNG, de su cabecera IHDR.
fn medidas_png(ruta: &Path) -> (u32, u32) {
    let d = std::fs::read(ruta).unwrap();
    let n = |i: usize| u32::from_be_bytes(d[i..i + 4].try_into().unwrap());
    (n(16), n(20))
}

// ADR 0032: el tamaño de todas, encima del preset, como en la ventana.
#[test]
fn el_lado_mayor_y_el_porcentaje_de_todas() {
    let d = carpeta("tamano");
    let r = apolo()
        .args(["lote", "-q", "--formato", "png", "--lado-mayor", "32"])
        .arg(d.join("fotos"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    // foto.webp mide 128 × 128.
    assert_eq!(medidas_png(&d.join("fotos-png/foto.png")), (32, 32));
    let r = apolo()
        .args([
            "lote",
            "-q",
            "--formato",
            "png",
            "--porcentaje",
            "50",
            "--salida",
        ])
        .arg(d.join("mitad"))
        .arg(d.join("fotos"))
        .output()
        .unwrap();
    assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    assert_eq!(medidas_png(&d.join("mitad/foto.png")), (64, 64));
    // Agrandar, no.
    let r = apolo()
        .args(["lote", "--porcentaje", "150"])
        .arg(d.join("fotos"))
        .output()
        .unwrap();
    assert!(!r.status.success());
    assert!(String::from_utf8_lossy(&r.stderr).contains("nunca agranda"));
}
