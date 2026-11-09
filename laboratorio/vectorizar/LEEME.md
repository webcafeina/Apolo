# Banco de vectorizar (fase 1, ADR 0038)

Laboratorio para investigar cómo vectorizar en Apolo. **No forma parte de la aplicación**: es un proyecto
Cargo aparte, fuera del workspace, que ni la CI ni `make comprobar` compilan.

```sh
make cli                         # en la raíz: el binario `apolo` que mide la nota
corpus/descargar.sh              # las fotos del corpus (los SVG ya están en git)
cargo run --release              # todo el corpus; o `cargo run --release -- bandera` para filtrar
```

Escribe en `salida/` (sin git) los SVG de cada motor, `resultados.csv` y dos galerías: `index.html` con
ficheros sueltos y `galeria-artefacto.html` en un solo fichero. Los SVG de Vectorizer.ai, si están en
`capturas/vectorizar/vectorizer-ai/<caso>.svg`, entran como un motor más.

| Módulo | Qué hace |
|---|---|
| `corpus.rs` | Los 53 casos y cómo se prepara cada uno (licencias en `corpus/LICENCIAS.md`) |
| `imagen.rs` | Rasterizar con resvg y medir con `apolo medir` (SSIMULACRA 2) |
| `paleta.rs` | Paleta automática con los píxeles planos; ¿hay suavizado? |
| `subpixel.rs` | Mezcla de colores en el borde y contorno por debajo del píxel |
| `poli.rs` | El motor `apolo-poli`: polígono primero, rectas, esquinas, Bézier, círculos |
| `ajuste.rs` | Bézier (Schneider), rectas, círculos y el motor anterior `apolo` |
| `motores.rs` | Todos los motores que se comparan |
| `potrace.rs` | Enlace con Potrace 1.16 (`vendor/potrace`, GPL-2.0-or-later) |

Variables para investigar: `APOLO_TOL_POLI`, `APOLO_TOL_CURVA`, `APOLO_DEPURAR`, `APOLO_VOLCAR`. En
zsh, para pasar varias: `env ${=v} cargo run --release` (trampas.md).
