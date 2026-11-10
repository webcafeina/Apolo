# Cobertura de vectorizar

Última actualización: **2026-11-10** (fase 2, hito 5; ADR 0039)

Vectorizar no copia ninguna herramienta: el motor es propio (`crates/vector`, ADR 0038 y 0039). La regla es
la de las demás coberturas: **una opción no está hecha hasta que su fila está completa** (núcleo, CLI,
interfaz, MCP y prueba).

- **Núcleo**: el campo de `AjusteVector` (`crates/nucleo/src/vector.rs`).
- **CLI**: la opción de `apolo vectorizar` (`crates/cli/src/main.rs` y `vectorizar.rs`).
- **Interfaz**: la pantalla «Vectorizar» (hito 7). **MCP**: `vectorizar_imagen` (hito 6). «—» es que todavía
  no está.
- **Prueba**:
  - **cli**: `crates/cli/tests/vectorizar.rs`, con el binario de verdad;
  - **núcleo**: las pruebas de `crates/nucleo/src/vector.rs`;
  - **formatos**: `crates/vector/tests/formatos.rs`, que pinta cada formato y lo compara con el SVG (PDF
    con Poppler, EPS con Ghostscript frente al PDF, DXF con un lector del estándar y ezdxf);
  - **servicio**: `crates/servicio/src/vectores.rs`.

## Entrada y salida

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Entradas | Imágenes o carpetas (con subcarpetas) | `vector::vectorizar` | `ENTRADAS…` | — | — | cli |
| Salida | Un fichero o, con varias, una carpeta; sin `-o`, al lado (una) o en «carpeta-svg» (varias), sin pisar | — | `-o`, `--salida` | — | — | cli |
| Formato | svg, pdf, eps, dxf; por defecto, el de la extensión de `-o`, o svg | `FormatoVector` | `-f`, `--formato` | — | — | cli, formatos |
| Imágenes grandes | Más de 2 Mpx: se reducen antes (y se avisa) | `MAX_PIXELES` | (siempre) | — | — | núcleo |
| Orientación EXIF | Se endereza antes de vectorizar | `preparar` | (siempre) | — | — | — |

## Dibujo

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Tipo | auto, logotipo, ilustración, blanco y negro, pixel art, foto | `tipo` | `-t`, `--tipo` | — | — | cli, núcleo |
| Colores | Como mucho, de 2 a 256; todos planos | `colores` | `-c`, `--colores` | — | — | cli |
| Paleta | Colores fijos, `#rrggbb`; sin degradados | `paleta` | `--paleta` | — | — | cli, núcleo |
| Detalle | 0–100 (75): cuánto se ciñe; con objetivo, el mínimo | `detalle` | `-d`, `--detalle` | — | — | cli |
| Objetivo | El dibujo más ligero que da esta nota SSIMULACRA 2 | `objetivo` | `--objetivo` | — | — | cli, núcleo |
| Degradados | Lineales y radiales donde el color cambia suave | `degradados` | `--sin-degradados` | — | — | cli, formatos |
| Área mínima | Las formas de menos píxeles cuadrados, fuera (0,8) | `area_minima` | `--area-minima` | — | — | cli |

## Exportar

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Estilo | rellenar, trazar formas, trazar bordes (cada borde una vez) | `estilo` | `-e`, `--estilo` | — | — | cli, formatos |
| Grosor | Del trazo, en píxeles (1) | `grosor` | `-g`, `--grosor` | — | — | cli, formatos |
| Color del trazo | `#rrggbb`; por defecto, el de la capa (en bordes, negro) | `color_trazo` | `--color-trazo` | — | — | cli |
| Nivel de DXF | rectas, arcos, splines | `nivel_dxf` | `--dxf` | — | — | cli, formatos |
