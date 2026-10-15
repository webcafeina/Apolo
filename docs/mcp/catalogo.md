# Las herramientas de apolo-mcp

> Generado desde el código (`crates/mcp`) con `make catalogo-mcp`. No se edita a mano: una
> prueba falla si se queda atrás. Las decisiones, en el [ADR 0023](../adr/0023-mcp.md).

## Lo que lee el agente al conectarse

Apolo optimiza imágenes en este ordenador con los mismos motores que cwebp, MozJPEG (cjpeg), oxipng, avifenc y cjxl, y da el mismo fichero que esas herramientas. Lee cualquier ruta que le des y siempre escribe ficheros nuevos: nunca sobrescribe ni borra (si el nombre existe, añade -2, -3…). Flujo habitual: analizar_imagen → optimizar_imagen (o comparar_formatos para elegir formato a igual calidad) → ver_imagen o medir_calidad para comprobar. Para carpetas o más de unas 5 imágenes, empezar_lote y ver_lote. La calidad percibida se mide con SSIMULACRA 2: 90 o más no se distingue del original, 80 es muy alta (no se distingue lado a lado), 70 alta, 50 se nota. Sin decir calidad, Apolo busca la más baja que da una nota de 80. Las rutas, absolutas o con ~. Apolo corre en este mismo ordenador, con la cuenta de quien te habla: sus rutas (/Users/…, C:\Users\…) son las suyas. Solo trabaja con ficheros del disco y no ve las imágenes adjuntas en la conversación: si te adjuntan una, pide su ruta o búscala en el disco. Si piden sobrescribir o borrar un original, Apolo no lo hace; antes de hacerlo con otras herramientas, avisa de que el original se perderá sin vuelta atrás, pide confirmación y propón guardarlo aparte. Responde en el idioma de quien pregunta.

## `acerca_de_apolo` — Acerca de Apolo

*solo lee · no destruye · idempotente · todo en este ordenador*

Qué versión de Apolo responde, desde dónde arranca (la aplicación, o la extensión de Claude Desktop cediéndole el turno) y los motores que lleva (libwebp, MozJPEG, oxipng, avifenc, cjxl). Para cuando pregunten por la versión o algo no cuadre con lo esperado.

Sin argumentos.

## `analizar_imagen` — Analizar una imagen o carpeta

*solo lee · no destruye · idempotente · todo en este ordenador*

Qué es una imagen (formato, medidas, peso, transparencia, metadatos, orientación EXIF) o qué imágenes hay en una carpeta. Úsala primero si no sabes qué es. No mide la calidad: para eso, medir_calidad. Ejemplo: {"ruta": "~/Fotos/playa.heic"}

| Argumento | Tipo | Qué es |
|---|---|---|
| `ruta` | texto | **Obligatorio.** Ruta absoluta de una imagen o de una carpeta (se admite ~). |

## `cancelar_lote` — Cancelar un lote

*hace cambios · no destruye · idempotente · todo en este ordenador*

Para un lote. Lo ya convertido se queda. Ejemplo: {"id": "l-1"}

| Argumento | Tipo | Qué es |
|---|---|---|
| `id` | texto | **Obligatorio.** El id que dio empezar_lote, como «l-3». |

## `comparar_formatos` — Comparar formatos a igual calidad

*hace cambios · no destruye · todo en este ordenador*

Codifica la misma imagen en varios formatos a la misma calidad percibida (nota SSIMULACRA 2, 80 por defecto) y dice cuál pesa menos. Guarda el mejor (por defecto), todos o ninguno, siempre como ficheros nuevos. Tarda: unas 7 pruebas por formato (con una foto de 12 MP, más de un minuto). Para una sola salida ya decidida, optimizar_imagen. Ejemplo: {"ruta": "~/a.png", "formatos": ["webp", "avif", "jxl"]}

| Argumento | Tipo | Qué es |
|---|---|---|
| `ruta` | texto | **Obligatorio.** Ruta absoluta de la imagen (se admite ~). |
| `alto` | entero | Alto máximo, en píxeles: reduce si es mayor y nunca agranda (con ancho solo, proporcional). |
| `ancho` | entero | Ancho máximo, en píxeles: reduce si es mayor y nunca agranda (con alto solo, proporcional). |
| `enderezar` | sí/no | Girar según la orientación EXIF si la foto lo necesita. Por defecto, sí. |
| `formatos` | lista de webp \| jpeg \| png \| qoi \| avif \| jxl | Los formatos a comparar. Por defecto, webp, avif y jpeg. |
| `guardar` | mejor \| todos \| ninguno | Qué guardar: mejor (por defecto), todos o ninguno. |
| `nota_objetivo` | número (0–100) | La nota SSIMULACRA 2 que tienen que alcanzar todos. Por defecto, 80. |
| `salida` | texto | Carpeta donde guardar. Por defecto, junto a la original. |

## `empezar_lote` — Empezar un lote

*hace cambios · no destruye · todo en este ordenador*

Convierte carpetas o muchas imágenes en segundo plano, con las subcarpetas, sin sobrescribir nada, y vuelve enseguida con un id. Por defecto, WebP con la nota 80, en una carpeta junto a la de entrada (fotos-webp, o fotos-apolo con varios formatos). Uno a la vez. Después, ver_lote. Ejemplo: {"entradas": ["~/Fotos/viaje"], "formatos": ["webp", "avif"], "mas_ligero": true}

| Argumento | Tipo | Qué es |
|---|---|---|
| `entradas` | lista de texto | **Obligatorio.** Carpetas o imágenes, rutas absolutas (se admite ~). |
| `alto` | entero | Alto máximo, en píxeles: reduce si es mayor y nunca agranda (con ancho solo, proporcional). |
| `ancho` | entero | Ancho máximo, en píxeles: reduce si es mayor y nunca agranda (con alto solo, proporcional). |
| `calidad` | entero (0–100) | Calidad fija de 0 a 100. No a la vez que nota_objetivo. |
| `enderezar` | sí/no | Girar cada foto según su orientación EXIF si lo necesita. Por defecto, sí. |
| `formatos` | lista de webp \| jpeg \| png \| qoi \| avif \| jxl | Formatos de salida (uno o varios). Por defecto, el del preset o webp. |
| `mas_ligero` | sí/no | Con varios formatos: guardar de cada imagen solo el que menos pese. |
| `medir` | sí/no | Medir la nota de cada fichero (tarda más). |
| `nota_objetivo` | número (0–100) | Buscar, imagen a imagen, la calidad que da esta nota. Por defecto, 80. |
| `orden` | texto | Opciones de la herramienta oficial (solo con un formato). |
| `preset` | texto | Un preset guardado, base de todo. |
| `salida` | texto | Carpeta de salida. Por defecto, junto a la de entrada con -webp, -apolo… |
| `sin_perdida` | sí/no | Sin pérdida (WebP, AVIF, JPEG XL). PNG y QOI siempre lo son. |

## `guardar_preset` — Guardar un preset

*hace cambios · no destruye · todo en este ordenador*

Guarda una combinación de formato y opciones como preset nuevo, para usarla en Apolo, en la línea de órdenes o aquí. No sobrescribe: si el nombre existe, es un error (cambiar o borrar presets se hace en Apolo). Ejemplo: {"nombre": "Web ligera", "formato": "avif", "nota_objetivo": 75, "ancho": 1920}

| Argumento | Tipo | Qué es |
|---|---|---|
| `nombre` | texto | **Obligatorio.** El nombre del preset nuevo. No puede existir ya. |
| `alto` | entero | Alto máximo, en píxeles: reduce si es mayor y nunca agranda (con ancho solo, proporcional). |
| `ancho` | entero | Ancho máximo, en píxeles: reduce si es mayor y nunca agranda (con alto solo, proporcional). |
| `calidad` | entero (0–100) | Calidad fija de 0 a 100. No a la vez que nota_objetivo. |
| `enderezar` | sí/no | Girar según la orientación EXIF si la foto lo necesita. Por defecto, sí. |
| `formato` | webp \| jpeg \| png \| qoi \| avif \| jxl | webp, jpeg, png, qoi, avif o jxl. Por defecto, el del preset, o webp. |
| `nota_objetivo` | número (0–100) | Buscar la calidad más baja que da esta nota SSIMULACRA 2 (90 no se distingue; 80 muy alta; 70 alta; 50 media). Tarda unos segundos. |
| `orden` | texto | Opciones de la herramienta oficial, tal cual: «-q 80 -m 6» (cwebp), «-quality 85 -progressive» (cjpeg), «-o 4» (oxipng), «-q 60 -s 6» (avifenc), «-d 1 -e 7» (cjxl). Puede empezar por el nombre de la herramienta. Para su formato, sustituye las opciones del preset. |
| `preset` | texto | Un preset guardado (listar_presets los da). Es la base de lo demás. |
| `sin_perdida` | sí/no | Sin pérdida (WebP, AVIF, JPEG XL). PNG y QOI siempre lo son. |

## `listar_presets` — Listar los presets

*solo lee · no destruye · idempotente · todo en este ordenador*

Los presets guardados de Apolo (los mismos que la aplicación y la línea de órdenes), con su formato y su orden. Se usan con preset en optimizar_imagen y empezar_lote.

Sin argumentos.

## `medir_calidad` — Medir la calidad

*solo lee · no destruye · idempotente · todo en este ordenador*

Cuánto se aleja una imagen de su original: SSIMULACRA 2 (90 o más no se distingue, 80 muy alta, 70 alta, 50 se nota), PSNR y SSIM. Tienen que medir lo mismo (si el resultado está enderezado, se endereza el original). Para ver dónde está la pérdida, ver_imagen con diferencias_con. Ejemplo: {"original": "~/a.jpg", "resultado": "~/a.webp"}

| Argumento | Tipo | Qué es |
|---|---|---|
| `original` | texto | **Obligatorio.** La imagen original, ruta absoluta. |
| `resultado` | texto | **Obligatorio.** La imagen a medir, del mismo tamaño. |

## `optimizar_imagen` — Optimizar una imagen

*hace cambios · no destruye · todo en este ordenador*

Optimiza una imagen y la guarda como un fichero NUEVO (junto a la original, o en salida; si el nombre existe, -2, -3…). Sin decir calidad, busca la más baja que da una nota SSIMULACRA 2 de 80. Da la orden equivalente de cwebp, cjpeg, oxipng, avifenc o cjxl. Para elegir entre formatos, comparar_formatos; para muchas imágenes, empezar_lote. Ejemplos: {"ruta": "~/a.jpg"} · {"ruta": "~/a.png", "formato": "avif", "nota_objetivo": 85} · {"ruta": "~/a.jpg", "formato": "webp", "orden": "-q 80 -m 6", "ancho": 1600}

| Argumento | Tipo | Qué es |
|---|---|---|
| `ruta` | texto | **Obligatorio.** Ruta absoluta de la imagen (se admite ~). |
| `alto` | entero | Alto máximo, en píxeles: reduce si es mayor y nunca agranda (con ancho solo, proporcional). |
| `ancho` | entero | Ancho máximo, en píxeles: reduce si es mayor y nunca agranda (con alto solo, proporcional). |
| `calidad` | entero (0–100) | Calidad fija de 0 a 100. No a la vez que nota_objetivo. |
| `enderezar` | sí/no | Girar según la orientación EXIF si la foto lo necesita. Por defecto, sí. |
| `formato` | webp \| jpeg \| png \| qoi \| avif \| jxl | webp, jpeg, png, qoi, avif o jxl. Por defecto, el del preset, o webp. |
| `medir` | sí/no | Medir también PSNR y SSIM (la nota ya se mide al buscarla). |
| `nota_objetivo` | número (0–100) | Buscar la calidad más baja que da esta nota SSIMULACRA 2 (90 no se distingue; 80 muy alta; 70 alta; 50 media). Tarda unos segundos. |
| `orden` | texto | Opciones de la herramienta oficial, tal cual: «-q 80 -m 6» (cwebp), «-quality 85 -progressive» (cjpeg), «-o 4» (oxipng), «-q 60 -s 6» (avifenc), «-d 1 -e 7» (cjxl). Puede empezar por el nombre de la herramienta. Para su formato, sustituye las opciones del preset. |
| `preset` | texto | Un preset guardado (listar_presets los da). Es la base de lo demás. |
| `salida` | texto | Dónde dejarla: una carpeta, o un fichero con la extensión del formato. Por defecto, junto a la original. Nunca se sobrescribe nada. |
| `sin_perdida` | sí/no | Sin pérdida (WebP, AVIF, JPEG XL). PNG y QOI siempre lo son. |

## `ver_imagen` — Ver una imagen

*solo lee · no destruye · idempotente · todo en este ordenador*

Devuelve la imagen para mirarla: reducida (768 px por defecto), una zona a tamaño real para ver defectos, o el mapa de diferencias con su original (diferencias_con). Cada vista gasta contexto: pídela solo cuando ayude. Ejemplos: {"ruta": "~/a.webp"} · {"ruta": "~/a.webp", "diferencias_con": "~/a.jpg"} · {"ruta": "~/a.webp", "zona": {"x": 400, "y": 300, "ancho": 256, "alto": 256}}

| Argumento | Tipo | Qué es |
|---|---|---|
| `ruta` | texto | **Obligatorio.** Ruta absoluta de la imagen a ver. |
| `diferencias_con` | texto | Ruta del original: en vez de la imagen, el mapa de dónde se aleja de él. |
| `lado_max` | entero | El lado mayor de la vista, en píxeles: 768 por defecto, como mucho 1568. |
| `tipo_mapa` | diferencia \| estructura | Con diferencias_con: estructura (por defecto) o diferencia. |
| `zona` | {alto, ancho, x, y} | Ver solo esta zona, en píxeles de la imagen (para mirar defectos a 1:1). |

## `ver_lote` — Ver un lote

*solo lee · no destruye · idempotente · todo en este ordenador*

Cómo va un lote: las imágenes nuevas desde «desde», y el resumen al acabar. Con esperar_segundos (hasta 25) espera a que haya algo nuevo, en vez de preguntar en bucle. Ejemplo: {"id": "l-1", "esperar_segundos": 20}

| Argumento | Tipo | Qué es |
|---|---|---|
| `id` | texto | **Obligatorio.** El id que dio empezar_lote, como «l-3». |
| `desde` | entero | Desde qué imagen contar las nuevas (siguiente_desde de la respuesta anterior). Por defecto, 0: todas. |
| `esperar_segundos` | entero | Esperar hasta este número de segundos (como mucho 25) a que haya algo nuevo o acabe, en vez de preguntar en bucle. |
