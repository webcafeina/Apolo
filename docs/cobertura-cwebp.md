# Cobertura de cwebp

Última actualización: **2026-10-06**

Cada opción de `cwebp` con su sitio en Apolo. **Una opción no está hecha hasta que su fila está
completa**: el campo de libwebp que toca, la opción de `apolo webp`, el control de la interfaz y la
prueba que lo vigila. La referencia es `cwebp -longhelp` de la versión de libwebp fijada en
`crates/nucleo/Cargo.toml`.

Niveles de la interfaz: **B** Básico, **A** Avanzado, **E** Experto.

| Opción | Qué hace | libwebp | CLI | Interfaz | Nivel | Prueba |
|---|---|---|---|---|---|---|
| `-q` | Calidad 0–100 (con pérdida) o esfuerzo (sin pérdida) | `config.quality` | — | — | B | — |
| `-alpha_q` | Calidad de la transparencia | `config.alpha_quality` | — | — | A | — |
| `-preset` | Punto de partida: default, photo, picture, drawing, icon, text | `WebPConfigPreset` | — | — | B | — |
| `-z` | Nivel sin pérdida 0–9, atajo de velocidad y tamaño | `WebPConfigLosslessPreset` | — | — | A | — |
| `-m` | Método de compresión 0–6, velocidad frente a tamaño | `config.method` | — | — | A | — |
| `-segments` | Número de segmentos 1–4 | `config.segments` | — | — | E | — |
| `-size` | Tamaño objetivo en bytes | `config.target_size` | — | — | A | — |
| `-psnr` | PSNR objetivo en dB | `config.target_PSNR` | — | — | E | — |
| `-sns` | Fuerza del modelado espacial del ruido 0–100 | `config.sns_strength` | — | — | E | — |
| `-f` | Fuerza del filtro de desbloqueo 0–100 | `config.filter_strength` | — | — | E | — |
| `-sharpness` | Nitidez del filtro 0–7 | `config.filter_sharpness` | — | — | E | — |
| `-strong` / `-nostrong` | Filtro fuerte o simple | `config.filter_type` | — | — | E | — |
| `-sharp_yuv` | Conversión RGB→YUV más nítida y lenta | `config.use_sharp_yuv` | — | — | A | — |
| `-partition_limit` | Límite de calidad para que quepa la partición 0 | `config.partition_limit` | — | — | E | — |
| `-pass` | Pasadas de análisis 1–10 | `config.pass` | — | — | E | — |
| `-qrange` | Calidad mínima y máxima | `config.qmin` / `config.qmax` | — | — | E | — |
| `-af` | Filtro automático | `config.autofilter` | — | — | A | — |
| `-crop` | Recortar antes de codificar | `WebPPictureCrop` | — | — | B | — |
| `-resize` | Redimensionar antes de codificar | `WebPPictureRescale` | — | — | B | — |
| `-mt` | Varios hilos | `config.thread_level` | — | — | E | — |
| `-low_memory` | Menos memoria, más lento | `config.low_memory` | — | — | E | — |
| `-alpha_method` | Compresión de la transparencia 0–1 | `config.alpha_compression` | — | — | E | — |
| `-alpha_filter` | Filtro de la transparencia: none, fast, best | `config.alpha_filtering` | — | — | E | — |
| `-exact` | Conservar el RGB bajo los píxeles transparentes | `config.exact` | — | — | A | — |
| `-blend_alpha` | Mezclar la transparencia con un color de fondo | `WebPBlendAlpha` | — | — | A | — |
| `-noalpha` | Descartar la transparencia | (sin canal alfa) | — | — | A | — |
| `-lossless` | Sin pérdida | `config.lossless` | — | — | B | — |
| `-near_lossless` | Casi sin pérdida 0–100 | `config.near_lossless` | — | — | A | — |
| `-hint` | Pista del tipo de imagen: photo, picture, graph | `config.image_hint` | — | — | E | — |
| `-metadata` | Qué metadatos copiar: all, none, exif, icc, xmp | WebPMux | — | — | A | — |
| `-jpeg_like` | Ajustar el tamaño para que se parezca al de un JPEG | `config.emulate_jpeg_size` | — | — | E | — |
| `-map` / `-print_psnr` / `-print_ssim` / `-print_lsim` | Estadísticas | `WebPAuxStats` | — | — | E | — |
| `-d` / `-pgm` | Volcar la imagen comprimida | — | — | — | — | — |
| `-v` / `-quiet` / `-progress` / `-short` | Salida del programa | `progress_hook` | — | — | — | — |
| `-noasm` | Desactivar las instrucciones SIMD | `VP8GetCPUInfo` | — | — | — | — |

Las tres últimas filas son de la herramienta de terminal y no del resultado. En la interfaz, lo que
pintan (progreso, estadísticas) se ve sin pedirlo.
