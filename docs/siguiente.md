# Lo siguiente

Última actualización: **2026-10-07**

El plan entero salió de la sesión del 2026-10-06, decidido con el cliente en cuatro rondas de
preguntas (ver [decisiones.md](decisiones.md)). Va **por entregas**; cada una se cierra con sus
pruebas en verde y su línea en [sesiones.md](sesiones.md).

## Alta

- ~~Publicar la v0.4.0~~ (2026-10-07). **Que el cliente actualice desde la v0.3.3** con el
  actualizador: es la primera vez que se ve de verdad (deuda). La clave del actualizador se queda en el VPS por decisión del
  cliente (deuda).

- **Montar la equivalencia con cwebp en CI para Windows y Linux arm64** (deuda). En un Mac con
  Apple Silicon ya la comprobó a mano el cliente: 25 de 25.

## Media

- **Entrega 5 — Resto de códecs y proceso.** MozJPEG, OxiPNG, AVIF, JPEG XL y QOI; redimensionar,
  recortar y reducir paleta. La ADR de AVIF (libavif + aom frente a ravif/rav1e) se decide midiendo.
- **Entrega 6 — Métricas.** PSNR, SSIM escrito aquí, el mapa de diferencias y el mapa SSIM.
- **Entrega 7 — 1.0.** DMG, NSIS, `.deb` y AppImage; README con la tabla de descargas. (La
  actualización automática se adelantó a la v0.3.3: ADR 0018.)

## Baja

- WebP animado: `gif2webp` e `img2webp`.
- Butteraugli.
- Firmar y notarizar ([ADR 0009](adr/0009-sin-firmar.md)).
- La traducción al inglés: con la i18n hecha, es traducir `es.json`.

## Cerrado

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
