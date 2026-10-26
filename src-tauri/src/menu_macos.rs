//! El menú de la aplicación en macOS, en el idioma de Apolo (ADR 0025).
//!
//! Sin menú propio, Tauri pone el suyo con los textos en inglés («About»,
//! «Hide», «Quit», «Edit»…), aunque Apolo y el Mac estén en español: lo vio el
//! cliente en la v0.18.2. Este es el mismo menú, con lo que macOS trae en el
//! suyo («Mostrar todo»), y se rehace al cambiar el idioma en Ajustes. Lo que
//! macOS añade por su cuenta (Emoji y símbolos, Dictado) sale en el idioma del
//! sistema, como los diálogos de abrir y guardar: para eso el paquete declara
//! sus idiomas (`Info.plist` y `es.lproj`, `en.lproj`).

use apolo_nucleo::tr;
use tauri::menu::{
    AboutMetadata, HELP_SUBMENU_ID, Menu, PredefinedMenuItem, Submenu, WINDOW_SUBMENU_ID,
};
use tauri::{AppHandle, Manager, Runtime};

/// El menú, en el idioma de ahora. Los identificadores de Ventana y Ayuda son
/// los que Tauri le pasa a macOS para la lista de ventanas y la búsqueda.
pub fn crear<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let info = app.package_info();
    let config = app.config();
    let acerca = AboutMetadata {
        name: Some(info.name.clone()),
        version: Some(info.version.to_string()),
        copyright: config.bundle.copyright.clone(),
        authors: config.bundle.publisher.clone().map(|p| vec![p]),
        ..Default::default()
    };
    let separador = || PredefinedMenuItem::separator(app);

    let apolo = Submenu::with_items(
        app,
        "Apolo",
        true,
        &[
            &PredefinedMenuItem::about(
                app,
                Some(&tr!("Acerca de Apolo", "About Apolo")),
                Some(acerca),
            )?,
            &separador()?,
            &PredefinedMenuItem::services(app, Some(&tr!("Servicios", "Services")))?,
            &separador()?,
            &PredefinedMenuItem::hide(app, Some(&tr!("Ocultar Apolo", "Hide Apolo")))?,
            &PredefinedMenuItem::hide_others(app, Some(&tr!("Ocultar otros", "Hide Others")))?,
            &PredefinedMenuItem::show_all(app, Some(&tr!("Mostrar todo", "Show All")))?,
            &separador()?,
            &PredefinedMenuItem::quit(app, Some(&tr!("Salir de Apolo", "Quit Apolo")))?,
        ],
    )?;
    let archivo = Submenu::with_items(
        app,
        tr!("Archivo", "File"),
        true,
        &[&PredefinedMenuItem::close_window(
            app,
            Some(&tr!("Cerrar ventana", "Close Window")),
        )?],
    )?;
    let edicion = Submenu::with_items(
        app,
        tr!("Edición", "Edit"),
        true,
        &[
            &PredefinedMenuItem::undo(app, Some(&tr!("Deshacer", "Undo")))?,
            &PredefinedMenuItem::redo(app, Some(&tr!("Rehacer", "Redo")))?,
            &separador()?,
            &PredefinedMenuItem::cut(app, Some(&tr!("Cortar", "Cut")))?,
            &PredefinedMenuItem::copy(app, Some(&tr!("Copiar", "Copy")))?,
            &PredefinedMenuItem::paste(app, Some(&tr!("Pegar", "Paste")))?,
            &PredefinedMenuItem::select_all(app, Some(&tr!("Seleccionar todo", "Select All")))?,
        ],
    )?;
    let visualizacion = Submenu::with_items(
        app,
        tr!("Visualización", "View"),
        true,
        &[&PredefinedMenuItem::fullscreen(
            app,
            Some(&tr!("Pantalla completa", "Toggle Full Screen")),
        )?],
    )?;
    let ventana = Submenu::with_id_and_items(
        app,
        WINDOW_SUBMENU_ID,
        tr!("Ventana", "Window"),
        true,
        &[
            &PredefinedMenuItem::minimize(app, Some(&tr!("Minimizar", "Minimize")))?,
            &PredefinedMenuItem::maximize(app, Some("Zoom"))?,
            &separador()?,
            &PredefinedMenuItem::close_window(app, Some(&tr!("Cerrar", "Close")))?,
        ],
    )?;
    let ayuda = Submenu::with_id_and_items(app, HELP_SUBMENU_ID, tr!("Ayuda", "Help"), true, &[])?;

    Menu::with_items(
        app,
        &[&apolo, &archivo, &edicion, &visualizacion, &ventana, &ayuda],
    )
}
