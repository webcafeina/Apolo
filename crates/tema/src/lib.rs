//! La paleta de Apolo, con su contraste medido.
//!
//! Apolo sigue la apariencia del sistema (ADR 0008) con su marca (ADR 0015):
//! superficies grises neutras, tipografía del sistema, y el **oro del sol** del
//! icono como color de acción. Es el mismo mecanismo que Esfinge
//! (`internal/tema`, sus ADR 0008 y 0021), traído a Rust: la paleta se define
//! aquí, el contraste se calcula aquí, y `frontend/src/tokens.css` se **genera**
//! con `make tokens`. No se edita a mano.
//!
//! La regla del oro es la de Esfinge: **el oro rellena, la piedra escribe**.
//! Blanco sobre el oro del sol no llega ni a 2:1; piedra sobre él pasa de 9:1.
//! Así que el oro va de fondo (botón principal, fila activa, selección) con
//! tinta oscura encima, y nunca como texto ni como línea fina: para eso está
//! `acento`, que es la tinta fuerte del tema.
//!
//! Gris neutro alrededor no es solo gusto: en una herramienta de imágenes, un
//! cromo con color tiñe el juicio sobre el color de la foto. El oro se queda en
//! los controles, nunca alrededor de la imagen.

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

    // Acción. `relleno` es el oro de un botón principal o de lo seleccionado,
    // con `sobre_acento` (piedra) encima. `acento` **no es oro**: es la tinta
    // fuerte del tema, para el filete de foco, los bordes que señalan y el
    // texto que destaca, porque el oro en una línea fina no se ve.
    pub relleno: Rgb,
    pub relleno_vivo: Rgb,
    pub sobre_acento: Rgb,
    pub acento: Rgb,

    // El oro con transparencia: el halo del foco (`anillo`) y el fondo de lo
    // seleccionado sin rellenar (`relleno_tenue`). Decoración: no llevan
    // texto que dependa de ellos, así que no entran en el contraste.
    pub anillo: (Rgb, u8),
    pub relleno_tenue: (Rgb, u8),

    // Superficies con nombre propio: la barra lateral, un campo que se hunde
    // y un botón que se levanta.
    pub barra: Rgb,
    /// Una fila de la barra lateral con el puntero encima: se hunde, no se
    /// levanta, como en las aplicaciones del sistema (y en Esfinge).
    pub barra_encima: Rgb,
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

/// El oro del sol del icono (`empaquetado/icono.svg`, el centro del degradado
/// «brillo»). Más cálido que el de Esfinge (`#f2c14e`), para que los dos
/// hermanos no se confundan.
const ORO: Rgb = Rgb::hex(0xffc83d);
/// La placa del icono, la misma que la de Esfinge. Es la tinta sobre el oro.
const PIEDRA: Rgb = Rgb::hex(0x2b2b31);
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
    // Si algún día el oro cambiara a uno que no aguanta la piedra encima, se
    // oscurece lo justo (y la prueba de contraste lo vigila).
    let relleno = contraste::relleno_legible(ORO, PIEDRA, AA_NORMAL);

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
        relleno_vivo: relleno.oscurecer(0.92),
        sobre_acento: PIEDRA,
        acento: PIEDRA,
        anillo: (ORO, 35),
        relleno_tenue: (ORO, 18),
        barra: Rgb::hex(0xf6f6f8),
        barra_encima: peor,
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
    let relleno = contraste::relleno_legible(ORO, PIEDRA, AA_NORMAL);

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
        relleno_vivo: relleno.oscurecer(0.92),
        sobre_acento: PIEDRA,
        acento: BLANCO,
        anillo: (ORO, 45),
        relleno_tenue: (ORO, 22),
        barra: Rgb::hex(0x242426),
        barra_encima: Rgb::hex(0x343437),
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

// Los colores de estado de Windows 11 (los System*Foreground de WinUI).
const VERDE_WINDOWS: Rgb = Rgb::hex(0x0f7b0f);
const AMBAR_WINDOWS: Rgb = Rgb::hex(0x9d5d00);
const ROJO_WINDOWS: Rgb = Rgb::hex(0xc42b1c);

/// El modo claro en Windows (ADR 0027): los neutros de WinUI 3 en vez de los
/// de macOS, aplanados sobre el gris de Mica (`SolidBackgroundFillColorBase`).
/// El oro, la piedra y el acento son los mismos: la marca no cambia de sistema.
pub fn windows_claro() -> Tema {
    let lienzo = Rgb::hex(0xf9f9f9);
    let tarjeta = Rgb::hex(0xffffff);
    let peor = Rgb::hex(0xebebeb);
    let relleno = contraste::relleno_legible(ORO, PIEDRA, AA_NORMAL);

    Tema {
        nombre: "windows-claro",
        oscuro: false,
        lienzo,
        suave: Rgb::hex(0xf3f3f3),
        tarjeta,
        elevada: peor,
        filete: Rgb::hex(0xe5e5e5),
        filete_fuerte: contraste::acento_legible(Rgb::hex(0x8a8a8a), peor, AA_GRANDE),
        tinta: Rgb::hex(0x1b1b1b),
        cuerpo: Rgb::hex(0x3d3d3d),
        apagado: contraste::acento_legible(Rgb::hex(0x616161), peor, AA_NORMAL),
        relleno,
        relleno_vivo: relleno.oscurecer(0.92),
        sobre_acento: PIEDRA,
        acento: PIEDRA,
        anillo: (ORO, 35),
        relleno_tenue: (ORO, 18),
        barra: Rgb::hex(0xf3f3f3),
        barra_encima: Rgb::hex(0xeaeaea),
        campo: tarjeta,
        boton: Rgb::hex(0xfbfbfb),
        boton_encima: Rgb::hex(0xf6f6f6),
        exito: contraste::acento_legible(VERDE_WINDOWS, peor, AA_NORMAL),
        aviso: contraste::acento_legible(AMBAR_WINDOWS, peor, AA_NORMAL),
        error: contraste::acento_legible(ROJO_WINDOWS, peor, AA_NORMAL),
        damero_a: Rgb::hex(0xffffff),
        damero_b: Rgb::hex(0xe6e6e6),
    }
}

/// El modo oscuro en Windows (ADR 0027), con los neutros de WinUI 3.
pub fn windows_oscuro() -> Tema {
    let elevada = Rgb::hex(0x383838);
    // Los botones con el puntero encima son la superficie más clara.
    let peor = Rgb::hex(0x3a3a3a);
    let relleno = contraste::relleno_legible(ORO, PIEDRA, AA_NORMAL);

    Tema {
        nombre: "windows-oscuro",
        oscuro: true,
        lienzo: Rgb::hex(0x272727),
        suave: Rgb::hex(0x1c1c1c),
        tarjeta: Rgb::hex(0x2b2b2b),
        elevada,
        filete: Rgb::hex(0x3a3a3a),
        filete_fuerte: contraste::acento_legible(Rgb::hex(0x9a9a9a), peor, AA_GRANDE),
        tinta: Rgb::hex(0xffffff),
        cuerpo: Rgb::hex(0xe4e4e4),
        apagado: contraste::acento_legible(Rgb::hex(0x9d9d9d), peor, AA_NORMAL),
        relleno,
        relleno_vivo: relleno.oscurecer(0.92),
        sobre_acento: PIEDRA,
        acento: BLANCO,
        anillo: (ORO, 45),
        relleno_tenue: (ORO, 22),
        barra: Rgb::hex(0x202020),
        barra_encima: Rgb::hex(0x2d2d2d),
        campo: Rgb::hex(0x1e1e1e),
        boton: Rgb::hex(0x2d2d2d),
        boton_encima: Rgb::hex(0x323232),
        exito: contraste::acento_legible(Rgb::hex(0x6ccb5f), peor, AA_NORMAL),
        aviso: contraste::acento_legible(Rgb::hex(0xfce100), peor, AA_NORMAL),
        error: contraste::acento_legible(Rgb::hex(0xff99a4), peor, AA_NORMAL),
        damero_a: Rgb::hex(0x2b2b2b),
        damero_b: Rgb::hex(0x383838),
    }
}

/// Todas las paletas: las dos de siempre y las dos de Windows. La prueba de
/// contraste las mide todas.
pub fn temas() -> [Tema; 4] {
    [claro(), oscuro(), windows_claro(), windows_oscuro()]
}
