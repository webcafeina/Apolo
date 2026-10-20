//! La memoria del proceso en Linux (entrega 8b).
//!
//! glibc da a cada hilo su propia reserva (arena) y no devuelve al sistema lo
//! que se libera en ella. Con fotos grandes en varios hilos, lo liberado se
//! queda: un lote de 48 fotos de 24 MP a AVIF con 8 hilos llegaba a 2,9 GB, y
//! con dos reservas como mucho, a 1,6 GB, y además más rápido (190 s frente a
//! 235). macOS y Windows sirven los bloques grandes aparte y los devuelven al
//! liberarlos: allí no hace falta (docs/rendimiento.md).

/// Se llama lo primero en `main`, antes de crear hilos: dos reservas como
/// mucho. En los demás sistemas no hace nada. Si `MALLOC_ARENA_MAX` está
/// puesta, manda ella.
pub fn preparar() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        const M_ARENA_MAX: std::ffi::c_int = -8;
        unsafe extern "C" {
            fn mallopt(parametro: std::ffi::c_int, valor: std::ffi::c_int) -> std::ffi::c_int;
        }
        if std::env::var_os("MALLOC_ARENA_MAX").is_none() {
            // SAFETY: mallopt de glibc, con un parámetro que existe; se llama
            // al arrancar, antes de que haya más hilos.
            unsafe {
                mallopt(M_ARENA_MAX, 2);
            }
        }
    }
}
