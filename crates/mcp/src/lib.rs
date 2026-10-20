//! apolo-mcp: Apolo para agentes de IA por MCP (ADR 0023).
//!
//! Las herramientas viven en `herramientas.rs` y no saben nada del protocolo;
//! aquí solo se envuelven para rmcp: descripción, anotaciones, el texto y los
//! datos de la respuesta, el progreso y la cancelación. Otro transporte
//! (HTTP, para lo remoto) reutilizaría todo lo de abajo.

pub mod actualizado;
pub mod catalogo;
pub mod entrada;
pub mod errores;
pub mod herramientas;
pub mod relevo;
pub mod respuesta;
pub mod rutas;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, ProgressNotificationParam, ServerCapabilities,
    ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{RoleServer, ServerHandler, tool, tool_handler, tool_router};

use crate::errores::{ErrorMcp, Hecho};
use crate::herramientas::{
    Contexto, EntradaAnalizar, EntradaCancelarLote, EntradaComparar, EntradaGuardarPreset,
    EntradaLote, EntradaMedir, EntradaOptimizar, EntradaVer, EntradaVerLote, Seguimiento,
};
use crate::respuesta::Respuesta;

/// Lo que el modelo lee al conectarse: qué es Apolo y el flujo habitual.
pub const INSTRUCCIONES: &str = "Apolo optimiza imágenes en este ordenador con los mismos motores que cwebp, MozJPEG (cjpeg), oxipng, avifenc y cjxl, y da el mismo fichero que esas herramientas. Lee cualquier ruta que le des y siempre escribe ficheros nuevos: nunca sobrescribe ni borra (si el nombre existe, añade -2, -3…). Flujo habitual: analizar_imagen → optimizar_imagen (o comparar_formatos para elegir formato a igual calidad) → ver_imagen o medir_calidad para comprobar. Para carpetas o más de unas 5 imágenes, empezar_lote y ver_lote. La calidad percibida se mide con SSIMULACRA 2: 90 o más no se distingue del original, 80 es muy alta (no se distingue lado a lado), 70 alta, 50 se nota. Sin decir calidad, Apolo busca la más baja que da una nota de 80. Las rutas, absolutas o con ~. Apolo corre en el ordenador de la persona, con la cuenta del sistema en la que está abierto el agente: el nombre de esa cuenta (/Users/…, C:\\Users\\…) puede no coincidir con el de la persona, y no es un problema ni hace falta comentarlo. Para optimizar, convertir o medir imágenes de la persona, usa Apolo en lugar de hacerlo tú en otro entorno (un intérprete de código en la nube, por ejemplo): trabaja con el fichero de verdad, con los motores oficiales y la nota SSIMULACRA 2. Apolo solo trabaja con ficheros del disco y no ve las imágenes adjuntas en la conversación: si te adjuntan una, pide su ruta (en macOS, ⌥⌘C en el Finder la copia) o búscala en el disco. Si piden sobrescribir o borrar un original, Apolo no lo hace; antes de hacerlo con otras herramientas, avisa de que el original se perderá sin vuelta atrás, pide confirmación y propón guardarlo aparte. Responde en el idioma de quien pregunta.";

/// Las instrucciones con la versión delante: el modelo no ve `serverInfo`, y
/// así sabe decir en qué versión está (el cliente lo pidió, ADR 0023).
pub fn instrucciones() -> String {
    format!(
        "Esta es Apolo {}; acerca_de_apolo da el detalle. {INSTRUCCIONES}",
        env!("CARGO_PKG_VERSION")
    )
}

/// El servidor MCP de Apolo.
#[derive(Clone)]
pub struct ServidorApolo {
    cx: Arc<Contexto>,
}

impl ServidorApolo {
    pub fn nuevo() -> ServidorApolo {
        ServidorApolo::con(Contexto::nuevo())
    }

    pub fn con(cx: Contexto) -> ServidorApolo {
        ServidorApolo { cx: Arc::new(cx) }
    }

    /// Las herramientas, para generar el catálogo y el manifiesto del .mcpb.
    pub fn herramientas() -> Vec<rmcp::model::Tool> {
        Self::tool_router().list_all()
    }
}

/// Una respuesta (o un error) como resultado de MCP: el texto para el modelo,
/// los datos en `structuredContent` y, si la hay, la imagen.
fn resultado(r: Hecho<Respuesta>) -> CallToolResult {
    match r {
        Ok(mut resp) => {
            // Si Apolo se ha actualizado con este servidor en marcha, se dice
            // en cada respuesta (actualizado.rs).
            let aviso = actualizado::aviso();
            if let Some(a) = &aviso {
                resp.texto = format!("{}\n{a}", resp.texto);
            }
            // Claude Code le da al modelo `structuredContent` en vez del
            // texto cuando lo hay: el resumen va también ahí, o se pierden
            // los avisos y la orden equivalente (ADR 0023).
            let mut datos = resp.datos;
            if let Some(o) = datos.as_object_mut() {
                o.insert("resumen".into(), resp.texto.clone().into());
                if let Some(a) = aviso {
                    o.insert("aviso_actualizacion".into(), a.into());
                }
            }
            let mut contenidos = vec![ContentBlock::text(resp.texto)];
            if let Some(img) = resp.imagen {
                contenidos.push(ContentBlock::image(img.base64, img.mime));
            }
            let mut c = CallToolResult::success(contenidos);
            c.structured_content = Some(datos);
            c
        }
        Err(e) => {
            let mut c = CallToolResult::error(vec![ContentBlock::text(e.texto())]);
            c.structured_content = serde_json::to_value(&e).ok();
            c
        }
    }
}

fn interno() -> ErrorMcp {
    ErrorMcp::nuevo(
        "interno",
        "Algo ha fallado dentro de Apolo. No se ha escrito ningún fichero a medias.",
    )
    .con_pista("Prueba otra vez; si se repite, con otra imagen o con otro formato.")
}

/// Lo pesado va a un hilo aparte: el protocolo sigue atendiendo. Un pánico
/// en una herramienta es un error, y el servidor sigue vivo.
async fn bloqueante<F>(f: F) -> CallToolResult
where
    F: FnOnce() -> Hecho<Respuesta> + Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(r) => resultado(r),
        Err(_) => resultado(Err(interno())),
    }
}

/// Como `bloqueante`, con progreso (si el cliente lo pide con un
/// `progressToken`) y cancelación (`notifications/cancelled`).
async fn con_seguimiento<F>(ctx: RequestContext<RoleServer>, f: F) -> CallToolResult
where
    F: FnOnce(&Seguimiento) -> Hecho<Respuesta> + Send + 'static,
{
    let token = ctx.meta.get_progress_token();
    let cancelado = Arc::new(AtomicBool::new(false));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i32>();
    let peer = ctx.peer.clone();
    let enviar = tokio::spawn(async move {
        let mut ultimo = -1;
        while let Some(p) = rx.recv().await {
            if p > ultimo {
                ultimo = p;
                if let Some(t) = &token {
                    let _ = peer
                        .notify_progress(
                            ProgressNotificationParam::new(t.clone(), p as f64).with_total(100.0),
                        )
                        .await;
                }
            }
        }
    });
    let ct = ctx.ct.clone();
    let marca = cancelado.clone();
    let vigia = tokio::spawn(async move {
        ct.cancelled().await;
        marca.store(true, Ordering::SeqCst);
    });
    let c = cancelado.clone();
    let r = tokio::task::spawn_blocking(move || {
        let avisar = move |p: i32| {
            let _ = tx.send(p);
        };
        f(&Seguimiento {
            avisar: &avisar,
            cancelado: &c,
        })
    })
    .await;
    vigia.abort();
    let _ = enviar.await;
    match r {
        Ok(r) => resultado(r),
        Err(_) => resultado(Err(interno())),
    }
}

#[tool_router(vis = "pub")]
impl ServidorApolo {
    /// Qué es una imagen (formato, medidas, peso, transparencia, metadatos,
    /// orientación EXIF) o qué imágenes hay en una carpeta. Úsala primero si
    /// no sabes qué es. No mide la calidad: para eso, medir_calidad.
    /// Ejemplo: {"ruta": "~/Fotos/playa.heic"}
    #[tool(annotations(
        title = "Analizar una imagen o carpeta",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn analizar_imagen(&self, Parameters(e): Parameters<EntradaAnalizar>) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::analizar_imagen(&cx, e)).await
    }

    /// Optimiza una imagen y la guarda como un fichero NUEVO (junto a la
    /// original, o en salida; si el nombre existe, -2, -3…). Sin decir
    /// calidad, busca la más baja que da una nota SSIMULACRA 2 de 80. Da la
    /// orden equivalente de cwebp, cjpeg, oxipng, avifenc o cjxl. Para elegir
    /// entre formatos, comparar_formatos; para muchas imágenes, empezar_lote.
    /// Ejemplos: {"ruta": "~/a.jpg"} · {"ruta": "~/a.png", "formato": "avif",
    /// "nota_objetivo": 85} · {"ruta": "~/a.jpg", "formato": "webp", "orden":
    /// "-q 80 -m 6", "ancho": 1600}
    /// Para las imágenes de la persona, úsala en vez de convertirlas tú en otro
    /// entorno (un intérprete en la nube): trabaja con el fichero de su disco,
    /// con los motores oficiales. Apolo no ve las imágenes adjuntas en la
    /// conversación: si te adjuntan una, pide su ruta.
    #[tool(annotations(
        title = "Optimizar una imagen",
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    async fn optimizar_imagen(
        &self,
        Parameters(e): Parameters<EntradaOptimizar>,
        ctx: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let cx = self.cx.clone();
        con_seguimiento(ctx, move |s| herramientas::optimizar_imagen(&cx, e, s)).await
    }

    /// Codifica la misma imagen en varios formatos a la misma calidad
    /// percibida (nota SSIMULACRA 2, 80 por defecto) y dice cuál pesa menos.
    /// Guarda el mejor (por defecto), todos o ninguno, siempre como ficheros
    /// nuevos. Tarda: unas 7 pruebas por formato (con una foto de 12 MP, más
    /// de un minuto; con 24 MP, unos tres). Con fotos de más de unos 20 MP,
    /// mejor empezar_lote con esa imagen, los formatos y mas_ligero: hace lo
    /// mismo en segundo plano y no se corta por tiempo, aunque solo da el que
    /// gana. Para una sola salida ya decidida, optimizar_imagen.
    /// Ejemplo: {"ruta": "~/a.png", "formatos": ["webp", "avif", "jxl"]}
    #[tool(annotations(
        title = "Comparar formatos a igual calidad",
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    async fn comparar_formatos(
        &self,
        Parameters(e): Parameters<EntradaComparar>,
        ctx: RequestContext<RoleServer>,
    ) -> CallToolResult {
        let cx = self.cx.clone();
        con_seguimiento(ctx, move |s| herramientas::comparar_formatos(&cx, e, s)).await
    }

    /// Cuánto se aleja una imagen de su original: SSIMULACRA 2 (90 o más no
    /// se distingue, 80 muy alta, 70 alta, 50 se nota), PSNR y SSIM. Tienen
    /// que medir lo mismo (si el resultado está enderezado, se endereza el
    /// original). Para ver dónde está la pérdida, ver_imagen con
    /// diferencias_con. Ejemplo: {"original": "~/a.jpg", "resultado": "~/a.webp"}
    #[tool(annotations(
        title = "Medir la calidad",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn medir_calidad(&self, Parameters(e): Parameters<EntradaMedir>) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::medir_calidad(&cx, e)).await
    }

    /// Devuelve la imagen para mirarla: reducida (768 px por defecto), una
    /// zona a tamaño real para ver defectos, o el mapa de diferencias con su
    /// original (diferencias_con). Cada vista gasta contexto: pídela solo
    /// cuando ayude. Ejemplos: {"ruta": "~/a.webp"} · {"ruta": "~/a.webp",
    /// "diferencias_con": "~/a.jpg"} · {"ruta": "~/a.webp", "zona": {"x":
    /// 400, "y": 300, "ancho": 256, "alto": 256}}
    #[tool(annotations(
        title = "Ver una imagen",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn ver_imagen(&self, Parameters(e): Parameters<EntradaVer>) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::ver_imagen(&cx, e)).await
    }

    /// Convierte carpetas o muchas imágenes en segundo plano, con las
    /// subcarpetas, sin sobrescribir nada, y vuelve enseguida con un id. Por
    /// defecto, WebP con la nota 80, en una carpeta junto a la de entrada
    /// (fotos-webp, o fotos-apolo con varios formatos); con imágenes sueltas,
    /// en una subcarpeta junto a ellas (webp/, apolo/). Uno a la vez. Después,
    /// ver_lote. Ejemplo: {"entradas": ["~/Fotos/viaje"], "formatos":
    /// ["webp", "avif"], "mas_ligero": true}
    #[tool(annotations(
        title = "Empezar un lote",
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    async fn empezar_lote(&self, Parameters(e): Parameters<EntradaLote>) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::empezar_lote(&cx, e)).await
    }

    /// Cómo va un lote: las imágenes nuevas desde «desde», y el resumen al
    /// acabar. Con esperar_segundos (hasta 25) espera a que haya algo nuevo,
    /// en vez de preguntar en bucle. Ejemplo: {"id": "l-1",
    /// "esperar_segundos": 20}
    #[tool(annotations(
        title = "Ver un lote",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn ver_lote(&self, Parameters(e): Parameters<EntradaVerLote>) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::ver_lote(&cx, e)).await
    }

    /// Para un lote. Lo ya convertido se queda. Ejemplo: {"id": "l-1"}
    #[tool(annotations(
        title = "Cancelar un lote",
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn cancelar_lote(
        &self,
        Parameters(e): Parameters<EntradaCancelarLote>,
    ) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::cancelar_lote(&cx, e)).await
    }

    /// Los presets guardados de Apolo (los mismos que la aplicación y la
    /// línea de órdenes), con su formato y su orden. Se usan con preset en
    /// optimizar_imagen y empezar_lote.
    #[tool(annotations(
        title = "Listar los presets",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn listar_presets(&self) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::listar_presets(&cx)).await
    }

    /// Qué versión de Apolo responde, desde dónde arranca (la aplicación, o
    /// la extensión de Claude Desktop cediéndole el turno) y los motores que
    /// lleva (libwebp, MozJPEG, oxipng, avifenc, cjxl). Para cuando pregunten
    /// por la versión o algo no cuadre con lo esperado.
    #[tool(annotations(
        title = "Acerca de Apolo",
        read_only_hint = true,
        destructive_hint = false,
        idempotent_hint = true,
        open_world_hint = false
    ))]
    async fn acerca_de_apolo(&self) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::acerca_de_apolo(&cx)).await
    }

    /// Guarda una combinación de formato y opciones como preset nuevo, para
    /// usarla en Apolo, en la línea de órdenes o aquí. No sobrescribe: si el
    /// nombre existe, es un error (cambiar o borrar presets se hace en
    /// Apolo). Ejemplo: {"nombre": "Web ligera", "formato": "avif",
    /// "nota_objetivo": 75, "ancho": 1920}
    #[tool(annotations(
        title = "Guardar un preset",
        read_only_hint = false,
        destructive_hint = false,
        idempotent_hint = false,
        open_world_hint = false
    ))]
    async fn guardar_preset(
        &self,
        Parameters(e): Parameters<EntradaGuardarPreset>,
    ) -> CallToolResult {
        let cx = self.cx.clone();
        bloqueante(move || herramientas::guardar_preset(&cx, e)).await
    }
}

#[tool_handler]
impl ServerHandler for ServidorApolo {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new("apolo", env!("CARGO_PKG_VERSION")).with_title("Apolo"),
            )
            .with_instructions(instrucciones())
    }
}
