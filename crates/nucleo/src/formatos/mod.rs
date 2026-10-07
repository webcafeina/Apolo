//! Los formatos de salida que no son WebP ni JPEG: PNG con OxiPNG y QOI
//! (ADR 0020), AVIF y JPEG XL (ADR 0021). Cada uno da el mismo fichero que su
//! herramienta oficial.

pub mod avif;
pub mod jxl;
pub mod png;
pub mod qoi;
