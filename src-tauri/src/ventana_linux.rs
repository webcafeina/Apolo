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
//! **El marco.** Sin decoraciones, GTK tampoco dibuja el marco de la ventana:
//! ni la sombra, ni las esquinas redondeadas, ni el borde exterior de
//! redimensionado (lo vio el cliente en su Ubuntu). En Wayland, tao pone de
//! barra de título un `EventBox` vacío; volviendo a decorar la ventana, GTK
//! dibuja su marco alrededor de esa barra de 0 px, y la de Apolo queda en su
//! sitio. El radio del marco no recorta el webview, así que la ventana es
//! transparente y la interfaz redondea sus propias esquinas, y las cuadra si la
//! ventana está maximizada o en mosaico (el evento `ventana:esquinas`).
//!
//! Lo que una ventana sin barra del sistema pierde en Linux: el menú del clic
//! derecho sobre la barra (Tauri no lo da) (deuda.md, trampas.md).

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use gtk::gdk::WindowState;
use gtk::prelude::{CssProviderExt, GtkSettingsExt, GtkWindowExt, WidgetExt};
use tauri::{Emitter, WebviewWindowBuilder};

/// El evento que oye la interfaz: `true` con las esquinas redondeadas, `false`
/// con la ventana maximizada, a pantalla completa o en mosaico.
pub const EVENTO_ESQUINAS: &str = "ventana:esquinas";

/// Si la interfaz redondea sus esquinas (hay marco y la ventana está suelta).
static ESQUINAS: AtomicBool = AtomicBool::new(false);

/// El marco de la ventana como el de libadwaita (`widgets/_window.scss` de la
/// 1.10, las mismas sombras): radio 15 en las cuatro esquinas, la sombra de la
/// ventana activa y, en segundo plano, la suya, más tenue y del mismo tamaño
/// para que la ventana no salte; sin radio ni sombra maximizada o a pantalla
/// completa, y en mosaico, solo el filo. La barra vacía de tao, sin estilo.
const MARCO: &[u8] = br#"
window.csd { background-color: transparent; }
window.csd decoration {
  border-radius: 15px;
  box-shadow: 0 0 14px 5px rgba(0, 0, 0, 0.15), 0 0 5px 2px rgba(0, 0, 0, 0.10), 0 0 0 1px rgba(0, 0, 0, 0.05);
}
window.csd:backdrop decoration {
  box-shadow: 0 0 14px 5px transparent, 0 0 10px 5px rgba(0, 0, 0, 0.08), 0 0 0 1px rgba(0, 0, 0, 0.05);
  transition: box-shadow 200ms ease-out;
}
window.maximized decoration, window.fullscreen decoration { border-radius: 0; box-shadow: none; }
window.tiled decoration, window.tiled-top decoration, window.tiled-right decoration,
window.tiled-bottom decoration, window.tiled-left decoration {
  border-radius: 0;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.05);
}
window.csd > .titlebar:not(headerbar) { min-height: 0; padding: 0; border: 0; background: none; box-shadow: none; }
"#;

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

/// Si la interfaz redondea sus esquinas: hay marco de GTK y la ventana está
/// suelta (ni maximizada, ni a pantalla completa, ni en mosaico).
pub fn esquinas() -> bool {
    ESQUINAS.load(Ordering::Relaxed)
}

/// Los botones de la ventana, como `gtk-decoration-layout`.
pub fn botones() -> String {
    VENTANA
        .get()
        .map(|v| v.botones.clone())
        .unwrap_or_else(|| BOTONES_POR_DEFECTO.to_string())
}

/// El escritorio, para el aspecto de la interfaz: `gnome`, `kde` (ADR 0031)
/// u `otro`, que se queda con el de GNOME. `XDG_CURRENT_DESKTOP` es una lista
/// (en Ubuntu, `ubuntu:GNOME`; en Budgie, `Budgie:GNOME`).
pub fn escritorio() -> &'static str {
    let lista = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();
    let tiene = |n: &str| lista.split(':').any(|p| p.eq_ignore_ascii_case(n));
    if tiene("kde") {
        "kde"
    } else if tiene("gnome") {
        "gnome"
    } else {
        "otro"
    }
}

fn es_gnome() -> bool {
    escritorio() == "gnome"
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
    // El marco solo en Wayland, donde tao pone la barra vacía; en X11, con
    // la ventana decorada volvería la barra del servidor. Y la ventana,
    // transparente solo entonces, para que las esquinas de la interfaz dejen
    // ver la sombra (la transparencia da guerra con algunas gráficas).
    let wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
        && std::env::var("GDK_BACKEND").map_or(true, |b| b.contains("wayland"));
    // `APOLO_ESQUINAS_RECTAS=1` la deja sin marco, como la primera
    // compilación de prueba que vio el cliente (el plan B): sin sombra y con
    // las esquinas rectas, pero sin transparencia. Sin transparencia no hay
    // marco posible: la sombra de GTK necesita una ventana con alfa.
    // Los huecos al repintar que vio el cliente en su máquina virtual no son
    // del marco: salen igual sin él y con la primera compilación de prueba
    // (trampas.md).
    let marco = propia && wayland && std::env::var_os("APOLO_ESQUINAS_RECTAS").is_none();
    let ventana = WebviewWindowBuilder::from_config(app.handle(), &config)?
        .decorations(!propia)
        .transparent(marco)
        .build()?;
    let _ = VENTANA.set(Ventana {
        barra_propia: propia,
        botones,
    });
    if marco {
        poner_marco(&ventana)?;
    }
    Ok(())
}

/// Vuelve a decorar la ventana para que GTK dibuje su marco (con la barra
/// vacía que puso tao) y sigue si está maximizada o en mosaico, para que la
/// interfaz cuadre sus esquinas.
fn poner_marco(ventana: &tauri::WebviewWindow) -> tauri::Result<()> {
    let gw = ventana.gtk_window()?;
    if gw.titlebar().is_none() {
        return Ok(());
    }
    let css = gtk::CssProvider::new();
    if css.load_from_data(MARCO).is_ok()
        && let Some(pantalla) = GtkWindowExt::screen(&gw)
    {
        gtk::StyleContext::add_provider_for_screen(
            &pantalla,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    // Se puede con la ventana ya creada: GTK recalcula la sombra y el borde
    // de redimensionado sin rehacerla (con `set_titlebar`, sí la rehace).
    gw.set_decorated(true);
    ESQUINAS.store(true, Ordering::Relaxed);
    let aviso = ventana.clone();
    gw.connect_window_state_event(move |_, e| {
        let estado = e.new_window_state();
        let cuadrada = estado.intersects(
            WindowState::MAXIMIZED
                | WindowState::FULLSCREEN
                | WindowState::TILED
                | WindowState::TOP_TILED
                | WindowState::RIGHT_TILED
                | WindowState::BOTTOM_TILED
                | WindowState::LEFT_TILED,
        );
        if ESQUINAS.swap(!cuadrada, Ordering::Relaxed) == cuadrada {
            let _ = aviso.emit(EVENTO_ESQUINAS, !cuadrada);
        }
        gtk::glib::Propagation::Proceed
    });
    Ok(())
}
