//! La aplicación de ventana: órdenes de Tauri sobre `apolo_servicio`.
//!
//! Aquí no hay lógica: cada orden llama a la función del servicio del mismo
//! nombre, igual que el servidor de desarrollo (`crates/dev`). Lo único propio
//! es el protocolo `apolo://`, por donde salen los píxeles (ADR 0013).

use std::path::PathBuf;
use std::sync::Arc;

use apolo_nucleo::Motor;
use apolo_nucleo::presets::PresetGuardado;
use apolo_nucleo::webp::{OpcionesWebp, Preset};
use apolo_servicio::{Fallo, InfoImagen, Inicio, Servicio, Vista, empaquetar_pixeles};
use tauri::State;
use tauri::http::{Response, StatusCode, header};

type Estado<'a> = State<'a, Arc<Servicio>>;

async fn bloqueante<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .expect("la tarea se cayó")
}

#[tauri::command]
fn inicio(s: Estado) -> Inicio {
    s.inicio()
}

#[tauri::command]
fn motores() -> Vec<Motor> {
    apolo_nucleo::motores()
}

#[tauri::command]
async fn abrir(s: Estado<'_>, ruta: PathBuf) -> Result<InfoImagen, Fallo> {
    let s = s.inner().clone();
    bloqueante(move || s.abrir_ruta(&ruta)).await
}

#[tauri::command]
fn cerrar(s: Estado, id: u64) {
    s.cerrar(id)
}

#[tauri::command]
async fn codificar(
    s: Estado<'_>,
    id: u64,
    opciones: OpcionesWebp,
    generacion: u64,
) -> Result<Vista, Fallo> {
    let s = s.inner().clone();
    bloqueante(move || s.codificar(id, &opciones, generacion)).await
}

#[tauri::command]
async fn exportar(
    s: Estado<'_>,
    id: u64,
    opciones: OpcionesWebp,
    ruta: PathBuf,
) -> Result<usize, Fallo> {
    let s = s.inner().clone();
    bloqueante(move || s.exportar(id, &opciones, &ruta)).await
}

#[tauri::command]
fn nombre_salida(s: Estado, id: u64) -> Result<String, Fallo> {
    s.nombre_salida(id)
}

#[tauri::command]
fn aplicar_preset(opciones: OpcionesWebp, preset: Preset) -> OpcionesWebp {
    apolo_servicio::aplicar_preset_cwebp(&opciones, preset)
}

#[tauri::command]
fn nivel_sin_perdida(opciones: OpcionesWebp, nivel: i32) -> Result<OpcionesWebp, Fallo> {
    apolo_servicio::nivel_sin_perdida(&opciones, nivel)
}

#[tauri::command]
fn leer_orden(texto: String) -> Result<OpcionesWebp, Fallo> {
    apolo_servicio::leer_orden(&texto)
}

#[tauri::command]
fn presets(s: Estado) -> Vec<PresetGuardado> {
    s.presets()
}

#[tauri::command]
fn guardar_preset(s: Estado, preset: PresetGuardado) -> Result<Vec<PresetGuardado>, Fallo> {
    s.guardar_preset(&preset)
}

#[tauri::command]
fn borrar_preset(s: Estado, nombre: String) -> Result<Vec<PresetGuardado>, Fallo> {
    s.borrar_preset(&nombre)
}

/// `apolo://localhost/original/<id>?enderezar=1` y `…/resultado/<id>`: ancho
/// y alto en u32 little-endian y el RGBA detrás. En Windows llega como
/// `http://apolo.localhost/…`; la ruta es la misma.
fn pixeles(s: &Servicio, ruta: &str, consulta: Option<&str>) -> Result<Vec<u8>, Fallo> {
    let partes: Vec<&str> = ruta.trim_matches('/').split('/').collect();
    let id = || -> Result<u64, Fallo> {
        partes
            .get(1)
            .and_then(|x| x.parse().ok())
            .ok_or_else(|| Fallo {
                mensaje: "Ruta no válida".into(),
                cancelado: false,
            })
    };
    match partes.first() {
        Some(&"original") => {
            let enderezar = consulta.is_some_and(|q| q.split('&').any(|p| p == "enderezar=1"));
            s.pixeles_original(id()?, enderezar).map(empaquetar_pixeles)
        }
        Some(&"resultado") => s.pixeles_resultado(id()?).map(empaquetar_pixeles),
        _ => Err(Fallo {
            mensaje: "Ruta no válida".into(),
            cancelado: false,
        }),
    }
}

pub fn arrancar() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(Servicio::con_carpeta_por_defecto()))
        .register_asynchronous_uri_scheme_protocol("apolo", |ctx, peticion, respuesta| {
            use tauri::Manager;
            let s = ctx.app_handle().state::<Arc<Servicio>>().inner().clone();
            let ruta = peticion.uri().path().to_string();
            let consulta = peticion.uri().query().map(String::from);
            std::thread::spawn(move || {
                let r = match pixeles(&s, &ruta, consulta.as_deref()) {
                    Ok(datos) => Response::builder()
                        .header(header::CONTENT_TYPE, "application/octet-stream")
                        .header(header::CACHE_CONTROL, "no-store")
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(datos),
                    Err(f) => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                        .body(f.mensaje.into_bytes()),
                };
                respuesta.respond(r.expect("respuesta bien formada"));
            });
        })
        .invoke_handler(tauri::generate_handler![
            inicio,
            motores,
            abrir,
            cerrar,
            codificar,
            exportar,
            nombre_salida,
            aplicar_preset,
            nivel_sin_perdida,
            leer_orden,
            presets,
            guardar_preset,
            borrar_preset
        ])
        .run(tauri::generate_context!())
        .expect("No se pudo arrancar Apolo");
}
