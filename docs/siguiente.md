# Lo siguiente

Última actualización: **2026-10-06**

El plan entero salió de la sesión del 2026-10-06, decidido con el cliente en cuatro rondas de
preguntas (ver [decisiones.md](decisiones.md)). Va **por entregas**; cada una se cierra con sus
pruebas en verde y su línea en [sesiones.md](sesiones.md).

## Alta

- **Entrega 2 — Estudio.** Comparador con deslizador, lado a lado y diferencias; zoom y
  desplazamiento sincronizados; controles en tres niveles (Básico, Avanzado, Experto) con su
  explicación; vista previa en vivo con generación y descarte; la orden cwebp equivalente, copiable y
  pegable; exportar. Las vistas previas por `apolo://preview/<id>`, sin base64 por IPC. Y un modo
  de desarrollo por HTTP para probar la interfaz con Playwright sin ventana, como el puente de dos
  caminos de Esfinge (su ADR 0009).
- **Montar la equivalencia con cwebp en macOS, Windows y Linux arm64** (deuda): Google publica
  cwebp para esas plataformas, y es donde libwebp y mozjpeg cambian de código SIMD.

## Media

- **Entrega 3 — Lotes.** Arrastrar carpetas, preset con uno o varios formatos de salida, patrón de
  nombre y carpeta, paralelo, cancelar, y el resumen con el ahorro y las peores imágenes.
- **Entrega 4 — Resto de códecs y proceso.** MozJPEG, OxiPNG, AVIF, JPEG XL y QOI; redimensionar,
  recortar y reducir paleta. La ADR de AVIF (libavif + aom frente a ravif/rav1e) se decide midiendo.
- **Entrega 5 — Métricas.** PSNR, SSIM escrito aquí, el mapa de diferencias y el mapa SSIM.
- **Entrega 6 — 1.0.** Icono definitivo, DMG, NSIS, `.deb` y AppImage; actualización automática con
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
