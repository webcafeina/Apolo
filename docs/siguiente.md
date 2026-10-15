# Lo siguiente

Última actualización: **2026-10-14**

El plan entero salió de la sesión del 2026-10-06, decidido con el cliente en cuatro rondas de
preguntas (ver [decisiones.md](decisiones.md)). Va **por entregas**; cada una se cierra con sus
pruebas en verde y su línea en [sesiones.md](sesiones.md).

## Alta

- **Entrega 7 — Agentes por MCP** (v0.8.0, [ADR 0023](adr/0023-mcp.md)). En tres partes:
  - ~~**7a**~~: `crates/mcp` con las diez herramientas (once desde la 7b), pruebas, catálogo y el paseo con Claude Code
    (2026-10-14, sin versión publicada).
  - **7b**, hecha en el código el 2026-10-14 y sin empujar: el empaquetado. `crates/mcpb` (`armar-mcpb`, manifiesto 0.3 con las herramientas sacadas
    de `apolo_mcp::ServidorApolo::herramientas()`); el sidecar `apolo-mcp` en los instaladores
    (`src-tauri/tauri.mcp.conf.json`, que se fusiona **solo al empaquetar**); el relevo de la copia
    del `.mcpb` al binario instalado (`APOLO_MCP_DESDE_PAQUETE`); la pestaña «Agentes» en Ajustes
    (`crates/servicio/src/agentes.rs`, órdenes `agentes` y `crear_mcpb`, `Agentes.tsx`), con su
    e2e; `apolo-mcp` en el paquete de la CLI; `publicar.yml`; README. **Lo que falta**: empujar y
    ver CI (comprobar, e2e y una publicación a mano con `workflow_dispatch` para ver los
    instaladores con el sidecar y los `.mcpb`); después, **la v0.8.0** (solo cuando lo diga el
    cliente: subir la versión en `Cargo.toml`, las notas ya están en `.github/notas/v0.8.0.md`) y la
    prueba guiada en su Mac: la fila MCP, crear la extensión e instalarla en Claude Desktop,
    preguntarle la versión, optimizar algo, y actualizar Apolo para ver que el agente sigue y dice la
    versión nueva.
  - **Pedido por el cliente el 2026-10-14 para la 7b** (hecho; ver ADR 0023):
    - **El conector se actualiza solo con Apolo**, sin reinstalar nada. Claude Code, Cursor y VS
      Code apuntan al binario instalado, que el actualizador reemplaza. El `.mcpb` de Claude Desktop
      cede el turno al instalado (el relevo de Esfinge, `internal/mcpb/relevo.go`). En Windows, el
      instalador NSIS cierra el `apolo-mcp.exe` en marcha antes de copiar (gancho
      `NSIS_HOOK_PREINSTALL`), o no podría reemplazarlo. La pantalla avisa de que los agentes se
      quedan con la lista de herramientas al conectar: lo nuevo aparece en la conversación siguiente.
    - **El agente sabe decir su versión**: va en las `instructions` y en una herramienta
      `acerca_de_apolo` (versión, de dónde arranca, si viene del `.mcpb` y cedió el turno, y las
      versiones de los motores). La sección de Apolo enseña la misma versión.
    - **Una fila «MCP» en la barra lateral, encima de Ajustes**, que abre Ajustes por el bloque de
      los agentes, donde está todo. Como Esfinge: no es una pantalla aparte, es una puerta a Ajustes.
  - **Retoques de la prueba guiada (2026-10-15), para la 7c:**
    - El agente de Claude Desktop **creyó que Apolo podía estar en otro ordenador** al ver
      `/Users/alvaro` en `acerca_de_apolo`, y avisó sin motivo (se corrigió solo después). Decirlo en
      las instrucciones y en `acerca_de_apolo`: Apolo corre en este ordenador, con la cuenta de quien
      habla con el agente.
    - **Una imagen adjunta en el chat no está en el disco**: Claude Desktop la sube a la
      conversación y Apolo solo trabaja con rutas. El agente buscó el fichero y, al parecer, copió el
      original a `~/Downloads/apolo-monte-alto/` con otra herramienta (sin confirmar). Decirlo en las
      instrucciones: si adjuntan una imagen, pedir su ruta o buscarla; Apolo no ve los adjuntos.
    - **Claude Code borró el original** cuando se le pidió «sobrescribe el original, no quiero
      copias» (prueba del 2026-10-15, Sonnet 5.5, esfuerzo medio): Apolo no sobrescribió, y el agente
      lo hizo con su shell (`rm` del JPEG y de seis copias). **Claude Code pidió permiso** antes del
      `rm` (modo «Accept edits on», que no aprueba órdenes de shell) y el cliente lo aceptó; el
      agente no avisó **antes** de que el original no tenía vuelta atrás, solo después. Añadir a las instrucciones: si piden sobrescribir o borrar originales, confirmar antes
      con la persona que el original se pierde, y proponer guardarlo aparte. No lo garantiza (el
      agente obedece a la persona), pero lo hace preguntar.
  - **7c**: la evaluación con unos 25 escenarios de `claude -p` (`scripts/evaluar-mcp.mjs`, informe
    en `docs/mcp/evaluacion.md`), con uno adversario («sobrescribe el original»). Los arreglos van a
    las descripciones, no a los escenarios. v0.8.1 si cambia algo.

- ~~Publicar la v0.4.0~~ (2026-10-07). **Que el cliente actualice desde la v0.3.3** con el
  actualizador: es la primera vez que se ve de verdad (deuda). La clave del actualizador se queda en el VPS por decisión del
  cliente (deuda).

- **Montar la equivalencia con cwebp en CI para Windows y Linux arm64** (deuda). En un Mac con
  Apple Silicon ya la comprobó a mano el cliente: 25 de 25.

## Media

- **Después de la v0.8.0, pedido por el cliente el 2026-10-15:**
  - **Recortar con el ratón en el Estudio**, como en Squoosh: dibujar el rectángulo sobre el
    original, con asas, en vez de escribir x, y, ancho y alto. Hoy el recorte del proceso es
    numérico, y con él el comparador pasa a lado a lado con escala aproximada (deuda: «Recortar en
    el comparador»). Hace falta decidir la proporción fija (libre, 1:1, 4:3, 16:9…) y cómo se
    compara después.
  - **Recortar y reducir la paleta desde el agente (MCP).** Hoy `EntradaFormato` solo lleva `ancho` y
    `alto`. Añadir `recorte` (`{x, y, ancho, alto}`, como la `zona` de `ver_imagen`) y `paleta`
    (número de colores), con sus errores y pistas (fuera de la imagen, paleta en formatos sin
    pérdida), en `optimizar_imagen`, `comparar_formatos`, `empezar_lote` y `guardar_preset`. Regenerar
    el catálogo y añadirlo a la evaluación de la 7c.

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
