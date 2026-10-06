# Sesiones

Bitácora: qué se hizo en cada sesión, la más reciente arriba. La plantilla está al final.

---

## 2026-10-06 (tarde) · Entrega 1: el mismo fichero que cwebp

- Se leyó `examples/cwebp.c` y los lectores de `imageio/` de libwebp 1.6.0 enteros, y se portaron:
  el bucle de opciones (`cwebp::leer`, con `strtol` en base 0 y `-preset` que reescribe lo
  anterior), el `main()` desde la lectura hasta el fichero (`webp::codificar`), los metadatos
  (`metadatos::escribir`) y los lectores (`entrada/`).
- JPEG con mozjpeg (el decodificador de libjpeg-turbo). nasm compilado de fuente en `~/.local`, sin
  sudo.
- **`make equivalencia`**: descarga el cwebp 1.6.0 de Google, comprueba su sha256 y compara 29
  imágenes generadas × 35 combinaciones. Primera pasada: 908 iguales. Fallaban el ICC (libpng
  descarta perfiles inválidos) y el TIFF (libtiff premultiplica el alfa). Corregido: **1015 de
  1015 idénticos**. Corre también en CI (trabajo `equivalencia`).
- Pruebas sin cwebp: sin pérdida conserva los píxeles, redimensionar, cancelar y las opciones en
  JSON. Ida y vuelta preset ⇄ orden cwebp en 12 combinaciones.
- ADR 0011 (leer como cwebp). Cinco trampas nuevas y siete apuntes de deuda, casi todos «sin
  comprobar».
- **No verificado**: la equivalencia fuera de Linux x86-64; que `publicar.yml` siga compilando con
  mozjpeg en las seis máquinas (se lanzó al subir).

---

## 2026-10-06 · Planteamiento y cimientos

- **Planteamiento con el cliente**, en cuatro rondas de preguntas. De ahí salen las diez primeras
  ADR: Tauri 2 + Rust, libwebp enlazada, WebP más el resto de Squoosh, Estudio y lotes, los extras
  (redimensionar, paleta, CLI, mapa de diferencias), **GPLv3 y repositorio público** (empezó como
  privado y a la venta; cambió a gratuito y público, y eso permitió usar libimagequant), aspecto del
  sistema, español con i18n, sin firmar por ahora y seis objetivos con ARM.
- Por qué no Go + Wails como Esfinge: los códecs son C/C++ y Rust; por cgo serían un suplicio y
  OxiPNG no existe en Go ([ADR 0001](adr/0001-tauri-y-rust.md)).
- **Entrega 0**: documentos vivos con el sistema de Esfinge; workspace de Cargo (`nucleo`, `tema`,
  `cli`, `src-tauri`); interfaz React con i18n y la carcasa (Estudio, Lotes, Ajustes); tokens
  generados desde Rust con la prueba de contraste; icono provisional; `comprobar.yml` y
  `publicar.yml`; repositorio `webcafeina/Apolo`.
- Verificado aquí: `make comprobar` en verde (fmt, clippy, pruebas de núcleo y tema, contraste,
  tokens al día, build y test de la interfaz). `apolo --version` dice **libwebp 1.6.0**. La primera
  paleta no pasaba el contraste en 14 parejas; se ajustó contra la peor superficie de cada modo.
- **No verificado aquí**: la ventana. El VPS no tiene `libwebkit2gtk-4.1-dev` y no hay sudo; se
  compila en CI ([deuda](deuda.md)).
- Rust instalado con rustup en `~/.cargo`, sin tocar el sistema.
- **CI**: `comprobar.yml` en verde a la primera, con la ventana. `publicar.yml` cayó en los cinco
  trabajos por `bundle.category` («Graphics» → «GraphicsAndDesign», a [trampas](trampas.md)), y a la
  segunda salió verde: DMG universal, NSIS x64 y ARM64, `.deb` y AppImage amd64 y arm64.

---

## Plantilla

```
## AAAA-MM-DD · <título>

- Qué se hizo o se decidió.
- Qué se verificó, y con qué.
- Qué queda abierto (→ mover a siguiente.md o deuda.md si procede).
```
