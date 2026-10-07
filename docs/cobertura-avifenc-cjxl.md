# Cobertura de avifenc y cjxl

Última actualización: **2026-10-07** (entrega 5b, v0.6)

Con avifenc y cjxl la regla de [cobertura-cwebp.md](cobertura-cwebp.md) se cumple de otra manera
(ADR 0021): **Apolo ejecuta el `main` de la herramienta** con los argumentos de la orden, así que
**todas sus opciones** dan el mismo efecto que en el binario oficial, también en `apolo avif` y
`apolo jxl`. Lo que cambia de una opción a otra es si el panel la enseña y si la prueba la cubre.

- **Núcleo**: el campo de `OpcionesAvif` (`crates/nucleo/src/formatos/avif.rs`) u `OpcionesJxl`
  (`formatos/jxl.rs`), u **otras** si va tal cual en `otras`.
- **Interfaz**: ✓ y la sección del panel (**B**, **A** o **E**), desde `frontend/src/estudio/opciones.ts`.
- **Prueba**:
  - **eq**: está en la lista de `crates/nucleo/tests/equivalencia_avif_jxl.rs`, que compara con el
    binario oficial **byte a byte**;
  - **ida**: la cubre la prueba de ida y vuelta opciones ⇄ orden.

## avifenc (libavif 1.4.2, aom 3.14.1)

| Opción | Qué hace | Núcleo | Interfaz | Prueba |
|---|---|---|---|---|
| `-q`, `--qcolor` | Calidad 0–100 (60) | `calidad` | ✓ B | eq, ida |
| `-s`, `--speed` | Velocidad 0–10 (6) | `velocidad` | ✓ B | eq, ida |
| `-l`, `--lossless` | Sin pérdida | `sin_perdida` | ✓ B | eq |
| `--qalpha` | Calidad del alfa | `calidad_alfa` | ✓ A | eq |
| `-y`, `--yuv` | 444, 422, 420, 400 | `submuestreo` | ✓ A | eq, ida |
| `--sharpyuv` | Conversión a 4:2:0 de libwebp | `sharpyuv` | ✓ A | eq, ida |
| `-a tune=…` | Afinado de aom | `afinado` | ✓ A | eq, ida |
| `-a sharpness=…` | Nitidez 0–7 | `nitidez` | ✓ A | eq, ida |
| `--progressive` | Por capas | `progresivo` | ✓ A | eq |
| `-d`, `--depth` | 8, 10 o 12 bits | `profundidad` | ✓ E | eq |
| `-r`, `--range` | Rango limitado o completo | `rango_limitado` | ✓ E | eq |
| `-p`, `--premultiply` | Premultiplicar el alfa | `premultiplicar` | ✓ E | eq |
| `--ignore-exif`, `--ignore-xmp`, `--ignore-icc` | No copiar metadatos | `sin_exif`… | ✓ E | eq, ida |
| `-a` con otra clave, `-c`, `-j`, `--autotiling`, `--tilerowslog2`, `--tilecolslog2`, `--min`, `--max`, `--minalpha`, `--maxalpha`, `--cicp`, `--target-size`, `--pasp`, `--crop`, `--clap`, `--irot`, `--imir`, `--clli`, `-k`, `--timescale`, `--duration`, `--scaling-mode`, `--input-format`, `--mini`, `--ignore-gain-map`, `--qgain-map`, `:u` | Las demás | otras | — | eq (`-a color:…`, `--autotiling`, `--min/--max`), ida |
| `--no-overwrite` | No cambia el fichero | — | — | — |
| `-o`, `--exif`, `--xmp`, `--icc`, `--grid`, `--layered`, `--stdin` | Ficheros aparte o varias imágenes | **error en el Estudio**; en `apolo avif` van a avifenc tal cual | — | — |

## cjxl (libjxl 0.12.0)

| Opción | Qué hace | Núcleo | Interfaz | Prueba |
|---|---|---|---|---|
| `-q`, `--quality` | Calidad (90 ≈ `-d 1`, lo de cjxl) | `calidad` | ✓ B | eq, ida |
| `-d`, `--distance` | Distancia visual; excluye `-q` | `distancia` | (la calidad la quita) | eq, ida |
| `-e`, `--effort` | Esfuerzo 1–10 (7) | `esfuerzo` | ✓ B | eq, ida |
| `-j`, `--lossless_jpeg` | Recomprimir el JPEG sin pérdida (1) | `jpeg_sin_perdida` | ✓ B, solo con JPEG | eq, ida |
| `-a`, `--alpha_distance` | Distancia del alfa | `distancia_alfa` | ✓ A | eq |
| `-p`, `--progressive` | Progresivo | `progresivo` | ✓ A | eq, ida |
| `-m`, `--modular` | VarDCT o modular | `modular` | ✓ A | eq |
| `--photon_noise_iso` | Grano | `ruido_iso` | ✓ A | eq |
| `--epf` | Filtro de bordes | `epf` | ✓ E | eq, ida |
| `--gaborish` | Gaborish | `gaborish` | ✓ E | eq |
| `--faster_decoding` | Decodificación rápida | `decodificacion_rapida` | ✓ E | eq |
| Las demás de `cjxl -h -v -v -v -v` (`--resampling`, `-I`, `-C`, `-P`, `--container`, `--compress_boxes`, `--progressive_dc`, `--modular_palette_colors`…) | | otras | — | eq (varias), ida |
| `--quiet`, `-v`, `--num_reps` | No cambian el fichero | — | — | — |
| `-x` con ficheros, `--disable_output`, `--streaming_input` | | **error en el Estudio**; en `apolo jxl` van a cjxl tal cual | — | — |

**Entradas:** las dos leen PNG y JPEG, comprobados. cjxl lee además PNM, GIF, APNG y JXL; Apolo se
los pasa tal cual en la CLI, pero en el Estudio y en Lotes los convierte a PNG y lo avisa.
