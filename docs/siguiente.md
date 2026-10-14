# Lo siguiente

Última actualización: **2026-10-14**

El plan entero salió de la sesión del 2026-10-06, decidido con el cliente en cuatro rondas de
preguntas (ver [decisiones.md](decisiones.md)). Va **por entregas**; cada una se cierra con sus
pruebas en verde y su línea en [sesiones.md](sesiones.md).

## Alta

- **Entrega 7 — Agentes por MCP** (v0.8.0, [ADR 0023](adr/0023-mcp.md)). En tres partes:
  - ~~**7a**~~: `crates/mcp` con las diez herramientas, pruebas, catálogo y el paseo con Claude Code
    (2026-10-14, sin versión publicada).
  - **7b**: el empaquetado. `crates/mcpb` (`armar-mcpb`, manifiesto 0.3 con las herramientas sacadas
    de `apolo_mcp::ServidorApolo::herramientas()`); el sidecar `apolo-mcp` en los instaladores
    (`src-tauri/tauri.mcp.conf.json`, que se fusiona **solo al empaquetar**); el relevo de la copia
    del `.mcpb` al binario instalado (`APOLO_MCP_DESDE_PAQUETE`); la pestaña «Agentes» en Ajustes
    (`crates/servicio/src/agentes.rs`, órdenes `agentes` y `crear_mcpb`, `Agentes.tsx`), con su
    e2e; `apolo-mcp` en el paquete de la CLI; `publicar.yml`; README. Después, **la v0.8.0** (solo
    cuando lo diga el cliente) y la prueba guiada en su Mac con Claude Desktop y el `.mcpb`.
  - **7c**: la evaluación con unos 25 escenarios de `claude -p` (`scripts/evaluar-mcp.mjs`, informe
    en `docs/mcp/evaluacion.md`), con uno adversario («sobrescribe el original»). Los arreglos van a
    las descripciones, no a los escenarios. v0.8.1 si cambia algo.

- ~~Publicar la v0.4.0~~ (2026-10-07). **Que el cliente actualice desde la v0.3.3** con el
  actualizador: es la primera vez que se ve de verdad (deuda). La clave del actualizador se queda en el VPS por decisión del
  cliente (deuda).

- **Montar la equivalencia con cwebp en CI para Windows y Linux arm64** (deuda). En un Mac con
  Apple Silicon ya la comprobó a mano el cliente: 25 de 25.

## Media

- **Comprobar AVIF y JPEG XL en Windows** contra los binarios oficiales (deuda): en CI, con el
  `windows-artifacts.zip` de libavif y el `jxl-x64-windows-static.zip` de libjxl. AVIF en Mac ya
  está (16 de 16, 2026-10-08).
- ~~**Entrega 6 — Métricas.**~~ Hecha como v0.7.0 (ver Cerrado).
- **Entrega 8 — 1.0.** DMG, NSIS, `.deb` y AppImage; README con la tabla de descargas. (La
  actualización automática se adelantó a la v0.3.3: ADR 0018.) Era la 7 hasta el 2026-10-14, cuando
  el cliente metió delante los agentes por MCP.

## Baja

- **Instalar la CLI desde un repositorio de paquetes.** Hoy solo se baja el `.tar.gz` de la
  Release, o se compila con `cargo install --git …`, que exige Rust, nasm y CMake. El cliente lo
  preguntó el 2026-10-07 y lo dejó para el futuro, no es prioritario.
  - **Lo propuesto, por orden**: primero **Homebrew con un tap propio**
    (`webcafeina/homebrew-apolo`, `brew install webcafeina/apolo/apolo`). Cubre macOS y Linux, y
    Homebrew no pone la cuarentena. Después Scoop o winget en Windows, y un repositorio apt propio
    en Debian, firmado y alojado.
  - **Qué haría falta** para el tap:
    - la receta, unas 20 líneas, que apunta al `.tar.gz` de cada Release;
    - un paso en `publicar.yml` que la actualice al publicar (versión y sumas);
    - un token de GitHub con permiso solo sobre ese repositorio, como secreto.

- WebP animado: `gif2webp` e `img2webp`.
- Butteraugli. Ya está dentro con libjxl (ADR 0022), pero es lenta; SSIMULACRA 2 cubre lo mismo.
- Firmar y notarizar ([ADR 0009](adr/0009-sin-firmar.md)).
- La traducción al inglés: con la i18n hecha, es traducir `es.json`.

## Cerrado

- ~~**Entrega 6 — Métricas**~~ (v0.7.0, publicada el 2026-10-08): SSIMULACRA 2 de libjxl (la misma
  nota que su herramienta, 56 de 56), PSNR y SSIM; el modo «Diferencias» con dos mapas; medir en
  Lotes; la calidad por nota en el Estudio, Lotes y la CLI; `apolo medir`. ADR 0022. · 2026-10-08

- ~~**Entrega 5b — AVIF y JPEG XL**~~ (v0.6.0, publicada el 2026-10-08): avifenc 1.4.2 y cjxl 0.12.0
  compilados dentro, con sus `main`; libjxl con clang. **376 de 376 iguales con avifenc y 280 de
  280 con cjxl.** Se abren AVIF y JXL; Estudio, Lotes, pegar órdenes, `apolo avif` y `apolo jxl`.
  ADR 0021. · 2026-10-07

- ~~**Entrega 5a — JPEG, PNG, QOI y proceso**~~ (v0.5.0): MozJPEG, OxiPNG y QOI con el mismo
  fichero que cjpeg, oxipng y qoiconv (816, 351 y 13 de 13); el proceso (enderezar, recortar,
  redimensionar y reducir paleta); el comparador con un formato por lado; Lotes con varios formatos
  y «solo el más ligero»; `apolo jpeg`, `apolo png` y `apolo qoi`. ADR 0020. · 2026-10-07

- ~~**Entrega 4 — Lotes**~~: carpetas y ficheros, presets, salida con subcarpetas sin pisar nada,
  en paralelo y cancelable, resumen con las que menos ahorran; `apolo lote` en la CLI. Cada imagen
  da los mismos bytes que cwebp. Los varios formatos de salida llegan con la entrega 5. ADR 0019.
  · 2026-10-07

- ~~**Entrega 0 — Cimientos**~~: repositorio, documentos vivos, workspace, carcasa de la interfaz,
  tokens con contraste y CI en verde en los seis objetivos. · 2026-10-06
- ~~**Entrega 1 — Núcleo WebP y CLI**~~: lectores de PNG, JPEG, TIFF, WebP, PNM, GIF, BMP, QOI y YUV;
  todas las opciones de cwebp 1.6.0 en el núcleo y en `apolo webp`; metadatos; las opciones como
  JSON; preset ⇄ orden cwebp en los dos sentidos. **Mismo fichero que cwebp byte a byte en 1015
  comparaciones.** Los presets con nombre (guardar, listar) quedan para la entrega 2, que es donde
  se usan. · 2026-10-06
- ~~**Entrega 2 — Estudio**~~: comparador con deslizador y lado a lado, zoom y desplazamiento;
  panel en tres niveles plegables pintado desde un esquema, con su explicación; vista previa en vivo
  con generaciones; la orden cwebp, copiable y pegable; presets con nombre como JSON, compartidos con
  la CLI (`-apolo_preset`, `apolo presets`); **enderezar según EXIF como opción** (ADR 0012), con aviso;
  exportar; `apolo-dev` y 14 pruebas de Playwright. El modo «diferencias» pasa a la entrega 5. ·
  2026-10-06
- ~~**v0.3.2 — lo que salió de la prueba guiada**~~: cabecera con aire; pesos con etiquetas, ahorro
  y barra; **la ventana ya no rebota con el zoom**; aviso de vista reducida; orden copiada con rutas
  completas (y un `.webp` ya no se propone con su mismo nombre); sección **Presets**; «Distinto de
  cwebp» con su motivo; y **leer HEIC** (ADR 0017). · 2026-10-06
- ~~**Entrega 3 — Identidad**~~ (ADR 0015 y 0016): el sol con laurel elegido entre cuatro propuestas;
  el oro del sol como acento («el oro rellena, la piedra escribe»); barra lateral translúcida en
  macOS; sello, firma, bienvenida con el icono y ficha en «Acerca de»; `.dmg` con fondo propio y sin
  licencia; `make iconos` y `make ventana-dmg`. · 2026-10-06
