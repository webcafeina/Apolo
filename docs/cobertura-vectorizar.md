# Cobertura de vectorizar

Última actualización: **2026-11-10** (fase 2, hito 6; ADR 0039)

Vectorizar no copia ninguna herramienta: el motor es propio (`crates/vector`, ADR 0038 y 0039). La regla es
la de las demás coberturas: **una opción no está hecha hasta que su fila está completa** (núcleo, CLI,
interfaz, MCP y prueba).

- **Núcleo**: el campo de `AjusteVector` (`crates/nucleo/src/vector.rs`).
- **CLI**: la opción de `apolo vectorizar` (`crates/cli/src/main.rs` y `vectorizar.rs`).
- **Interfaz**: la pantalla «Vectorizar» (hito 7); «—» es que todavía no está. **MCP**: el argumento de
  `vectorizar_imagen`. El área mínima no está en el MCP a propósito: es un ajuste fino que un agente no
  necesita.
- **Prueba**:
  - **cli**: `crates/cli/tests/vectorizar.rs`, con el binario de verdad;
  - **núcleo**: las pruebas de `crates/nucleo/src/vector.rs`;
  - **formatos**: `crates/vector/tests/formatos.rs`, que pinta cada formato y lo compara con el SVG (PDF
    con Poppler, EPS con Ghostscript frente al PDF, DXF con un lector del estándar y ezdxf);
  - **servicio**: `crates/servicio/src/vectores.rs`;
  - **mcp**: `crates/mcp/tests/vectorizar.rs`.

## Entrada y salida

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Entradas | Imágenes o carpetas (con subcarpetas) | `vector::vectorizar` | `ENTRADAS…` | — | `ruta` (una) | cli, mcp |
| Salida | Un fichero o, con varias, una carpeta; sin `-o`, al lado (una) o en «carpeta-svg» (varias), sin pisar | — | `-o`, `--salida` | — | `salida` | cli, mcp |
| Formato | svg, pdf, eps, dxf; por defecto, el de la extensión de `-o`, o svg | `FormatoVector` | `-f`, `--formato` | — | `formato` | cli, formatos, mcp |
| Imágenes grandes | Más de 2 Mpx: se reducen antes (y se avisa) | `MAX_PIXELES` | (siempre) | — | (siempre) | núcleo |
| Orientación EXIF | Se endereza antes de vectorizar | `preparar` | (siempre) | — | (siempre) | — |
| Transparencia | Lo transparente no se dibuja: sin fondo, y los agujeros quedan agujeros | `Rgba::alfa` | (siempre) | — | (siempre) | núcleo, `crates/vector/tests/transparencia.rs` |

## Dibujo

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Tipo | auto, logotipo, ilustración, blanco y negro, pixel art, foto | `tipo` | `-t`, `--tipo` | — | `tipo` | cli, núcleo, mcp |
| Colores | Como mucho, de 2 a 256; todos planos | `colores` | `-c`, `--colores` | — | `colores` | cli, mcp |
| Paleta | Colores fijos, `#rrggbb`; sin degradados | `paleta` | `--paleta` | — | `paleta` | cli, núcleo, mcp |
| Detalle | 0–100 (75): cuánto se ciñe; con objetivo, el mínimo | `detalle` | `-d`, `--detalle` | — | `detalle` | cli, mcp |
| Objetivo | El dibujo más ligero que da esta nota SSIMULACRA 2 | `objetivo` | `--objetivo` | — | `nota_objetivo` | cli, núcleo, mcp |
| Degradados | Lineales y radiales donde el color cambia suave | `degradados` | `--sin-degradados` | — | `degradados` | cli, formatos, mcp |
| Área mínima | Las formas de menos píxeles cuadrados, fuera (0,8) | `area_minima` | `--area-minima` | — | (no, a propósito) | cli |

## Exportar

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Estilo | rellenar, trazar formas, trazar bordes (cada borde una vez) | `estilo` | `-e`, `--estilo` | — | `estilo` | cli, formatos, mcp |
| Grosor | Del trazo, en píxeles (1) | `grosor` | `-g`, `--grosor` | — | `grosor` | cli, formatos, mcp |
| Color del trazo | `#rrggbb`; por defecto, el de la capa (en bordes, negro) | `color_trazo` | `--color-trazo` | — | `color_trazo` | cli, mcp |
| Nivel de DXF | rectas, arcos, splines | `nivel_dxf` | `--dxf` | — | `nivel_dxf` | cli, formatos, mcp |
