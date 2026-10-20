//! La nota SSIMULACRA 2 de Apolo se calcula canal a canal, con menos memoria
//! (c/medir.cc de apolo-avifjxl, entrega 8b). Tiene que ser **la misma, bit a
//! bit**, que la de la función de libjxl tal cual: con y sin transparencia, y
//! con tamaños impares o que se quedan en pocas escalas.

use apolo_avifjxl::{ssimulacra2, ssimulacra2_libjxl};
use apolo_nucleo::entrada::{self, Lectura};
use apolo_nucleo::salida::{self, Ajuste, FormatoSalida};
use apolo_nucleo::vista;

mod comun;
use comun::corpus;

fn rgb(rgba: &[u8]) -> Vec<u8> {
    rgba.as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [p[0], p[1], p[2]])
        .collect()
}

/// Las dos notas, iguales bit a bit, en RGB y en RGBA.
fn iguales(a: &[u8], b: &[u8], w: u32, h: u32) -> usize {
    let mut n = 0;
    for (x, y, canales) in [(rgb(a), rgb(b), 3), (a.to_vec(), b.to_vec(), 4)] {
        let ligera = ssimulacra2(&x, &y, w, h, canales);
        let libjxl = ssimulacra2_libjxl(&x, &y, w, h, canales);
        assert_eq!(
            ligera.map(f64::to_bits),
            libjxl.map(f64::to_bits),
            "{w}×{h}, {canales} canales: {ligera:?} frente a {libjxl:?}"
        );
        n += 1;
    }
    n
}

#[test]
fn la_misma_nota_que_libjxl_con_el_corpus() {
    let mut comparadas = 0;
    for caso in corpus::generar() {
        let Ok(img) = entrada::leer(&caso.datos, Lectura::default()) else {
            continue;
        };
        let Ok((w, h, referencia)) = vista::rgba(&img) else {
            continue;
        };
        if w < 8 || h < 8 {
            continue;
        }
        let a = Ajuste {
            formato: FormatoSalida::Webp,
            ..Default::default()
        };
        let c = salida::codificar(&caso.datos, &img, &a, None).unwrap();
        let (_, _, distorsionada) = vista::decodificar(&c.datos).unwrap();
        comparadas += iguales(&referencia, &distorsionada, w, h);
    }
    assert!(comparadas > 20, "solo {comparadas} comparaciones");
}

#[test]
fn la_misma_nota_que_libjxl_con_tamanos_raros() {
    // Un patrón con bordes, degradados y transparencia variable, y su versión
    // con ruido: tamaños que acaban la búsqueda en escalas distintas.
    for (w, h) in [(8, 8), (9, 13), (15, 16), (37, 101), (129, 64), (300, 7)] {
        let mut a = Vec::with_capacity((w * h * 4) as usize);
        let mut b = Vec::with_capacity((w * h * 4) as usize);
        let mut semilla = 12345u32;
        for y in 0..h {
            for x in 0..w {
                let p = [
                    (x * 255 / w.max(1)) as u8,
                    if (x / 3 + y / 5) % 2 == 0 { 30 } else { 220 },
                    (y * 255 / h.max(1)) as u8,
                    ((x + y) * 7 % 256) as u8,
                ];
                a.extend_from_slice(&p);
                for v in p {
                    semilla = semilla.wrapping_mul(1_103_515_245).wrapping_add(12345);
                    let ruido = (semilla >> 16) % 21;
                    b.push(v.saturating_add(ruido as u8).saturating_sub(10));
                }
            }
        }
        if w < 8 || h < 8 {
            assert_eq!(ssimulacra2(&a, &b, w, h, 4), None);
            continue;
        }
        iguales(&a, &b, w, h);
    }
}
