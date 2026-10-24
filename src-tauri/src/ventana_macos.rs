//! La ventana en macOS (ADR 0028): el material de barra lateral del sistema
//! detrás de la ventana, y el vidrio de Liquid Glass en los controles desde
//! macOS 26.
//!
//! La ventana sale de `tauri.macos.conf.json`, ya transparente y con la barra
//! de título superpuesta, con los semáforos colocados para la barra de
//! herramientas unificada de 52 px (`trafficLightPosition`). Aquí solo se le
//! pone el vidrio, y se pone a mano y no con `windowEffects` para saber si se
//! pudo: `window_vibrancy` devuelve el error, Tauri se lo traga. Si no hay
//! vidrio, la interfaz pinta el fondo opaco (`data-vidrio="no"`).
//!
//! El material es `NSVisualEffectMaterial::Sidebar`, el que Apple da a las
//! barras laterales. Va detrás de toda la ventana; la interfaz solo lo deja ver
//! en la barra lateral: el contenido lleva su fondo (sistema-macos.css). Hasta
//! la v0.18.1, en macOS 26 y 27 iba el `NSGlassEffectView` (estilo `Regular`),
//! que es el de los controles flotantes: dejaba pasar mucho más de las ventanas
//! de detrás que la barra lateral de Finder (el cliente lo comparó en su Mac).

//!
//! **El vidrio de los controles.** WebKit tiene una propiedad CSS propia de
//! Apple, `-apple-visual-effect: -apple-system-glass-material`, que pinta
//! debajo de un elemento HTML el material de vidrio del sistema (el de SwiftUI,
//! con su refracción), desde macOS 26. Viene apagada, detrás de una preferencia
//! privada de `WKPreferences` (`_setUseSystemAppearance:`). Aquí se enciende si
//! existe, y la interfaz pone el vidrio de verdad en las cápsulas de la barra
//! (vidrio-macos.css); si no, se queda la imitación de sistema-macos.css. Es
//! API privada: vale fuera de la Mac App Store, que es como se distribuye
//! Apolo. La técnica, de lennondotw/tauri-liquid-glass-demo (MIT).

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use objc2::runtime::{AnyClass, AnyObject};
use objc2::{msg_send, sel};
use tauri::Manager;
use window_vibrancy::{NSVisualEffectMaterial, NSVisualEffectState, apply_vibrancy};

static VIDRIO: OnceLock<bool> = OnceLock::new();
static VIDRIO_WEB: AtomicBool = AtomicBool::new(false);

/// Si la ventana tiene vidrio: la orden `plataforma` lo cuenta.
pub fn hay_vidrio() -> bool {
    VIDRIO.get().copied().unwrap_or(false)
}

/// Si la interfaz puede pintar el vidrio del sistema en sus controles.
pub fn hay_vidrio_web() -> bool {
    VIDRIO_WEB.load(Ordering::Relaxed)
}

/// Enciende `-apple-visual-effect` en el WKWebView de la ventana. Solo con
/// Liquid Glass (macOS 26+): antes, la propiedad no tiene el material de vidrio.
fn encender_vidrio_web(ventana: &tauri::WebviewWindow) {
    let _ = ventana.with_webview(|w| unsafe {
        let vista = w.inner().cast::<AnyObject>();
        if vista.is_null() {
            return;
        }
        // `configuration` y `preferences` son públicas; solo el último no.
        let configuracion: *mut AnyObject = msg_send![&*vista, configuration];
        if configuracion.is_null() {
            return;
        }
        let preferencias: *mut AnyObject = msg_send![&*configuracion, preferences];
        if preferencias.is_null() {
            return;
        }
        let existe: bool =
            msg_send![&*preferencias, respondsToSelector: sel!(_setUseSystemAppearance:)];
        if existe {
            let _: () = msg_send![&*preferencias, _setUseSystemAppearance: true];
            VIDRIO_WEB.store(true, Ordering::Relaxed);
        }
    });
}

/// Le pone el vidrio a la ventana principal. `setup` corre en el hilo de la
/// interfaz, que es donde AppKit lo exige.
pub fn poner_vidrio(app: &tauri::App) {
    let Some(ventana) = app.get_webview_window("principal") else {
        return;
    };
    // El material de barra lateral, en todas las versiones (ADR 0016 y 0028).
    let vidrio = apply_vibrancy(
        &ventana,
        NSVisualEffectMaterial::Sidebar,
        Some(NSVisualEffectState::FollowsWindowActiveState),
        None,
    )
    .is_ok();
    // Liquid Glass (macOS 26+), donde existe su vista: el vidrio de los controles.
    if AnyClass::get(c"NSGlassEffectView").is_some() {
        encender_vidrio_web(&ventana);
    }
    let _ = VIDRIO.set(vidrio);
}
