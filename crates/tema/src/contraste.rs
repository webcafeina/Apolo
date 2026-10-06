//! Contraste WCAG 2.x, y los ajustes que hacen legible un color.
//!
//! Es el cálculo de Esfinge (`internal/tema/contraste.go`) traído a Rust.

use std::fmt;

/// Texto normal: 4,5:1.
pub const AA_NORMAL: f64 = 4.5;
/// Texto grande y componentes de la interfaz: 3:1.
pub const AA_GRANDE: f64 = 3.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn hex(v: u32) -> Self {
        Rgb {
            r: ((v >> 16) & 0xff) as u8,
            g: ((v >> 8) & 0xff) as u8,
            b: (v & 0xff) as u8,
        }
    }

    /// Multiplica cada canal por `factor` (0–1).
    pub fn oscurecer(self, factor: f64) -> Self {
        let f = |c: u8| (c as f64 * factor).round().clamp(0.0, 255.0) as u8;
        Rgb {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }

    /// Acerca cada canal al blanco en la proporción `t` (0–1).
    pub fn aclarar(self, t: f64) -> Self {
        let f = |c: u8| {
            (c as f64 + (255.0 - c as f64) * t)
                .round()
                .clamp(0.0, 255.0) as u8
        };
        Rgb {
            r: f(self.r),
            g: f(self.g),
            b: f(self.b),
        }
    }

    pub fn luminancia(self) -> f64 {
        fn lineal(v: u8) -> f64 {
            let f = v as f64 / 255.0;
            if f <= 0.04045 {
                f / 12.92
            } else {
                ((f + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * lineal(self.r) + 0.7152 * lineal(self.g) + 0.0722 * lineal(self.b)
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

/// La razón de contraste entre dos colores, de 1 a 21.
pub fn contraste(a: Rgb, b: Rgb) -> f64 {
    let (la, lb) = (a.luminancia(), b.luminancia());
    let (alto, bajo) = if la >= lb { (la, lb) } else { (lb, la) };
    (alto + 0.05) / (bajo + 0.05)
}

/// `acento` usado como texto sobre `fondo`: se oscurece (fondo claro) o se
/// aclara (fondo oscuro) lo justo para llegar a `minimo`.
pub fn acento_legible(acento: Rgb, fondo: Rgb, minimo: f64) -> Rgb {
    let fondo_claro = fondo.luminancia() > 0.18;
    let mut c = acento;
    for paso in 0..=100 {
        if contraste(c, fondo) >= minimo {
            return c;
        }
        let t = paso as f64 / 100.0;
        c = if fondo_claro {
            acento.oscurecer(1.0 - t)
        } else {
            acento.aclarar(t)
        };
    }
    c
}

/// `relleno` como fondo de un botón con `tinta` encima: se ajusta el relleno,
/// no la tinta, lo justo para llegar a `minimo`.
pub fn relleno_legible(relleno: Rgb, tinta: Rgb, minimo: f64) -> Rgb {
    acento_legible(relleno, tinta, minimo)
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn negro_sobre_blanco_es_21() {
        let c = contraste(Rgb::hex(0x000000), Rgb::hex(0xffffff));
        assert!((c - 21.0).abs() < 1e-9);
    }

    #[test]
    fn el_azul_del_sistema_no_llega_con_blanco() {
        // Es la razón de que exista relleno_legible.
        let c = contraste(Rgb::hex(0x007aff), Rgb::hex(0xffffff));
        assert!(c < AA_NORMAL, "{c}");
    }

    #[test]
    fn relleno_legible_llega_y_no_se_pasa() {
        let blanco = Rgb::hex(0xffffff);
        let r = relleno_legible(Rgb::hex(0x007aff), blanco, AA_NORMAL);
        let c = contraste(r, blanco);
        assert!(c >= AA_NORMAL, "{r} da {c}");
        assert!(c < AA_NORMAL + 0.3, "{r} se oscureció de más: {c}");
    }
}
