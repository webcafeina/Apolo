//! El núcleo de Apolo: todo el trabajo con imágenes, sin interfaz.
//!
//! La ventana (`src-tauri`) y la línea de comandos (`crates/cli`) son dos caras
//! sobre este mismo código. Lo que hace una lo hace la otra.

pub mod motores;

pub use motores::{Motor, motores};
