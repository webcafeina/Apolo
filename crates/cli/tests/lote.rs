//! `apolo lote` de punta a punta, con el binario de verdad.

use std::path::{Path, PathBuf};
use std::process::Command;

fn apolo() -> Command {
    Command::new(env!("CARGO_BIN_EXE_apolo"))
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
