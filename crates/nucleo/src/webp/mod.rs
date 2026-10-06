//! WebP con libwebp: las opciones de cwebp y la codificación.

mod codificar;
pub mod opciones;

pub use codificar::{Codificado, Estadisticas, Extras, Medida, Progreso, codificar};
pub use opciones::{
    FiltradoAlfa, ModoRedimension, OpcionesWebp, Pista, Preset, Recorte, Redimension,
};
