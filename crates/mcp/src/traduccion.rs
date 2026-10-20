//! Las herramientas en inglés (ADR 0025).
//!
//! Las descripciones de las herramientas y de sus argumentos salen de los
//! comentarios `///` del código (rmcp y schemars), y se quedan en español: son
//! también la documentación del código y del catálogo. En inglés se cambian al
//! servirlas, con esta tabla. La clave es el texto en español con los espacios
//! juntados (un comentario de varias líneas llega con saltos), y una prueba
//! exige que cada descripción tenga la suya y que no sobre ninguna.
//!
//! Los nombres de herramientas y argumentos no se traducen: son
//! identificadores, y salen tal cual en los ejemplos.

use apolo_nucleo::idioma::{self, Idioma};
use rmcp::model::Tool;
use serde_json::Value;
use std::sync::Arc;

/// Español (con los espacios juntados) → inglés.
static TABLA: &[(&str, &str)] = &[
    // ─── Títulos ───
    (
        "Analizar una imagen o carpeta",
        "Analyze an image or folder",
    ),
    ("Optimizar una imagen", "Optimize an image"),
    (
        "Comparar formatos a igual calidad",
        "Compare formats at equal quality",
    ),
    ("Medir la calidad", "Measure quality"),
    ("Ver una imagen", "View an image"),
    ("Empezar un lote", "Start a batch"),
    ("Ver un lote", "View a batch"),
    ("Cancelar un lote", "Cancel a batch"),
    ("Listar los presets", "List the presets"),
    ("Acerca de Apolo", "About Apolo"),
    ("Guardar un preset", "Save a preset"),
    // ─── Herramientas ───
    (
        r#"Qué es una imagen (formato, medidas, peso, transparencia, metadatos, orientación EXIF) o qué imágenes hay en una carpeta. Úsala primero si no sabes qué es. No mide la calidad: para eso, medir_calidad. Ejemplo: {"ruta": "~/Fotos/playa.heic"}"#,
        r#"What an image is (format, dimensions, size, transparency, metadata, EXIF orientation) or which images a folder holds. Use it first if you don't know what it is. It doesn't measure quality: for that, medir_calidad. Example: {"ruta": "~/Photos/beach.heic"}"#,
    ),
    (
        r#"Optimiza una imagen y la guarda como un fichero NUEVO (junto a la original, o en salida; si el nombre existe, -2, -3…). Sin decir calidad, busca la más baja que da una nota SSIMULACRA 2 de 80. Da la orden equivalente de cwebp, cjpeg, oxipng, avifenc o cjxl. Para elegir entre formatos, comparar_formatos; para muchas imágenes, empezar_lote. Ejemplos: {"ruta": "~/a.jpg"} · {"ruta": "~/a.png", "formato": "avif", "nota_objetivo": 85} · {"ruta": "~/a.jpg", "formato": "webp", "orden": "-q 80 -m 6", "ancho": 1600} Para las imágenes de la persona, úsala en vez de convertirlas tú en otro entorno (un intérprete en la nube): trabaja con el fichero de su disco, con los motores oficiales. Apolo no ve las imágenes adjuntas en la conversación: si te adjuntan una, pide su ruta."#,
        r#"Optimizes an image and saves it as a NEW file (next to the original, or in salida; if the name exists, -2, -3…). Without a quality, it searches for the lowest one that gives a SSIMULACRA 2 score of 80. Gives the equivalent command for cwebp, cjpeg, oxipng, avifenc or cjxl. To choose between formats, comparar_formatos; for many images, empezar_lote. Examples: {"ruta": "~/a.jpg"} · {"ruta": "~/a.png", "formato": "avif", "nota_objetivo": 85} · {"ruta": "~/a.jpg", "formato": "webp", "orden": "-q 80 -m 6", "ancho": 1600} For the person's images, use it instead of converting them yourself in another environment (a cloud interpreter): it works with the file on their disk, with the official engines. Apolo doesn't see images attached to the conversation: if one is attached, ask for its path."#,
    ),
    (
        r#"Codifica la misma imagen en varios formatos a la misma calidad percibida (nota SSIMULACRA 2, 80 por defecto) y dice cuál pesa menos. Guarda el mejor (por defecto), todos o ninguno, siempre como ficheros nuevos. Tarda: unas 7 pruebas por formato (con una foto de 12 MP, más de un minuto; con 24 MP, unos tres). Con fotos de más de unos 20 MP, mejor empezar_lote con esa imagen, los formatos y mas_ligero: hace lo mismo en segundo plano y no se corta por tiempo, aunque solo da el que gana. Para una sola salida ya decidida, optimizar_imagen. Ejemplo: {"ruta": "~/a.png", "formatos": ["webp", "avif", "jxl"]}"#,
        r#"Encodes the same image in several formats at the same perceived quality (SSIMULACRA 2 score, 80 by default) and says which one is the lightest. Saves the best one (by default), all or none, always as new files. It takes a while: about 7 trials per format (with a 12 MP photo, over a minute; with 24 MP, about three). With photos over about 20 MP, better use empezar_lote with that image, the formats and mas_ligero: it does the same in the background and isn't cut off by a timeout, although it only gives the winner. For a single output already decided, optimizar_imagen. Example: {"ruta": "~/a.png", "formatos": ["webp", "avif", "jxl"]}"#,
    ),
    (
        r#"Cuánto se aleja una imagen de su original: SSIMULACRA 2 (90 o más no se distingue, 80 muy alta, 70 alta, 50 se nota), PSNR y SSIM. Tienen que medir lo mismo (si el resultado está enderezado, se endereza el original). Para ver dónde está la pérdida, ver_imagen con diferencias_con. Ejemplo: {"original": "~/a.jpg", "resultado": "~/a.webp"}"#,
        r#"How far an image is from its original: SSIMULACRA 2 (90 or more indistinguishable, 80 very high, 70 high, 50 noticeable), PSNR and SSIM. They have to be the same size (if the result is auto-rotated, the original is auto-rotated too). To see where the loss is, ver_imagen with diferencias_con. Example: {"original": "~/a.jpg", "resultado": "~/a.webp"}"#,
    ),
    (
        r#"Devuelve la imagen para mirarla: reducida (768 px por defecto), una zona a tamaño real para ver defectos, o el mapa de diferencias con su original (diferencias_con). Cada vista gasta contexto: pídela solo cuando ayude. Ejemplos: {"ruta": "~/a.webp"} · {"ruta": "~/a.webp", "diferencias_con": "~/a.jpg"} · {"ruta": "~/a.webp", "zona": {"x": 400, "y": 300, "ancho": 256, "alto": 256}}"#,
        r#"Returns the image to look at: scaled down (768 px by default), an area at full size to spot defects, or the map of differences from its original (diferencias_con). Each view uses up context: ask for one only when it helps. Examples: {"ruta": "~/a.webp"} · {"ruta": "~/a.webp", "diferencias_con": "~/a.jpg"} · {"ruta": "~/a.webp", "zona": {"x": 400, "y": 300, "ancho": 256, "alto": 256}}"#,
    ),
    (
        r#"Convierte carpetas o muchas imágenes en segundo plano, con las subcarpetas, sin sobrescribir nada, y vuelve enseguida con un id. Por defecto, WebP con la nota 80, en una carpeta junto a la de entrada (fotos-webp, o fotos-apolo con varios formatos); con imágenes sueltas, en una subcarpeta junto a ellas (webp/, apolo/). Uno a la vez. Después, ver_lote. Ejemplo: {"entradas": ["~/Fotos/viaje"], "formatos": ["webp", "avif"], "mas_ligero": true}"#,
        r#"Converts folders or many images in the background, subfolders included, without overwriting anything, and comes back right away with an id. By default, WebP at score 80, in a folder next to the input one (photos-webp, or photos-apolo with several formats); with loose images, in a subfolder next to them (webp/, apolo/). One at a time. Afterwards, ver_lote. Example: {"entradas": ["~/Photos/trip"], "formatos": ["webp", "avif"], "mas_ligero": true}"#,
    ),
    (
        r#"Cómo va un lote: las imágenes nuevas desde «desde», y el resumen al acabar. Con esperar_segundos (hasta 25) espera a que haya algo nuevo, en vez de preguntar en bucle. Ejemplo: {"id": "l-1", "esperar_segundos": 20}"#,
        r#"How a batch is going: the new images since “desde”, and the summary when it ends. With esperar_segundos (up to 25) it waits until there is something new, instead of asking in a loop. Example: {"id": "l-1", "esperar_segundos": 20}"#,
    ),
    (
        r#"Para un lote. Lo ya convertido se queda. Ejemplo: {"id": "l-1"}"#,
        r#"Stops a batch. What has already been converted stays. Example: {"id": "l-1"}"#,
    ),
    (
        "Los presets guardados de Apolo (los mismos que la aplicación y la línea de órdenes), con su formato y su orden. Se usan con preset en optimizar_imagen y empezar_lote.",
        "Apolo's saved presets (the same ones as the app and the command line), with their format and their command. They are used with preset in optimizar_imagen and empezar_lote.",
    ),
    (
        "Qué versión de Apolo responde, desde dónde arranca (la aplicación, o la extensión de Claude Desktop cediéndole el turno) y los motores que lleva (libwebp, MozJPEG, oxipng, avifenc, cjxl). Para cuando pregunten por la versión o algo no cuadre con lo esperado.",
        "Which version of Apolo is answering, where it starts from (the app, or the Claude Desktop extension handing over to it) and the engines it carries (libwebp, MozJPEG, oxipng, avifenc, cjxl). For when someone asks about the version or something doesn't match what was expected.",
    ),
    (
        r#"Guarda una combinación de formato y opciones como preset nuevo, para usarla en Apolo, en la línea de órdenes o aquí. No sobrescribe: si el nombre existe, es un error (cambiar o borrar presets se hace en Apolo). Ejemplo: {"nombre": "Web ligera", "formato": "avif", "nota_objetivo": 75, "ancho": 1920}"#,
        r#"Saves a combination of format and options as a new preset, to use in Apolo, on the command line or here. It doesn't overwrite: if the name exists, it's an error (changing or deleting presets is done in Apolo). Example: {"nombre": "Light web", "formato": "avif", "nota_objetivo": 75, "ancho": 1920}"#,
    ),
    // ─── Argumentos ───
    (
        "Ruta absoluta de una imagen o de una carpeta (se admite ~).",
        "Absolute path of an image or a folder (~ is accepted).",
    ),
    (
        "Ruta absoluta de la imagen (se admite ~).",
        "Absolute path of the image (~ is accepted).",
    ),
    (
        "Ruta absoluta de la imagen a ver.",
        "Absolute path of the image to view.",
    ),
    (
        "Carpetas o imágenes, rutas absolutas (se admite ~).",
        "Folders or images, absolute paths (~ is accepted).",
    ),
    (
        "La imagen original, ruta absoluta.",
        "The original image, absolute path.",
    ),
    (
        "La imagen a medir, del mismo tamaño.",
        "The image to measure, of the same size.",
    ),
    (
        "webp, jpeg, png, qoi, avif o jxl. Por defecto, el del preset, o webp.",
        "webp, jpeg, png, qoi, avif or jxl. By default, the preset's, or webp.",
    ),
    (
        "Un preset guardado (listar_presets los da). Es la base de lo demás.",
        "A saved preset (listar_presets lists them). It is the base for everything else.",
    ),
    (
        "Un preset guardado, base de todo.",
        "A saved preset, the base for everything.",
    ),
    (
        "Opciones de la herramienta oficial, tal cual: «-q 80 -m 6» (cwebp), «-quality 85 -progressive» (cjpeg), «-o 4» (oxipng), «-q 60 -s 6» (avifenc), «-d 1 -e 7» (cjxl). Puede empezar por el nombre de la herramienta. Para su formato, sustituye las opciones del preset.",
        "Options of the official tool, as they are: “-q 80 -m 6” (cwebp), “-quality 85 -progressive” (cjpeg), “-o 4” (oxipng), “-q 60 -s 6” (avifenc), “-d 1 -e 7” (cjxl). It may start with the tool's name. For its format, it replaces the preset's options.",
    ),
    (
        "Opciones de la herramienta oficial (solo con un formato).",
        "Options of the official tool (only with one format).",
    ),
    (
        "Calidad fija de 0 a 100. No a la vez que nota_objetivo.",
        "Fixed quality from 0 to 100. Not together with nota_objetivo.",
    ),
    (
        "Buscar la calidad más baja que da esta nota SSIMULACRA 2 (90 no se distingue; 80 muy alta; 70 alta; 50 media). Tarda unos segundos.",
        "Search for the lowest quality that gives this SSIMULACRA 2 score (90 indistinguishable; 80 very high; 70 high; 50 medium). Takes a few seconds.",
    ),
    (
        "Buscar, imagen a imagen, la calidad que da esta nota. Por defecto, 80.",
        "Search, image by image, for the quality that gives this score. 80 by default.",
    ),
    (
        "La nota SSIMULACRA 2 que tienen que alcanzar todos. Por defecto, 80.",
        "The SSIMULACRA 2 score they all have to reach. 80 by default.",
    ),
    (
        "Sin pérdida (WebP, AVIF, JPEG XL). PNG y QOI siempre lo son.",
        "Lossless (WebP, AVIF, JPEG XL). PNG and QOI always are.",
    ),
    (
        "Ancho máximo, en píxeles: reduce si es mayor y nunca agranda (con alto solo, proporcional).",
        "Maximum width, in pixels: scales down if larger and never enlarges (with alto alone, proportional).",
    ),
    (
        "Alto máximo, en píxeles: reduce si es mayor y nunca agranda (con ancho solo, proporcional).",
        "Maximum height, in pixels: scales down if larger and never enlarges (with ancho alone, proportional).",
    ),
    (
        "Girar según la orientación EXIF si la foto lo necesita. Por defecto, sí.",
        "Auto-rotate according to the EXIF orientation if the photo needs it. Yes by default.",
    ),
    (
        "Girar cada foto según su orientación EXIF si lo necesita. Por defecto, sí.",
        "Auto-rotate each photo according to its EXIF orientation if it needs it. Yes by default.",
    ),
    (
        "Quedarse con este rectángulo, antes de redimensionar (en píxeles de la imagen ya enderezada; lo que se salga del borde se ajusta).",
        "Crop to this rectangle, before resizing (in pixels of the already auto-rotated image; whatever goes past the edge is trimmed).",
    ),
    (
        "Desde la izquierda, en píxeles.",
        "From the left, in pixels.",
    ),
    ("Desde arriba, en píxeles.", "From the top, in pixels."),
    (
        "Reducir a este número de colores (de 2 a 256), con tramado: sobre todo para PNG de logos, iconos y capturas.",
        "Reduce the palette to this number of colors (2 to 256), with dithering: mainly for PNGs of logos, icons and screenshots.",
    ),
    (
        "Reducir a este número de colores (de 2 a 256), con tramado.",
        "Reduce the palette to this number of colors (2 to 256), with dithering.",
    ),
    (
        "Dónde dejarla: una carpeta, o un fichero con la extensión del formato. Por defecto, junto a la original. Nunca se sobrescribe nada.",
        "Where to put it: a folder, or a file with the format's extension. By default, next to the original. Nothing is ever overwritten.",
    ),
    (
        "Medir también PSNR y SSIM (la nota ya se mide al buscarla).",
        "Also measure PSNR and SSIM (the score is already measured when searching for it).",
    ),
    (
        "Los formatos a comparar. Por defecto, webp, avif y jpeg.",
        "The formats to compare. By default, webp, avif and jpeg.",
    ),
    (
        "Qué guardar: mejor (por defecto), todos o ninguno.",
        "What to save: mejor (the best one, by default), todos (all) or ninguno (none).",
    ),
    (
        "Solo el más ligero de los que llegan a la nota.",
        "Only the lightest of those that reach the score.",
    ),
    (
        "Todos, cada uno en su fichero.",
        "All of them, each in its own file.",
    ),
    ("Ninguno: solo comparar.", "None: just compare."),
    (
        "Carpeta donde guardar. Por defecto, junto a la original.",
        "Folder to save to. By default, next to the original.",
    ),
    (
        "El lado mayor de la vista, en píxeles: 768 por defecto, como mucho 1568.",
        "The longest side of the view, in pixels: 768 by default, 1568 at most.",
    ),
    (
        "Ver solo esta zona, en píxeles de la imagen (para mirar defectos a 1:1).",
        "View only this area, in pixels of the image (to look at defects at 1:1).",
    ),
    (
        "Ruta del original: en vez de la imagen, el mapa de dónde se aleja de él.",
        "Path of the original: instead of the image, the map of where it departs from it.",
    ),
    (
        "Con diferencias_con: estructura (por defecto) o diferencia.",
        "With diferencias_con: estructura (structure, by default) or diferencia (difference).",
    ),
    (
        "Dónde cambian los píxeles, aunque sea poco.",
        "Where the pixels change, even slightly.",
    ),
    (
        "Dónde se pierde detalle (1 − SSIM local): lo que el ojo nota.",
        "Where detail is lost (1 − local SSIM): what the eye notices.",
    ),
    (
        "Formatos de salida (uno o varios). Por defecto, el del preset o webp.",
        "Output formats (one or several). By default, the preset's or webp.",
    ),
    (
        "Con varios formatos: guardar de cada imagen solo el que menos pese.",
        "With several formats: keep only the lightest one of each image.",
    ),
    (
        "Medir la nota de cada fichero (tarda más).",
        "Measure the score of each file (takes longer).",
    ),
    (
        "Carpeta de salida. Por defecto, junto a la de entrada con -webp, -apolo…; con imágenes sueltas, una subcarpeta webp/, apolo/… junto a ellas.",
        "Output folder. By default, next to the input one with -webp, -apolo…; with loose images, a webp/, apolo/… subfolder next to them.",
    ),
    (
        "El id que dio empezar_lote, como «l-3».",
        "The id that empezar_lote gave, like “l-3”.",
    ),
    (
        "Desde qué imagen contar las nuevas (siguiente_desde de la respuesta anterior). Por defecto, 0: todas.",
        "From which image to count the new ones (siguiente_desde from the previous answer). 0 by default: all of them.",
    ),
    (
        "Esperar hasta este número de segundos (como mucho 25) a que haya algo nuevo o acabe, en vez de preguntar en bucle.",
        "Wait up to this number of seconds (25 at most) for something new or for the end, instead of asking in a loop.",
    ),
    (
        "El nombre del preset nuevo. No puede existir ya.",
        "The name of the new preset. It can't exist already.",
    ),
];

/// Junta los espacios: un comentario de varias líneas llega con saltos.
fn junto(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// La versión en inglés de un texto de la tabla.
pub fn al_ingles(es: &str) -> Option<&'static str> {
    let clave = junto(es);
    TABLA.iter().find(|(e, _)| *e == clave).map(|(_, en)| *en)
}

/// El texto en el idioma actual: en español, tal cual; en inglés, el de la
/// tabla (o el español si falta, que la prueba no deja pasar).
fn traducir(es: &str) -> Option<&'static str> {
    match idioma::actual() {
        Idioma::Es => None,
        Idioma::En => al_ingles(es),
    }
}

/// Cambia cada `description` y `title` de un esquema, a cualquier
/// profundidad (`properties`, `$defs`, `items`, `oneOf`…). Las claves de
/// `properties` son nombres de argumentos y su valor es un objeto, así que un
/// argumento que se llamara `title` no se toca.
fn traducir_esquema(v: &mut Value) {
    match v {
        Value::Object(m) => {
            for (k, x) in m.iter_mut() {
                match x {
                    Value::String(s) if k == "description" || k == "title" => {
                        if let Some(en) = traducir(s) {
                            *s = en.to_string();
                        }
                    }
                    _ => traducir_esquema(x),
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(traducir_esquema),
        _ => {}
    }
}

fn traducir_objeto(o: &mut Arc<serde_json::Map<String, Value>>) {
    let mut v = Value::Object(o.as_ref().clone());
    traducir_esquema(&mut v);
    if let Value::Object(m) = v {
        *o = Arc::new(m);
    }
}

/// Una herramienta en el idioma actual: descripción, títulos y esquemas.
pub fn herramienta(mut h: Tool) -> Tool {
    if idioma::actual() == Idioma::Es {
        return h;
    }
    if let Some(d) = h.description.as_deref().and_then(traducir) {
        h.description = Some(d.into());
    }
    if let Some(t) = h.title.as_deref().and_then(traducir) {
        h.title = Some(t.into());
    }
    if let Some(a) = h.annotations.as_mut()
        && let Some(t) = a.title.as_deref().and_then(traducir)
    {
        a.title = Some(t.into());
    }
    traducir_objeto(&mut h.input_schema);
    if let Some(o) = h.output_schema.as_mut() {
        traducir_objeto(o);
    }
    h
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::ServidorApolo;

    /// Cada texto en español de las herramientas: descripciones y títulos.
    fn textos(v: &Value, fuera: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                for (k, x) in m {
                    match x {
                        Value::String(s) if k == "description" || k == "title" => {
                            fuera.push(s.clone())
                        }
                        _ => textos(x, fuera),
                    }
                }
            }
            Value::Array(a) => a.iter().for_each(|x| textos(x, fuera)),
            _ => {}
        }
    }

    fn en_espanol() -> Vec<String> {
        let h = idioma::en_este_hilo(Idioma::Es, ServidorApolo::herramientas);
        let mut v = Vec::new();
        textos(&serde_json::to_value(&h).unwrap(), &mut v);
        v
    }

    #[test]
    fn cada_descripcion_tiene_su_traduccion_y_no_sobra_ninguna() {
        let es = en_espanol();
        assert!(es.len() > 60, "{}", es.len());
        let faltan: Vec<String> = es
            .iter()
            .map(|s| junto(s))
            .filter(|s| al_ingles(s).is_none())
            .collect();
        assert!(
            faltan.is_empty(),
            "sin traducción en traduccion.rs: {faltan:#?}"
        );
        let juntos: Vec<String> = es.iter().map(|s| junto(s)).collect();
        let sobran: Vec<&str> = TABLA
            .iter()
            .map(|(e, _)| *e)
            .filter(|e| !juntos.iter().any(|s| s == e))
            .collect();
        assert!(
            sobran.is_empty(),
            "en la tabla y ya no en el código: {sobran:#?}"
        );
    }

    #[test]
    fn en_ingles_no_queda_nada_en_espanol() {
        let es = en_espanol();
        let h = idioma::en_este_hilo(Idioma::En, ServidorApolo::herramientas);
        let mut en = Vec::new();
        textos(&serde_json::to_value(&h).unwrap(), &mut en);
        assert_eq!(es.len(), en.len());
        for (e, i) in es.iter().zip(&en) {
            assert_ne!(junto(e), junto(i), "sin traducir: {e}");
            assert_eq!(Some(i.as_str()), al_ingles(e));
        }
        // Ni rastro de las comillas españolas.
        assert!(en.iter().all(|s| !s.contains('«')), "{en:#?}");
    }
}
