//! `apolo-dev`: el Estudio por HTTP, para desarrollar y probar la interfaz en
//! un navegador sin compilar la ventana (ADR 0013).
//!
//! Cada orden de Tauri tiene aquí su `POST /api/<orden>` con los mismos
//! argumentos en JSON, y los píxeles salen por `GET /pixeles/…`, como el
//! protocolo `apolo://` de la aplicación. Solo escucha en 127.0.0.1.
//!
//! `make dev-web` lo levanta junto a Vite, que le reenvía `/api` y `/pixeles`.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use apolo_nucleo::presets::PresetGuardado;
use apolo_nucleo::salida::{Ajuste, FormatoSalida};
use apolo_nucleo::webp::{OpcionesWebp, Preset};
use apolo_servicio::{Fallo, Servicio, empaquetar_pixeles};
use axum::Json;
use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;

type Estado = State<Arc<Servicio>>;

struct Error(Fallo);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!(self.0)),
        )
            .into_response()
    }
}

impl From<Fallo> for Error {
    fn from(f: Fallo) -> Self {
        Error(f)
    }
}

type R<T> = Result<T, Error>;

fn binario(datos: Vec<u8>) -> Response {
    (
        [
            (header::CONTENT_TYPE, "application/octet-stream"),
            (header::CACHE_CONTROL, "no-store"),
        ],
        datos,
    )
        .into_response()
}

async fn bloqueante<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    tokio::task::spawn_blocking(f)
        .await
        .expect("la tarea se cayó")
}

#[derive(Deserialize)]
struct Id {
    id: u64,
}

#[derive(Deserialize)]
struct PeticionMedir {
    id: u64,
    #[serde(default)]
    lado: u8,
    generacion: u64,
}

#[derive(Deserialize)]
struct PeticionMapa {
    #[serde(default)]
    lado: u8,
    tipo: apolo_nucleo::medir::TipoMapa,
}

#[derive(Deserialize)]
struct PeticionCodificar {
    id: u64,
    ajuste: Ajuste,
    #[serde(default)]
    lado: u8,
    generacion: u64,
}

#[derive(Deserialize)]
struct PeticionExportar {
    id: u64,
    ajuste: Ajuste,
    #[serde(default)]
    lado: u8,
}

#[derive(Deserialize)]
struct PeticionNombre {
    id: u64,
    formato: FormatoSalida,
}

#[derive(Deserialize)]
struct PeticionLeerOrden {
    texto: String,
    base: Ajuste,
}

#[derive(Deserialize)]
struct PeticionPreset {
    opciones: OpcionesWebp,
    preset: Preset,
}

#[derive(Deserialize)]
struct PeticionNivel {
    opciones: OpcionesWebp,
    nivel: i32,
}

#[derive(Deserialize)]
struct PeticionOrden {
    ajuste: Ajuste,
}

#[derive(Deserialize)]
struct Nombre {
    nombre: String,
}

#[derive(Deserialize)]
struct Si {
    si: bool,
}

#[derive(Deserialize)]
struct Forzar {
    forzar: bool,
}

#[derive(Deserialize)]
struct Entradas {
    entradas: Vec<String>,
}

/// En camelCase, como los manda la interfaz (Tauri los pasa a snake_case solo).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EmpezarLote {
    entradas: Vec<String>,
    ajustes: Vec<Ajuste>,
    #[serde(default)]
    solo_mas_ligero: bool,
    #[serde(default)]
    medir: bool,
    salida: String,
}

#[derive(Deserialize)]
struct EstadoLote {
    id: u64,
    desde: usize,
}

#[derive(Deserialize)]
struct Guardar {
    preset: PresetGuardado,
}

#[derive(Deserialize)]
struct Enderezar {
    #[serde(default)]
    enderezar: u8,
}

#[derive(Deserialize)]
struct Lado {
    #[serde(default)]
    lado: u8,
}

fn rutas(s: Arc<Servicio>) -> Router {
    Router::new()
        .route("/salud", get(|| async { "ok" }))
        .route(
            "/api/inicio",
            post(|State(s): Estado| async move { Json(s.inicio()) }),
        )
        .route(
            "/api/motores",
            post(|| async { Json(apolo_nucleo::motores()) }),
        )
        .route(
            "/api/abrir",
            post(
                |State(s): Estado, cabeceras: HeaderMap, cuerpo: Bytes| async move {
                    let nombre = cabeceras
                        .get("x-nombre")
                        .and_then(|v| v.to_str().ok())
                        .map(percent_decode)
                        .unwrap_or_else(|| "imagen".into());
                    let r: R<_> = bloqueante(move || s.abrir_bytes(&nombre, cuerpo.to_vec()))
                        .await
                        .map(Json)
                        .map_err(Error);
                    r
                },
            ),
        )
        .route(
            "/api/cerrar",
            post(|State(s): Estado, Json(p): Json<Id>| async move {
                s.cerrar(p.id);
                Json(())
            }),
        )
        .route(
            "/api/codificar",
            post(
                |State(s): Estado, Json(p): Json<PeticionCodificar>| async move {
                    let r: R<_> =
                        bloqueante(move || s.codificar(p.id, &p.ajuste, p.lado, p.generacion))
                            .await
                            .map(Json)
                            .map_err(Error);
                    r
                },
            ),
        )
        .route(
            "/api/exportar",
            post(
                |State(s): Estado, Json(p): Json<PeticionExportar>| async move {
                    let r: R<_> = bloqueante(move || s.bytes_finales(p.id, &p.ajuste, p.lado))
                        .await
                        .map(binario)
                        .map_err(Error);
                    r
                },
            ),
        )
        .route(
            "/api/nombre_salida",
            post(
                |State(s): Estado, Json(p): Json<PeticionNombre>| async move {
                    s.nombre_salida(p.id, p.formato).map(Json).map_err(Error)
                },
            ),
        )
        .route(
            "/api/aplicar_preset",
            post(|Json(p): Json<PeticionPreset>| async move {
                Json(apolo_servicio::aplicar_preset_cwebp(&p.opciones, p.preset))
            }),
        )
        .route(
            "/api/nivel_sin_perdida",
            post(|Json(p): Json<PeticionNivel>| async move {
                apolo_servicio::nivel_sin_perdida(&p.opciones, p.nivel)
                    .map(Json)
                    .map_err(Error)
            }),
        )
        .route(
            "/api/leer_orden",
            post(|Json(p): Json<PeticionLeerOrden>| async move {
                apolo_servicio::leer_orden(&p.texto, &p.base)
                    .map(Json)
                    .map_err(Error)
            }),
        )
        .route(
            "/api/presets",
            post(|State(s): Estado| async move { Json(s.presets()) }),
        )
        .route(
            "/api/orden_opciones",
            post(|Json(p): Json<PeticionOrden>| async move {
                Json(apolo_servicio::orden_opciones(&p.ajuste))
            }),
        )
        .route(
            "/api/guardar_preset",
            post(|State(s): Estado, Json(p): Json<Guardar>| async move {
                s.guardar_preset(&p.preset).map(Json).map_err(Error)
            }),
        )
        .route(
            "/api/borrar_preset",
            post(|State(s): Estado, Json(p): Json<Nombre>| async move {
                s.borrar_preset(&p.nombre).map(Json).map_err(Error)
            }),
        )
        .route(
            "/api/recoger_lote",
            post(|Json(p): Json<Entradas>| async move {
                Json(bloqueante(move || apolo_servicio::recoger_lote(&p.entradas)).await)
            }),
        )
        .route(
            "/api/empezar_lote",
            post(|State(s): Estado, Json(p): Json<EmpezarLote>| async move {
                let r: R<_> = bloqueante(move || {
                    s.empezar_lote(
                        &p.entradas,
                        &p.ajustes,
                        p.solo_mas_ligero,
                        p.medir,
                        &p.salida,
                    )
                })
                .await
                .map(Json)
                .map_err(Error);
                r
            }),
        )
        .route(
            "/api/estado_lote",
            post(|State(s): Estado, Json(p): Json<EstadoLote>| async move {
                s.estado_lote(p.id, p.desde).map(Json).map_err(Error)
            }),
        )
        .route(
            "/api/cancelar_lote",
            post(|State(s): Estado, Json(p): Json<Id>| async move {
                s.cancelar_lote(p.id);
                Json(())
            }),
        )
        .route(
            "/api/ajustes",
            post(|State(s): Estado| async move { Json(s.ajustes()) }),
        )
        .route(
            "/api/buscar_actualizaciones",
            post(|State(s): Estado, Json(p): Json<Si>| async move {
                Json(s.buscar_actualizaciones(p.si))
            }),
        )
        .route(
            "/api/reservar_comprobacion",
            post(|State(s): Estado, Json(p): Json<Forzar>| async move {
                Json(s.reservar_comprobacion(p.forzar))
            }),
        )
        .route(
            "/pixeles/original/{id}",
            get(
                |State(s): Estado, Path(id): Path<u64>, Query(q): Query<Enderezar>| async move {
                    let r: R<_> = bloqueante(move || s.pixeles_original(id, q.enderezar == 1))
                        .await
                        .map(|p| binario(empaquetar_pixeles(p)))
                        .map_err(Error);
                    r
                },
            ),
        )
        .route(
            "/api/medir",
            post(
                |State(s): Estado, Json(p): Json<PeticionMedir>| async move {
                    let r: R<_> = bloqueante(move || s.medir(p.id, p.lado, p.generacion))
                        .await
                        .map(Json)
                        .map_err(Error);
                    r
                },
            ),
        )
        .route(
            "/pixeles/mapa/{id}",
            get(
                |State(s): Estado, Path(id): Path<u64>, Query(q): Query<PeticionMapa>| async move {
                    let r: R<_> = bloqueante(move || s.pixeles_mapa(id, q.lado, q.tipo))
                        .await
                        .map(|p| binario(empaquetar_pixeles(p)))
                        .map_err(Error);
                    r
                },
            ),
        )
        .route(
            "/pixeles/resultado/{id}",
            get(
                |State(s): Estado, Path(id): Path<u64>, Query(q): Query<Lado>| async move {
                    s.pixeles_resultado(id, q.lado)
                        .map(|p| binario(empaquetar_pixeles(p)))
                        .map_err(Error)
                },
            ),
        )
        .layer(DefaultBodyLimit::max(512 * 1024 * 1024))
        .with_state(s)
}

/// El nombre llega en una cabecera, codificado con encodeURIComponent.
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut v = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Ok(x) =
                u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or(""), 16)
        {
            v.push(x);
            i += 3;
            continue;
        }
        v.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&v).into_owned()
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let mut direccion: SocketAddr = "127.0.0.1:34500".parse().unwrap();
    let mut config: Option<PathBuf> = None;
    while let Some(a) = args.next() {
        match a.as_str() {
            "--direccion" => {
                direccion = args
                    .next()
                    .expect("falta la dirección")
                    .parse()
                    .expect("dirección no válida")
            }
            "--config" => config = Some(args.next().expect("falta la carpeta").into()),
            otro => panic!("Opción desconocida: {otro}"),
        }
    }
    assert!(
        direccion.ip().is_loopback(),
        "el servidor de desarrollo solo escucha en 127.0.0.1"
    );
    let servicio = Arc::new(match config {
        Some(p) => Servicio::nuevo(p),
        None => Servicio::con_carpeta_por_defecto(),
    });
    eprintln!(
        "apolo-dev en http://{direccion} · presets en {}",
        servicio.carpeta_presets().display()
    );
    let escucha = tokio::net::TcpListener::bind(direccion)
        .await
        .expect("no se puede escuchar");
    axum::serve(escucha, rutas(servicio)).await.unwrap();
}
