//! Vectorizar imágenes: de un mapa de bits a un dibujo vectorial (ADR 0038 y 0039).
//!
//! El motor salió del banco de pruebas de `laboratorio/vectorizar/`, donde se midió frente a VTracer,
//! Potrace y Vectorizer.ai con la nota SSIMULACRA 2. Los pasos:
//!
//! 1. **Paleta** con los colores de los píxeles planos (`paleta`); los del borde son mezcla del suavizado.
//! 2. **Mezcla y contorno por debajo del píxel** (`subpixel`): cada píxel de borde es mezcla de dos
//!    colores, y el contorno de cada capa es la línea de nivel 0,5 de su cobertura.
//! 3. **Polígono primero** (`poli`): rectas con esquinas exactas, Bézier, círculos y elipses (`ajuste`), y
//!    los polígonos pequeños ajustados por síntesis frente al original (`sintesis`).
//! 4. **Degradados** donde el color cambia suavemente (`degradado`).
//! 5. **Pixel art** trazado a píxeles (`pixelart`).
//!
//! El resultado es un [`Documento`], del que salen el SVG (`svg`) y la vista previa (`vista`).

mod ajuste;
mod degradado;
mod geometria;
mod imagen;
mod modelo;
mod motor;
mod paleta;
mod pixelart;
mod poli;
mod sintesis;
mod subpixel;
pub mod svg;
pub mod vista;

pub use imagen::Rgba;
pub use modelo::{Capa, Documento, Forma, Relleno, Tramo};
pub use motor::{OpcionesVector, TipoImagen, tipo_efectivo, vectorizar};
