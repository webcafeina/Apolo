//! La puerta del contraste: si una pareja de colores que se usa junta no llega
//! a AA, las pruebas fallan. Que el sistema de origen cumpla no dice nada de la
//! combinación resultante.

use apolo_tema::{AA_GRANDE, AA_NORMAL, Rgb, Tema, contraste, temas};
use std::path::Path;

fn parejas(t: &Tema) -> Vec<(&'static str, Rgb, &'static str, Rgb, f64)> {
    let mut v = Vec::new();
    let fondos = [
        ("lienzo", t.lienzo),
        ("suave", t.suave),
        ("tarjeta", t.tarjeta),
        ("elevada", t.elevada),
        ("barra", t.barra),
        ("campo", t.campo),
        ("boton", t.boton),
        ("boton-encima", t.boton_encima),
    ];
    for (nf, f) in fondos {
        v.push(("tinta", t.tinta, nf, f, AA_NORMAL));
        v.push(("cuerpo", t.cuerpo, nf, f, AA_NORMAL));
        v.push(("apagado", t.apagado, nf, f, AA_NORMAL));
        v.push(("acento", t.acento, nf, f, AA_NORMAL));
        v.push(("exito", t.exito, nf, f, AA_NORMAL));
        v.push(("aviso", t.aviso, nf, f, AA_NORMAL));
        v.push(("error", t.error, nf, f, AA_NORMAL));
    }
    v.push((
        "sobre-acento",
        t.sobre_acento,
        "relleno",
        t.relleno,
        AA_NORMAL,
    ));
    v.push((
        "sobre-acento",
        t.sobre_acento,
        "relleno-vivo",
        t.relleno_vivo,
        AA_NORMAL,
    ));
    // El borde de un campo o un botón es un componente: 3:1 contra lo que lo
    // rodea. El oro no está aquí a propósito, igual que en Esfinge (su ADR
    // 0021): un botón dorado se identifica por su texto, que sí llega a AA
    // sobre él, y el foco lo marca `acento`, que sí llega a 3:1 en todas las
    // superficies (abajo).
    for (nf, f) in [
        ("lienzo", t.lienzo),
        ("tarjeta", t.tarjeta),
        ("barra", t.barra),
    ] {
        v.push(("filete-fuerte", t.filete_fuerte, nf, f, AA_GRANDE));
    }
    v
}

#[test]
fn todas_las_parejas_llegan_a_aa() {
    let mut fallos = Vec::new();
    for t in temas() {
        for (nt, ct, nf, cf, minimo) in parejas(&t) {
            let c = contraste(ct, cf);
            if c < minimo {
                fallos.push(format!(
                    "{}: {nt} {ct} sobre {nf} {cf} da {c:.2}:1, y hace falta {minimo}:1",
                    t.nombre
                ));
            }
        }
    }
    assert!(fallos.is_empty(), "\n{}", fallos.join("\n"));
}

#[test]
fn tokens_css_esta_al_dia() {
    let ruta = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(apolo_tema::tokens::RUTA);
    let en_disco = std::fs::read_to_string(&ruta).unwrap_or_default();
    assert!(
        en_disco == apolo_tema::tokens::css(),
        "{} no coincide con la paleta: corre `make tokens` (y no lo edites a mano)",
        apolo_tema::tokens::RUTA
    );
}

#[test]
fn blanco_sobre_el_oro_no_se_lee() {
    // La razón de que la tinta sobre el oro sea piedra y no blanco. Si un día
    // el oro cambia y esto deja de cumplirse, hay que revisar la regla, no
    // solo el color.
    for t in temas() {
        let c = contraste(Rgb::hex(0xffffff), t.relleno);
        assert!(
            c < AA_GRANDE,
            "{}: blanco sobre el oro da {c:.2}:1",
            t.nombre
        );
    }
}
