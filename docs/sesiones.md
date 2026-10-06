# Sesiones

Bitácora: qué se hizo en cada sesión, la más reciente arriba. La plantilla está al final.

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
