//! El núcleo de Apolo: todo el trabajo con imágenes, sin interfaz.
//!
//! La ventana (`src-tauri`) y la línea de comandos (`crates/cli`) son dos caras
//! sobre este mismo código. Lo que hace una lo hace la otra.

pub mod cwebp;
pub mod entrada;
pub mod error;
pub mod formatos;
pub mod jpeg;
pub mod lote;
pub mod metadatos;
pub mod motores;
pub mod orientacion;
pub mod presets;
pub mod proceso;
pub mod salida;
pub mod vista;
pub mod webp;

pub use error::{Error, Resultado};
pub use motores::{Motor, motores};
