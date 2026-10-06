# Cobertura de cwebp

Última actualización: **2026-10-06** (tarde, entrega 2)

Cada opción de `cwebp` 1.6.0 con su sitio en Apolo. **Una opción no está hecha hasta que su fila
está completa**: el campo de libwebp que toca, la opción de `apolo webp`, el control de la interfaz
y la prueba que la vigila. La referencia es `cwebp -longhelp` y `examples/cwebp.c` de libwebp 1.6.0.

- **CLI**: ✓ si `apolo webp` la acepta con el mismo efecto.
- **Interfaz**: ✓ y la sección del panel del Estudio donde está: **B** (Básico), **A** (Avanzado) o
  **E** (Experto). El panel se pinta desde `frontend/src/estudio/opciones.ts`.
- **Prueba**: **eq** si está en la lista de `crates/nucleo/tests/equivalencia_cwebp.rs`, que
  compara el fichero con el del cwebp oficial **byte a byte** sobre 29 imágenes; **ida** si la cubre
  la prueba de ida y vuelta preset ⇄ orden (`cwebp::pruebas::ida_y_vuelta`).

| Opción | Qué hace | libwebp / núcleo | CLI | Interfaz | Prueba |
|---|---|---|---|---|---|
| `-q` | Calidad 0–100 con pérdida, o esfuerzo sin pérdida | `quality` · `calidad` | ✓ | ✓ B | eq, ida |
| `-alpha_q` | Calidad de la transparencia | `alpha_quality` · `calidad_alfa` | ✓ | ✓ A | eq, ida |
| `-preset` | Punto de partida; reescribe todo lo anterior | `WebPConfigPreset` · `aplicar_preset` | ✓ | ✓ «Partir de» | eq (photo, drawing, icon, text), ida |
| `-z` | Nivel sin pérdida 0–9; lo anulan `-q` y `-m` | `WebPConfigLosslessPreset` · `aplicar_nivel_sin_perdida` | ✓ | ✓ A | eq (0 y 9), ida |
| `-m` | Método 0–6 | `method` · `metodo` | ✓ | ✓ A | eq, ida |
| `-segments` | Segmentos 1–4 | `segments` · `segmentos` | ✓ | ✓ E | eq, ida |
| `-size` | Tamaño objetivo en bytes (fuerza 6 pasadas si hay 1) | `target_size` · `tamano_objetivo` | ✓ | ✓ A | eq, ida |
| `-psnr` | PSNR objetivo | `target_PSNR` · `psnr_objetivo` | ✓ | ✓ E | eq, ida |
| `-s` | La entrada es YUV 4:2:0 crudo de ese tamaño | `ReadYUV` · `entrada::leer_yuv` | ✓ | — | eq (`yuv.yuv`) |
| `-sns` | Modelado espacial del ruido 0–100 | `sns_strength` · `sns` | ✓ | ✓ E | eq, ida |
| `-f` | Fuerza del filtro 0–100 | `filter_strength` · `fuerza_filtro` | ✓ | ✓ E | eq, ida |
| `-sharpness` | Nitidez del filtro 0–7 | `filter_sharpness` · `nitidez_filtro` | ✓ | ✓ E | eq, ida |
| `-strong` / `-nostrong` | Filtro fuerte o simple | `filter_type` · `filtro_fuerte` | ✓ | ✓ E | eq (`-nostrong`; `-strong` es el valor por defecto), ida |
| `-sharp_yuv` | RGB→YUV más nítido | `use_sharp_yuv` · `yuv_nitido` | ✓ | ✓ A | eq, ida |
| `-partition_limit` | Límite para que quepa la partición 0 | `partition_limit` · `limite_particion` | ✓ | ✓ E | eq, ida |
| `-pass` | Pasadas de análisis 1–10 | `pass` · `pasadas` | ✓ | ✓ E | eq, ida |
| `-qrange` | Calidad mínima y máxima | `qmin`/`qmax` · `calidad_minima`/`calidad_maxima` | ✓ | ✓ E | eq, ida |
| `-crop` | Recortar (con una vista sobre la misma picture) | `WebPPictureView` · `recorte` | ✓ | ✓ B | eq, ida |
| `-resize` | Redimensionar tras recortar; 0 = proporcional | `WebPPictureRescale` · `redimension` | ✓ | ✓ B | eq, ida |
| `-resize_mode` | `up_only`, `down_only`, `always` | `ApplyResizeMode` · `modo_redimension` | ✓ | ✓ A | eq (los dos modos), ida |
| `-mt` | Hilos; cada `-mt` sube un nivel | `thread_level` · `hilos` | ✓ | ✓ E | eq, ida |
| `-low_memory` | Menos memoria | `low_memory` · `poca_memoria` | ✓ | ✓ E | eq, ida |
| `-map` | Mapa por macrobloque | `extra_info` · `Extras::mapa` | ✓ | — (entrega 5, con las métricas) | — (salida de consola) |
| `-print_psnr` / `-print_ssim` / `-print_lsim` | Distorsión media | `WebPPictureDistortion` · `Extras::medir` | ✓ | PSNR en la barra de estado | — (salida de consola) |
| `-d` | Volcar el resultado como PGM | `DumpPicture` · `Extras::volcar` | ✓ | — | — |
| `-alpha_method` | Compresión de la transparencia 0–1 | `alpha_compression` · `compresion_alfa` | ✓ | ✓ E | eq, ida |
| `-alpha_filter` | `none`, `fast`, `best` | `alpha_filtering` · `filtrado_alfa` | ✓ | ✓ E | eq (none, best), ida |
| `-alpha_cleanup` | Obsoleta: lo contrario de `-exact` | `exact = 0` | ✓ | — | ida |
| `-exact` | Conservar el RGB bajo lo transparente (y redimensionar sin premultiplicar) | `exact` · `exacto` | ✓ | ✓ A | eq, ida |
| `-blend_alpha` | Mezclar con un color de fondo | `WebPBlendAlpha` · `mezclar_alfa` | ✓ | ✓ A | eq, ida |
| `-noalpha` | Descartar la transparencia | lectores · `sin_alfa` | ✓ | ✓ A | eq, ida |
| `-lossless` | Sin pérdida | `lossless` · `sin_perdida` | ✓ | ✓ B | eq, ida |
| `-near_lossless` | Casi sin pérdida 0–100 (activa sin pérdida) | `near_lossless` · `casi_sin_perdida` | ✓ | ✓ A | eq, ida |
| `-hint` | `photo`, `picture`, `graph` | `image_hint` · `pista` | ✓ | ✓ E | eq (graph), ida |
| `-metadata` | `all`, `none`, `exif`, `icc`, `xmp` | `WriteWebPWithMetadata` · `metadatos::escribir` | ✓ | ✓ A | eq (all; icc,xmp), ida |
| `-jpeg_like` | Tamaño parecido al de un JPEG | `emulate_jpeg_size` · `emular_jpeg` | ✓ | ✓ E | eq, ida |
| `-af` | Filtro automático | `autofilter` · `autofiltro` | ✓ | ✓ A | eq, ida |
| `-pre` | Preprocesado (experimental) | `preprocessing` · `preprocesado` | ✓ | ✓ E | eq, ida |
| `-short` / `-quiet` / `-v` / `-progress` | Salida del programa | `progress_hook` · `Progreso` | ✓ | indicador «codificando» | — |
| `-version` | Versión de libwebp | `WebPGetEncoderVersion` | ✓ | Ajustes | `libwebp_esta_enlazada` |
| `-noasm` | Desactivar SIMD | `VP8GetCPUInfo` | se acepta y **se ignora** | — | — |

## Propias de Apolo

No son de cwebp. Llevan el prefijo `-apolo_` para no chocar nunca con una opción suya, y con ellas la
interfaz avisa de que la orden cwebp equivalente ya no da el mismo fichero.

| Opción | Qué hace | Núcleo | CLI | Interfaz | Prueba |
|---|---|---|---|---|---|
| `-apolo_enderezar` | Girar según la orientación EXIF y dejarla a 1 ([ADR 0012](adr/0012-enderezar-como-opcion.md)) | `orientacion::enderezar` · `enderezar` | ✓ | ✓ B, y aviso si la foto viene girada | `las_ocho_orientaciones`, `enderezar_gira_y_deja_el_exif_a_1`, e2e |
| `-apolo_preset <nombre>` | Partir de un preset guardado | `presets::cargar` · `cwebp::leer_desde` | ✓ | «Partir de» → Guardados | `guardar_listar_cargar_borrar`, e2e |

## Lo que no se puede expresar con cwebp

Dos campos de `WebPConfig` no tienen opción en cwebp: `partitions` y `use_delta_palette`. **Apolo
tampoco los ofrece**: si los ofreciera, habría ficheros de Apolo que ninguna orden cwebp puede
reproducir, y la orden equivalente que enseña la interfaz dejaría de serlo.

## Cómo lee cwebp, y cómo se repite

Los lectores de entrada deciden los píxeles de partida; si no son idénticos, la salida tampoco. Lo
que se copia de `imageio/` está en [ADR 0011](adr/0011-leer-como-cwebp.md).
