//! La ventana en macOS (ADR 0028): Liquid Glass en macOS 26 y 27, y el
//! material de barra lateral de siempre en los anteriores.
//!
//! La ventana sale de `tauri.macos.conf.json`, ya transparente y con la barra
//! de título superpuesta, con los semáforos colocados para la barra de
//! herramientas unificada de 52 px (`trafficLightPosition`). Aquí solo se le
//! pone el vidrio, y se pone a mano y no con `windowEffects` para saber si se
//! pudo: `window_vibrancy` devuelve el error, Tauri se lo traga. Si no hay
//! vidrio, la interfaz pinta el fondo opaco (`data-vidrio="no"`).
//!
//! El vidrio es el `NSGlassEffectView` de Apple con el estilo `Regular`, el
//! público (los demás estilos del crate son API privada). Va detrás de toda la
//! ventana; la interfaz solo lo deja ver en la barra lateral: el contenido
//! lleva su fondo (sistema-macos.css).

use std::sync::OnceLock;

use tauri::Manager;
use window_vibrancy::{
    LiquidGlassOptions, NSGlassEffectViewStyle, NSVisualEffectMaterial, NSVisualEffectState,
    apply_liquid_glass, apply_vibrancy,
};

static VIDRIO: OnceLock<bool> = OnceLock::new();

/// Si la ventana tiene vidrio: la orden `plataforma` lo cuenta.
pub fn hay_vidrio() -> bool {
    VIDRIO.get().copied().unwrap_or(false)
}

/// Le pone el vidrio a la ventana principal. `setup` corre en el hilo de la
/// interfaz, que es donde AppKit lo exige.
pub fn poner_vidrio(app: &tauri::App) {
    let Some(ventana) = app.get_webview_window("principal") else {
        return;
    };
    // macOS 26 y posteriores: Liquid Glass. Antes, `apply_liquid_glass` dice
    // que no y se pone el material de barra lateral, el de la ADR 0016.
    let cristal = apply_liquid_glass(
        &ventana,
        LiquidGlassOptions::new(NSGlassEffectViewStyle::Regular),
    )
    .is_ok();
    let vidrio = cristal
        || apply_vibrancy(
            &ventana,
            NSVisualEffectMaterial::Sidebar,
            Some(NSVisualEffectState::FollowsWindowActiveState),
            None,
        )
        .is_ok();
    let _ = VIDRIO.set(vidrio);
}
