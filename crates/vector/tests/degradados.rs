//! Degradados suaves que la paleta parte en varios colores (el fondo del icono de Apolo, 2026-11-10): cada trozo
//! parece plano, pero juntos son un degradado y tienen que salir como tal, no en franjas. Y lo contrario: dos
//! colores distintos con un borde nítido no se funden en un degradado.

use apolo_vector::{OpcionesVector, Relleno, Rgba, vectorizar, vista};

fn imagen(lado: u32, color: impl Fn(f64, f64) -> [u8; 3]) -> Rgba {
    let mut px = Vec::new();
    for y in 0..lado {
        for x in 0..lado {
            let mut s = [0u32; 3];
            for sy in 0..4 {
                for sx in 0..4 {
                    let c = color(
                        x as f64 + (sx as f64 + 0.5) / 4.0,
                        y as f64 + (sy as f64 + 0.5) / 4.0,
                    );
                    for k in 0..3 {
                        s[k] += c[k] as u32;
                    }
                }
            }
            px.extend([(s[0] / 16) as u8, (s[1] / 16) as u8, (s[2] / 16) as u8, 255]);
        }
    }
    Rgba::nueva(lado, lado, px)
}

#[test]
fn un_fondo_suave_partido_en_dos_sale_como_degradado() {
    // De #47474f arriba a #2e2e34 abajo (el fondo del icono), con un círculo blanco encima.
    let im = imagen(256, |x, y| {
        if ((x - 128.0).powi(2) + (y - 128.0).powi(2)).sqrt() < 50.0 {
            return [255, 255, 255];
        }
        let t = y / 256.0;
        let m = |a: f64, b: f64| (a + (b - a) * t).round() as u8;
        [m(71.0, 46.0), m(71.0, 46.0), m(79.0, 52.0)]
    });
    let doc = vectorizar(&im, &OpcionesVector::default()).unwrap();
    assert!(
        matches!(doc.fondo, Some(Relleno::Lineal { .. })),
        "el fondo, un degradado: {:?}",
        doc.fondo
    );
    // Sin franja: arriba y abajo del todo, el color del original (a menos de 4 niveles).
    let v = vista::rasterizar(&doc).unwrap();
    for y in [3usize, 128, 252] {
        let i = (y * 256 + 8) * 4;
        let o = &im.px[i..i + 3];
        let d = (0..3)
            .map(|k| (v[i + k] as i32 - o[k] as i32).abs())
            .max()
            .unwrap();
        assert!(d <= 4, "fila {y}: {:?} frente a {:?}", &v[i..i + 3], o);
    }
}

#[test]
fn dos_colores_con_borde_nitido_no_se_funden() {
    // Un cuadrado azul y un círculo rojo sobre blanco: tres colores planos, ningún degradado.
    let im = imagen(128, |x, y| {
        if ((x - 88.0).powi(2) + (y - 80.0).powi(2)).sqrt() < 26.0 {
            [220, 40, 40]
        } else if (16.0..60.0).contains(&x) && (18.0..62.0).contains(&y) {
            [30, 70, 200]
        } else {
            [255, 255, 255]
        }
    });
    let doc = vectorizar(&im, &OpcionesVector::default()).unwrap();
    assert!(matches!(doc.fondo, Some(Relleno::Color(_))));
    assert!(
        doc.capas
            .iter()
            .all(|c| matches!(c.relleno, Relleno::Color(_))),
        "sin degradados"
    );
}
