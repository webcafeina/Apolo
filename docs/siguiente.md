# Lo siguiente

Última actualización: **2026-10-06**

El plan entero salió de la sesión del 2026-10-06, decidido con el cliente en cuatro rondas de
preguntas (ver [decisiones.md](decisiones.md)). Va **por entregas**; cada una se cierra con sus
pruebas en verde y su línea en [sesiones.md](sesiones.md).

## Alta

- **v0.3.2 — lo que salió de la prueba guiada** (2026-10-06, en el Mac del cliente):
  1. Más aire arriba en la cabecera de cada sección.
  2. Los pesos, explicados y con diseño: etiquetas (Original, WebP), el ahorro grande y una barra
     que compare los dos tamaños.
  3. **Fallo**: al hacer zoom en el comparador (rueda o pellizco), la ventana entera rebota como si
     hiciera scroll. El comparador tiene que quedarse el gesto (`wheel` no pasivo con
     `preventDefault`) y nada en la ventana debe rebotar (`overscroll-behavior: none`).
  4. Con la vista reducida, la compresión no se ve: una nota «Vista reducida al X %: acércate al
     100 % para juzgar el detalle», que lleve al 100 %.
  5. La orden copiada, con las rutas completas de entrada y salida (en pantalla, corta).
  6. Sección propia de **Presets** en la barra lateral, con la lista, lo que hace cada uno y su orden,
     y desde donde aplicarlos, renombrarlos y borrarlos. Ahora están escondidos en Ajustes.
  7. «Con extras de Apolo» no se entiende: decir qué pasa («Distinto de cwebp: está enderezada»)
     con una explicación.
- **Montar la equivalencia con cwebp en CI para Windows y Linux arm64** (deuda). En un Mac con
  Apple Silicon ya la comprobó a mano el cliente: 25 de 25.

## Media

- **Entrega 4 — Lotes.** Lo siguiente de código, tras la prueba guiada. Antes era la 3; la adelantó la Identidad. Arrastrar carpetas, preset con uno o varios formatos de salida, patrón de
  nombre y carpeta, paralelo, cancelar, y el resumen con el ahorro y las peores imágenes.
- **Entrega 5 — Resto de códecs y proceso.** MozJPEG, OxiPNG, AVIF, JPEG XL y QOI; redimensionar,
  recortar y reducir paleta. La ADR de AVIF (libavif + aom frente a ravif/rav1e) se decide midiendo.
- **Entrega 6 — Métricas.** PSNR, SSIM escrito aquí, el mapa de diferencias y el mapa SSIM.
- **Entrega 7 — 1.0.** DMG, NSIS, `.deb` y AppImage; actualización automática con
  `tauri-plugin-updater` y clave minisign; README con la tabla de descargas.

## Baja

- WebP animado: `gif2webp` e `img2webp`.
- Entrada HEIC (libheif, LGPL: compatible con la GPLv3).
- Butteraugli.
- Firmar y notarizar ([ADR 0009](adr/0009-sin-firmar.md)).
- La traducción al inglés: con la i18n hecha, es traducir `es.json`.

## Cerrado

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
- ~~**Entrega 3 — Identidad**~~ (ADR 0015 y 0016): el sol con laurel elegido entre cuatro propuestas;
  el oro del sol como acento («el oro rellena, la piedra escribe»); barra lateral translúcida en
  macOS; sello, firma, bienvenida con el icono y ficha en «Acerca de»; `.dmg` con fondo propio y sin
  licencia; `make iconos` y `make ventana-dmg`. · 2026-10-06
