//! `apolo vectorizar` de punta a punta, con el binario de verdad (ADR 0039, docs/cobertura-vectorizar.md).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// El binario, en español (ver tests/lote.rs).
fn apolo() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_apolo"));
    c.env("LC_ALL", "es_ES.UTF-8");
    c
}

/// Un logotipo: un cuadrado azul y un círculo rojo sobre blanco, suavizado.
fn logo(ruta: &Path) {
    let lado = 96u32;
    let mut px = Vec::new();
    for y in 0..lado {
        for x in 0..lado {
            let mut s = [0u32; 3];
            for sy in 0..4 {
                for sx in 0..4 {
                    let (fx, fy) = (
                        x as f64 + (sx as f64 + 0.5) / 4.0,
                        y as f64 + (sy as f64 + 0.5) / 4.0,
                    );
                    let c = if ((fx - 64.0).powi(2) + (fy - 60.0).powi(2)).sqrt() < 20.0 {
                        [220, 40, 40]
                    } else if (12.0..44.0).contains(&fx) && (14.0..46.0).contains(&fy) {
                        [30, 70, 200]
                    } else {
                        [255, 255, 255]
                    };
                    for k in 0..3 {
                        s[k] += c[k];
                    }
                }
            }
            px.extend([(s[0] / 16) as u8, (s[1] / 16) as u8, (s[2] / 16) as u8, 255]);
        }
    }
    let f = std::fs::File::create(ruta).unwrap();
    let mut e = png::Encoder::new(f, lado, lado);
    e.set_color(png::ColorType::Rgba);
    e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(&px).unwrap();
}

fn carpeta(nombre: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("apolo-cli-vector-{nombre}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("iconos/sub")).unwrap();
    logo(&d.join("logo.png"));
    logo(&d.join("iconos/a.png"));
    logo(&d.join("iconos/sub/b.png"));
    d
}

fn bien(r: &Output) -> String {
    assert!(
        r.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&r.stdout),
        String::from_utf8_lossy(&r.stderr)
    );
    String::from_utf8_lossy(&r.stdout).into_owned()
}

fn empieza(ruta: &Path, cabecera: &[u8]) {
    let b = std::fs::read(ruta).unwrap_or_else(|e| panic!("{}: {e}", ruta.display()));
    assert!(
        b.starts_with(cabecera),
        "{}: {:?}",
        ruta.display(),
        String::from_utf8_lossy(&b[..b.len().min(40)])
    );
}

#[test]
fn una_imagen_al_lado_sin_pisar() {
    let d = carpeta("lado");
    let s = bien(
        &apolo()
            .arg("vectorizar")
            .arg(d.join("logo.png"))
            .output()
            .unwrap(),
    );
    assert!(s.contains("logo.png → ") && s.contains("nota "), "{s}");
    empieza(&d.join("logo.svg"), b"<svg");
    // Otra vez: no pisa.
    bien(
        &apolo()
            .args(["vectorizar", "-q"])
            .arg(d.join("logo.png"))
            .output()
            .unwrap(),
    );
    empieza(&d.join("logo-2.svg"), b"<svg");
}

#[test]
fn el_formato_sale_de_la_extension() {
    let d = carpeta("ext");
    for (nombre, cabecera) in [
        ("a.svg", &b"<svg"[..]),
        ("a.pdf", b"%PDF-1.4"),
        ("a.eps", b"%!PS-Adobe-3.0 EPSF-3.0"),
        ("a.dxf", b"  0\nSECTION"),
    ] {
        bien(
            &apolo()
                .args(["vectorizar", "-q"])
                .arg(d.join("logo.png"))
                .arg("-o")
                .arg(d.join(nombre))
                .output()
                .unwrap(),
        );
        empieza(&d.join(nombre), cabecera);
    }
    // -o escribe aunque exista (se ha pedido ese fichero).
    bien(
        &apolo()
            .args(["vectorizar", "-q"])
            .arg(d.join("logo.png"))
            .arg("-o")
            .arg(d.join("a.svg"))
            .output()
            .unwrap(),
    );
    assert!(!d.join("a-2.svg").exists());
    // --formato manda sobre la carpeta.
    bien(
        &apolo()
            .args(["vectorizar", "-q", "--formato", "eps"])
            .arg(d.join("logo.png"))
            .arg("-o")
            .arg(d.join("sal"))
            .output()
            .unwrap(),
    );
    empieza(&d.join("sal/logo.eps"), b"%!PS");
}

#[test]
fn carpetas_con_subcarpetas() {
    let d = carpeta("carpetas");
    let s = bien(
        &apolo()
            .args([
                "vectorizar",
                "--formato",
                "dxf",
                "--dxf",
                "rectas",
                "--estilo",
                "trazar-bordes",
            ])
            .arg(d.join("iconos"))
            .output()
            .unwrap(),
    );
    assert!(s.contains("2 vectorizadas, 0 con error"), "{s}");
    // Junto a la carpeta, con el formato de sufijo, como los lotes.
    let txt = std::fs::read_to_string(d.join("iconos-dxf/sub/b.dxf")).unwrap();
    assert!(txt.contains("LWPOLYLINE") && !txt.contains("SPLINE") && !txt.contains("CIRCLE"));
    assert!(d.join("iconos-dxf/a.dxf").exists());
}

#[test]
fn opciones_de_dibujo() {
    let d = carpeta("opciones");
    let svg = |args: &[&str]| -> String {
        let o = d.join("x.svg");
        bien(
            &apolo()
                .args(["vectorizar", "-q"])
                .arg(d.join("logo.png"))
                .arg("-o")
                .arg(&o)
                .args(args)
                .output()
                .unwrap(),
        );
        std::fs::read_to_string(o).unwrap()
    };
    let colores = |s: &str| s.matches("fill=\"#").count();
    assert_eq!(
        colores(&svg(&["--colores", "2"])),
        2,
        "dos colores: fondo y una capa"
    );
    let p = svg(&["--paleta", "#000000,#ffffff"]);
    assert!(p.contains("#000000") && !p.contains("#1e46c8"), "{p}");
    let t = svg(&[
        "--estilo",
        "trazar-formas",
        "--grosor",
        "2.5",
        "--color-trazo",
        "#00ff00",
    ]);
    assert!(
        t.contains(r##"stroke="#00ff00""##) && t.contains(r#"stroke-width="2.5""#),
        "{t}"
    );
    let pixel = svg(&["--tipo", "pixel-art"]);
    assert!(!pixel.contains('C'), "pixel art: sin curvas");
    assert!(svg(&["--detalle", "0"]).len() <= svg(&["--detalle", "100"]).len());
    // Sin degradados ni área mínima rotos: solo que corran.
    svg(&["--sin-degradados", "--area-minima", "20"]);
}

#[test]
fn por_nota() {
    let d = carpeta("nota");
    let s = bien(
        &apolo()
            .args(["vectorizar", "--objetivo", "60", "--detalle", "0"])
            .arg(d.join("logo.png"))
            .arg("-o")
            .arg(d.join("n.svg"))
            .output()
            .unwrap(),
    );
    assert!(s.contains("objetivo 60: detalle "), "{s}");
}

#[test]
fn errores() {
    let d = carpeta("errores");
    for args in [
        vec!["--colores", "1"],
        vec!["--paleta", "#12"],
        vec!["--color-trazo", "rojo"],
        vec!["--grosor", "0"],
        vec!["--objetivo", "120"],
    ] {
        let r = apolo()
            .arg("vectorizar")
            .arg(d.join("logo.png"))
            .args(&args)
            .output()
            .unwrap();
        assert_eq!(r.status.code(), Some(2), "{args:?}");
        assert!(
            String::from_utf8_lossy(&r.stderr).starts_with("Error: "),
            "{args:?}"
        );
    }
    // Una que no es imagen falla sola; la orden sale con 1.
    std::fs::write(d.join("iconos/roto.png"), b"no soy un png").unwrap();
    let r = apolo()
        .args(["vectorizar", "-q"])
        .arg(d.join("iconos"))
        .output()
        .unwrap();
    assert_eq!(r.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&r.stderr).contains("roto.png"));
    assert!(d.join("iconos-svg/a.svg").exists());
}

#[test]
fn ayuda_en_ingles() {
    let r = Command::new(env!("CARGO_BIN_EXE_apolo"))
        .args(["--idioma", "en", "vectorizar", "--help"])
        .output()
        .unwrap();
    let s = String::from_utf8_lossy(&r.stdout);
    assert!(
        s.contains("Vectorizes: from a bitmap to SVG, PDF, EPS or DXF"),
        "{s}"
    );
}
