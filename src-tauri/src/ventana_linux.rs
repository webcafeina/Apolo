//! La ventana en Linux (ADR 0029): en GNOME, sin la barra del sistema, con la
//! barra de Apolo como la de una aplicación de libadwaita; en los demás
//! escritorios (KDE, Xfce…), con la barra del sistema, que allí encaja mejor.
//!
//! La ventana se crea aquí y no desde `tauri.conf.json` (`tauri.linux.conf.json`
//! la deja en `"create": false`) porque las decoraciones se deciden al crearla y
//! dependen del escritorio, que solo se sabe al arrancar. La interfaz se entera
//! por la orden `plataforma` (`barra_propia`) y pinta entonces los botones de la
//! ventana en el lado y el orden que diga el sistema (`botones`, el
//! `gtk-decoration-layout` de GTK, que GNOME sincroniza con su ajuste
//! `button-layout`; en Ubuntu, «:minimize,maximize,close»).
//!
//! Lo que una ventana sin barra del sistema pierde en Linux: el menú del clic
//! derecho sobre la barra (Tauri no lo da) y el borde para redimensionar, que
//! es una franja de 5 px por dentro de la ventana (deuda.md, trampas.md).

use std::sync::OnceLock;

use gtk::prelude::GtkSettingsExt;
use tauri::WebviewWindowBuilder;

/// Lo que se decidió al crear la ventana.
struct Ventana {
    barra_propia: bool,
    botones: String,
}

static VENTANA: OnceLock<Ventana> = OnceLock::new();

/// La disposición de Ubuntu, si GTK no dice otra.
const BOTONES_POR_DEFECTO: &str = ":minimize,maximize,close";

/// Si la ventana va sin la barra del sistema y Apolo pinta la suya.
pub fn barra_propia() -> bool {
    VENTANA.get().is_some_and(|v| v.barra_propia)
}

/// Los botones de la ventana, como `gtk-decoration-layout`.
pub fn botones() -> String {
    VENTANA
        .get()
        .map(|v| v.botones.clone())
        .unwrap_or_else(|| BOTONES_POR_DEFECTO.to_string())
}

/// GNOME (en Ubuntu, `XDG_CURRENT_DESKTOP=ubuntu:GNOME`).
fn es_gnome() -> bool {
    std::env::var("XDG_CURRENT_DESKTOP")
        .is_ok_and(|d| d.split(':').any(|p| p.eq_ignore_ascii_case("gnome")))
}

/// Crea la ventana principal con la configuración de `tauri.linux.conf.json`.
pub fn crear(app: &tauri::App) -> tauri::Result<()> {
    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|v| v.label == "principal")
        .expect("la ventana principal está en tauri.linux.conf.json")
        .clone();
    let propia = es_gnome();
    // La lee GTK, que ya está en marcha: `setup` corre en su hilo. Sin
    // `gio::Settings`, que aborta el programa si falta el esquema de GNOME.
    let botones = gtk::Settings::default()
        .and_then(|s| s.gtk_decoration_layout())
        .map(|l| l.to_string())
        .filter(|l| l.contains(':'))
        .unwrap_or_else(|| BOTONES_POR_DEFECTO.to_string());
    WebviewWindowBuilder::from_config(app.handle(), &config)?
        .decorations(!propia)
        .build()?;
    let _ = VENTANA.set(Ventana {
        barra_propia: propia,
        botones,
    });
    Ok(())
}
