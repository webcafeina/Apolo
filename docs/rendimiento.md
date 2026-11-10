# Rendimiento

Lo que gasta Apolo con fotos grandes, medido con `scripts/banco.sh` (entrega 8b, ADR 0024). El
objetivo: que todo quepa con holgura en un Mac de 8 GB, con un pico por debajo de unos 2,5 GB.

## Cómo se mide

- **Fotos**: las 24 de Kodak (las de `pruebas/lotes/generar.sh`) ampliadas a 6000 × 4000 (24 MP),
  en JPEG de calidad alta, como las de una cámara. Ampliadas tienen menos detalle que una foto de
  24 MP de verdad: el tiempo de codificar puede salir algo corto, pero la memoria depende de los
  píxeles. El lote son 200 enlaces duros a esas 24.
- **Lotes con 8 a la vez** (`-j 8`), como en un Mac con M1, que tiene 8 núcleos.
- **Pico**: la memoria residente máxima del proceso (`/usr/bin/time`). El Estudio se mide paso a
  paso con `examples/banco_estudio.rs`, que usa el mismo `Servicio` que la ventana, sin la
  interfaz. El agente, con `scripts/banco_mcp.py`, por las tuberías como lo pide Claude.
- **La máquina**: el VPS de desarrollo, Linux x86-64 con 4 núcleos y 15 GB. Los tiempos dependen
  de la máquina (y de su carga); la memoria, mucho menos.

```sh
scripts/banco.sh                    # 200 fotos, unos 45 minutos → target/banco/resultado.md
BANCO_LOTE=16 scripts/banco.sh      # un lote corto
```

## Antes y después

Con una foto de 24 MP, o 200 en los lotes. «Antes» es la v0.10.2; «ahora», la v0.11.0
(`ee1e05d`, 2026-10-20).

| Caso | Antes | Ahora |
|---|---:|---:|
| `apolo medir` (SSIMULACRA 2, PSNR, SSIM) | 4,3 GB · 25 s | **1,7 GB · 8 s** |
| `apolo webp` / `jpeg` / `qoi` | 0,1 / 0,2 / 0,2 GB | igual |
| `apolo avif` / `jxl` / `png` | 0,7 / 0,5 / 0,5 GB | igual |
| Estudio: la sesión entera | 4,5 GB | **1,9 GB** |
| Estudio: vista previa AVIF | 2,0 GB · 11,8 s | **0,9 GB · 4,4 s** |
| Estudio: mapa de estructura | 1,85 GB | **1,3 GB** |
| Lote → WebP | 0,9 GB · 143 s | 1,0 GB · 110 s |
| Lote → AVIF | 2,9 GB · 707 s | **1,5 GB · 490 s** |
| Lote → WebP, JPEG y AVIF, el más ligero | 3,0 GB · 609 s | **2,2 GB · 586 s** |
| Lote → WebP midiendo | 6,0 GB (16 fotos) | **2,3 GB** |
| Agente: `comparar_formatos` (webp, avif, jpeg) | 4,1 GB · 324 s | **2,0 GB · 165 s** |

**Ningún fichero de salida cambia**: `make equivalencia` sigue igual (cwebp 1015, cjpeg 816,
oxipng 351, qoiconv 13, cjxl 280 y avifenc 376 de los mismos), y la nota SSIMULACRA 2 es la de la
herramienta oficial (56 de 56).

## Qué se cambió

- **SSIMULACRA 2, canal a canal** (`crates/avifjxl/c/medir.cc`). La función de libjxl tenía a la
  vez unas once imágenes de tres canales en coma flotante (288 MB cada una con 24 MP). Su cálculo
  es independiente por canal, así que se hace canal a canal, con las mismas operaciones en el
  mismo orden, y la imagen grande se suelta en cuanto se ha reducido. La nota es la misma bit a
  bit (`crates/nucleo/tests/ssimulacra2_ligero.rs` la compara con la de libjxl).
- **Ver un AVIF** (`avif_a_png`): avifdec con un hilo y el PNG sin comprimir. El decodificador de
  aom reserva mucho por cada hilo, y comprimir un PNG que se lee al momento no servía de nada:
  de 1,8 GB y 8,8 s a 433 MB y 1,6 s, con los mismos píxeles.
- **El turno de medir antes de descomprimir** (`medir::turno`): en un lote, los hilos que esperan
  para medir ya no tienen el original y el resultado descomprimidos en memoria.
- **PSNR y SSIM desde los píxeles**, sin copias en coma flotante de la imagen entera.
- **En Linux, dos reservas de memoria como mucho** (`nucleo::memoria`, trampas.md): glibc da a cada
  hilo la suya y no devuelve lo liberado. macOS y Windows devuelven los bloques grandes.

## Lo que queda

- **`comparar_formatos` tarda** con fotos muy grandes (unos 3 minutos con 24 MP). El cliente
  prefirió no acelerarlo: con más de unos 20 MP, la descripción manda a `empezar_lote` con
  `mas_ligero`, que va en segundo plano (deuda: los tiempos de espera de cada cliente).
- **oxipng con 24 MP tarda unos 30 s**: es lo que tarda el oficial con sus opciones por defecto.
- **No se ha medido en un Mac** ni en Windows: solo aquí. La memoria debería ser parecida o menor
  (no tienen el problema de glibc); los tiempos, otros.

## Vectorizar (hito 2, 2026-11-10, ADR 0039)

Medido en el banco de `laboratorio/vectorizar/` (`APOLO_MOTORES=apolo-crate`), **de una imagen en una**:
el banco procesa ocho a la vez y en cuatro núcleos se pisan (las fotos salen a unos 14 s). Mismo VPS.

| Imagen | Antes | Después |
|---|---|---|
| Bandera de EE. UU., 512 × 384 (50 estrellas) | 20 s | **1,1 s** |
| Bandera de EE. UU., 128 px | 1 s | 0,08 s |
| La más lenta sin fotos, con el banco cargado (ocho a la vez) | 27,9 s | 1,7 s (media 0,35 s) |
| Foto de 1.024 × 970 (la pera) | 6,5 s | **3,9 s** |

Objetivos del plan: menos de 2 s para 512 px y menos de 10 s para una foto de 1.024 px. Se cumplen.

- **La síntesis** (`crates/vector/src/sintesis.rs`) era el 97 % del tiempo: para cada prueba de mover un
  vértice preguntaba, submuestra a submuestra (36 por píxel), si estaba dentro del polígono recorriendo
  todos sus lados, y recalculaba también el error de antes del movimiento. Ahora corta los lados una vez
  por fila de submuestras y cuenta (de 19,3 s a 3,3 s), y guarda el error de cada píxel en una rejilla
  alrededor de la figura (a 1,9 s). Las submuestras están en las mismas posiciones y se cuentan como
  enteros: el resultado es el mismo bit a bit.
- **En paralelo con rayon**: las capas, las formas de cada capa y los dos órdenes de capas que se comparan.
  Las fotos ganan sobre todo por las capas (24, y en cada una los contornos son lo caro).

### El modo por nota (hito 5, 2026-11-10)

`apolo vectorizar --objetivo N --detalle 0`, con la CLI en release, de una en una, en el mismo VPS. Parte en dos
el detalle (hasta nueve pruebas, cada una vectoriza y mide con SSIMULACRA 2):

| Imagen | Objetivo 70 | Objetivo 80 |
|---|---|---|
| Corazón (tabler), 512 px | 0,18 s (detalle 26) | 0,19 s (detalle 44) |
| Ancla (silueta) | 0,75 s (26) | 0,74 s (63) |
| Taza (openmoji) | 0,80 s (58) | 0,91 s (no llega: 73,8) |
| Cohete (openmoji) | 1,2 s (43) | 1,2 s (no llega: 75,2) |
| León (noto) | 1,9 s (39) | 2,1 s (no llega: 79,7) |
| Bandera de Corea | 2,6 s (12) | 2,9 s (38) |
| Bandera de EE. UU. | 0,73 s (0, a la primera) | 6,9 s (no llega: 76,9) |

**Más detalle no siempre da más nota** (la bandera de EE. UU.: 76,9 con el detalle 0 y 74,2 con el 100), así
que de lo probado se queda el dibujo más ligero que llega, y si ninguno llega, el de mejor nota.

