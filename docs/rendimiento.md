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
(`092d54f`, 2026-10-20).

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
