//! La paleta de Apolo, con su contraste medido.
//!
//! Apolo sigue la apariencia del sistema (ADR 0008): superficies grises
//! neutras, tipografía del sistema, y el azul del sistema como color de acción.
//! Es el mismo mecanismo que Esfinge (`internal/tema`, su ADR 0008), traído a
//! Rust: la paleta se define aquí, el contraste se calcula aquí, y
//! `frontend/src/tokens.css` se **genera** con `make tokens`. No se edita a mano.
//!
//! Gris neutro no es solo gusto: en una herramienta de imágenes, un cromo con
//! color tiñe el juicio sobre el color de la foto.

pub mod contraste;
pub mod tokens;

pub use contraste::{AA_GRANDE, AA_NORMAL, Rgb, contraste};

/// La paleta resuelta para un modo de apariencia.
#[derive(Debug, Clone)]
pub struct Tema {
    pub nombre: &'static str,
    pub oscuro: bool,

    // Superficies, de la más al fondo a la más elevada.
    pub lienzo: Rgb,
    pub suave: Rgb,
    pub tarjeta: Rgb,
    pub elevada: Rgb,

    // Separaciones. `filete_fuerte` es el borde de un control, y como tal
    // tiene que llegar a 3:1 contra el fondo (WCAG 1.4.11).
    pub filete: Rgb,
    pub filete_fuerte: Rgb,

    // Texto, en tres pesos de presencia.
    pub tinta: Rgb,
    pub cuerpo: Rgb,
    pub apagado: Rgb,

    // Acción. `relleno` es el fondo de un botón principal, con `sobre_acento`
    // encima; `acento` es el mismo azul cuando hace de texto (un enlace, un
    // valor seleccionado), y por eso se ajusta contra el fondo y no contra el
    // blanco.
    pub relleno: Rgb,
    pub relleno_vivo: Rgb,
    pub sobre_acento: Rgb,
    pub acento: Rgb,

    // Superficies con nombre propio: la barra lateral, un campo que se hunde
    // y un botón que se levanta.
    pub barra: Rgb,
    pub campo: Rgb,
    pub boton: Rgb,
    pub boton_encima: Rgb,

    // Estados, usados como texto.
    pub exito: Rgb,
    pub aviso: Rgb,
    pub error: Rgb,

    // El damero que se ve detrás de una imagen con transparencia. No lleva
    // texto encima: no entra en el contraste.
    pub damero_a: Rgb,
    pub damero_b: Rgb,
}

const AZUL_CLARO: Rgb = Rgb::hex(0x007aff); // el de macOS
const AZUL_OSCURO: Rgb = Rgb::hex(0x0a84ff); // su variante para modo oscuro
const BLANCO: Rgb = Rgb::hex(0xffffff);
const VERDE: Rgb = Rgb::hex(0x34c759);
const AMBAR: Rgb = Rgb::hex(0xff9500);
const ROJO: Rgb = Rgb::hex(0xff3b30);

/// El modo claro.
pub fn claro() -> Tema {
    let lienzo = Rgb::hex(0xffffff);
    let tarjeta = Rgb::hex(0xf2f2f7);
    // Todo texto tiene que leerse en todas las superficies, así que se ajusta
    // contra la peor: en claro, la más oscura.
    let peor = Rgb::hex(0xe8e8ed);
    // El azul del sistema con blanco encima se queda en 4,02:1. Se oscurece
    // lo justo para llegar a AA, que es lo que hace Esfinge con su oro.
    let relleno = contraste::relleno_legible(AZUL_CLARO, BLANCO, AA_NORMAL);

    Tema {
        nombre: "claro",
        oscuro: false,
        lienzo,
        suave: Rgb::hex(0xf7f7f9),
        tarjeta,
        elevada: peor,
        filete: Rgb::hex(0xd8d8de),
        filete_fuerte: contraste::acento_legible(Rgb::hex(0x8e8e93), tarjeta, AA_GRANDE),
        tinta: Rgb::hex(0x1c1c1e),
        cuerpo: Rgb::hex(0x3c3c43),
        apagado: contraste::acento_legible(Rgb::hex(0x6c6c72), peor, AA_NORMAL),
        relleno,
        relleno_vivo: relleno.oscurecer(0.88),
        sobre_acento: BLANCO,
        acento: contraste::acento_legible(AZUL_CLARO, peor, AA_NORMAL),
        barra: Rgb::hex(0xf6f6f8),
        campo: lienzo,
        boton: lienzo,
        boton_encima: Rgb::hex(0xf2f2f5),
        exito: contraste::acento_legible(VERDE, peor, AA_NORMAL),
        aviso: contraste::acento_legible(AMBAR, peor, AA_NORMAL),
        error: contraste::acento_legible(ROJO, peor, AA_NORMAL),
        damero_a: Rgb::hex(0xffffff),
        damero_b: Rgb::hex(0xe6e6ea),
    }
}

/// El modo oscuro.
pub fn oscuro() -> Tema {
    let lienzo = Rgb::hex(0x1e1e1e);
    let elevada = Rgb::hex(0x3a3a3c);
    // En oscuro la peor superficie para el texto es la más clara.
    let peor = Rgb::hex(0x48484a);
    let relleno = contraste::relleno_legible(AZUL_OSCURO, BLANCO, AA_NORMAL);

    Tema {
        nombre: "oscuro",
        oscuro: true,
        lienzo,
        suave: Rgb::hex(0x252527),
        tarjeta: Rgb::hex(0x2c2c2e),
        elevada,
        filete: Rgb::hex(0x3f3f42),
        filete_fuerte: Rgb::hex(0x7a7a80),
        tinta: Rgb::hex(0xffffff),
        cuerpo: Rgb::hex(0xe3e3e6),
        apagado: contraste::acento_legible(Rgb::hex(0xa1a1a8), peor, AA_NORMAL),
        relleno,
        relleno_vivo: relleno.oscurecer(0.88),
        sobre_acento: BLANCO,
        acento: contraste::acento_legible(AZUL_OSCURO, peor, AA_NORMAL),
        barra: Rgb::hex(0x242426),
        campo: Rgb::hex(0x1a1a1c),
        boton: elevada,
        boton_encima: peor,
        exito: contraste::acento_legible(VERDE, peor, AA_NORMAL),
        aviso: contraste::acento_legible(AMBAR, peor, AA_NORMAL),
        error: contraste::acento_legible(ROJO, peor, AA_NORMAL),
        damero_a: Rgb::hex(0x2c2c2e),
        damero_b: Rgb::hex(0x3a3a3c),
    }
}

/// Los dos modos, en el orden en que se escriben en el CSS.
pub fn temas() -> [Tema; 2] {
    [claro(), oscuro()]
}
