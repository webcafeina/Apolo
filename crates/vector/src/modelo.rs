//! El dibujo vectorial: capas apiladas, cada una con su relleno y sus formas. De aquí salen SVG, PDF, EPS y
//! DXF, así que los cuatro son el mismo dibujo.

use crate::geometria::P;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tramo {
    Linea(P, P),
    Cubica(P, P, P, P),
}

impl Tramo {
    pub fn ini(&self) -> P {
        match *self {
            Tramo::Linea(a, _) | Tramo::Cubica(a, ..) => a,
        }
    }
    pub fn fin(&self) -> P {
        match *self {
            Tramo::Linea(_, b) | Tramo::Cubica(.., b) => b,
        }
    }
}

/// Una figura cerrada. Dentro de una capa, las figuras se combinan par-impar: una figura dentro de otra
/// es un hueco.
#[derive(Clone, Debug, PartialEq)]
pub enum Forma {
    Circulo {
        c: P,
        r: f64,
    },
    /// Elipse de semiejes `rx`, `ry`, girada `ang` radianes.
    Elipse {
        c: P,
        rx: f64,
        ry: f64,
        ang: f64,
    },
    /// Tramos encadenados: el final de uno es el principio del siguiente, y el último cierra.
    Tramos(Vec<Tramo>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Relleno {
    Color([u8; 3]),
    /// Paradas: (posición de 0 a 1, color).
    Lineal {
        p1: P,
        p2: P,
        paradas: Vec<(f64, [u8; 3])>,
    },
    Radial {
        c: P,
        r: f64,
        paradas: Vec<(f64, [u8; 3])>,
    },
}

impl Relleno {
    /// El color medio, para los formatos sin degradados (DXF).
    pub fn color_medio(&self) -> [u8; 3] {
        match self {
            Relleno::Color(c) => *c,
            Relleno::Lineal { paradas, .. } | Relleno::Radial { paradas, .. } => {
                let n = paradas.len().max(1) as u32;
                [0, 1, 2].map(|ch| (paradas.iter().map(|p| p.1[ch] as u32).sum::<u32>() / n) as u8)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Capa {
    pub relleno: Relleno,
    pub formas: Vec<Forma>,
}

/// El resultado de vectorizar: un fondo que cubre todo y capas que se pintan encima, en orden.
#[derive(Clone, Debug, PartialEq)]
pub struct Documento {
    pub ancho: u32,
    pub alto: u32,
    pub fondo: Option<Relleno>,
    pub capas: Vec<Capa>,
}

impl Documento {
    /// Cuántas figuras, nodos (órdenes de dibujo) y colores distintos tiene.
    pub fn estadisticas(&self) -> Estadisticas {
        let mut formas = 0;
        let mut nodos = 0;
        let mut colores = std::collections::BTreeSet::new();
        if let Some(f) = &self.fondo {
            colores.insert(f.color_medio());
        }
        for c in &self.capas {
            colores.insert(c.relleno.color_medio());
            for f in &c.formas {
                formas += 1;
                nodos += match f {
                    Forma::Circulo { .. } | Forma::Elipse { .. } => 4,
                    Forma::Tramos(t) => t.len() + 1,
                };
            }
        }
        Estadisticas {
            formas,
            nodos,
            colores: colores.len(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Estadisticas {
    pub formas: usize,
    pub nodos: usize,
    pub colores: usize,
}
