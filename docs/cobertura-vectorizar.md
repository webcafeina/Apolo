# Cobertura de vectorizar

Última actualización: **2026-11-10** (fase 2, hito 7; ADR 0039)

Vectorizar no copia ninguna herramienta: el motor es propio (`crates/vector`, ADR 0038 y 0039). La regla es
la de las demás coberturas: **una opción no está hecha hasta que su fila está completa** (núcleo, CLI,
interfaz, MCP y prueba).

- **Núcleo**: el campo de `AjusteVector` (`crates/nucleo/src/vector.rs`).
- **CLI**: la opción de `apolo vectorizar` (`crates/cli/src/main.rs` y `vectorizar.rs`).
- **Interfaz**: la pantalla «Vectorizar» (`frontend/src/vectorizar/`), con el grupo del panel. **MCP**: el argumento de
  `vectorizar_imagen`. El área mínima no está en el MCP a propósito: es un ajuste fino que un agente no
  necesita.
- **Prueba**:
  - **cli**: `crates/cli/tests/vectorizar.rs`, con el binario de verdad;
  - **núcleo**: las pruebas de `crates/nucleo/src/vector.rs`;
  - **formatos**: `crates/vector/tests/formatos.rs`, que pinta cada formato y lo compara con el SVG (PDF
    con Poppler, EPS con Ghostscript frente al PDF, DXF con un lector del estándar y ezdxf);
  - **servicio**: `crates/servicio/src/vectores.rs`;
  - **mcp**: `crates/mcp/tests/vectorizar.rs`;
  - **e2e**: `frontend/e2e/vectorizar.spec.ts`, en el navegador contra `apolo-dev`.

## Entrada y salida

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Entradas | Imágenes o carpetas (con subcarpetas) | `vector::vectorizar` | `ENTRADAS…` | abrir o soltar (una) | `ruta` (una) | cli, mcp, e2e |
| Salida | Un fichero o, con varias, una carpeta; sin `-o`, al lado (una) o en «carpeta-svg» (varias), sin pisar | — | `-o`, `--salida` | diálogo de guardar | `salida` | cli, mcp, e2e |
| Formato | svg, pdf, eps, dxf; por defecto, el de la extensión de `-o`, o svg | `FormatoVector` | `-f`, `--formato` | ✓ Exportar | `formato` | cli, formatos, mcp, e2e |
| Imágenes grandes | Más de 2 Mpx: se reducen antes (y se avisa) | `MAX_PIXELES` | (siempre) | aviso | (siempre) | núcleo |
| Orientación EXIF | Se endereza antes de vectorizar | `preparar` | (siempre) | (siempre) | (siempre) | — |
| Transparencia | Lo transparente no se dibuja: sin fondo, y los agujeros quedan agujeros; en la pantalla, sobre cuadros | `Rgba::alfa` | (siempre) | «con transparencia», damero | (siempre) | núcleo, `crates/vector/tests/transparencia.rs`, servicio, e2e |

## Dibujo

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Tipo | auto, logotipo, ilustración, blanco y negro, pixel art, foto | `tipo` | `-t`, `--tipo` | ✓ cabecera del panel | `tipo` | cli, núcleo, mcp, e2e |
| Colores | Como mucho, de 2 a 256; todos planos | `colores` | `-c`, `--colores` | ✓ Dibujo | `colores` | cli, mcp, e2e |
| Paleta | Colores fijos, `#rrggbb`; sin degradados | `paleta` | `--paleta` | ✓ Avanzado | `paleta` | cli, núcleo, mcp, e2e |
| Detalle | 0–100 (75): cuánto se ciñe; con objetivo, el mínimo | `detalle` | `-d`, `--detalle` | ✓ Dibujo | `detalle` | cli, mcp, e2e |
| Objetivo | El dibujo más ligero que da esta nota SSIMULACRA 2 | `objetivo` | `--objetivo` | ✓ Dibujo | `nota_objetivo` | cli, núcleo, mcp, e2e |
| Degradados | Lineales y radiales donde el color cambia suave | `degradados` | `--sin-degradados` | ✓ Dibujo | `degradados` | cli, formatos, mcp, e2e |
| Área mínima | Las formas de menos píxeles cuadrados, fuera (0,8) | `area_minima` | `--area-minima` | ✓ Avanzado | (no, a propósito) | cli, e2e |

## Exportar

| Opción | Qué hace | Núcleo | CLI | Interfaz | MCP | Prueba |
|---|---|---|---|---|---|---|
| Estilo | rellenar, trazar formas, trazar bordes (cada borde una vez) | `estilo` | `-e`, `--estilo` | ✓ Exportar | `estilo` | cli, formatos, mcp, e2e |
| Grosor | Del trazo, en píxeles (1) | `grosor` | `-g`, `--grosor` | ✓ Exportar | `grosor` | cli, formatos, mcp, e2e |
| Color del trazo | `#rrggbb`; por defecto, el de la capa (en bordes, negro) | `color_trazo` | `--color-trazo` | ✓ Exportar | `color_trazo` | cli, mcp, e2e |
| Nivel de DXF | rectas, arcos, splines | `nivel_dxf` | `--dxf` | ✓ Exportar (con DXF) | `nivel_dxf` | cli, formatos, mcp, e2e |
