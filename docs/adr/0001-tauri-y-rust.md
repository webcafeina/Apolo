# ADR 0001 — Tauri 2 y Rust

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

Apolo es una aplicación nativa para macOS, Windows y Linux. La referencia de la casa es Esfinge, que
es Go + Wails (su ADR 0001) y de la que se copian las convenciones. Pero lo que Apolo hace es
distinto: su trabajo son los códecs de Squoosh, y esos son C/C++ (libwebp, MozJPEG, libavif/aom,
libjxl) o directamente Rust (OxiPNG, libimagequant 4, rav1e).

## Decisión

**Tauri 2 con el núcleo en Rust**, y la interfaz en React 19 + Vite + TypeScript con pnpm, igual
que Esfinge. Un workspace de Cargo con el núcleo (`crates/nucleo`) separado de la ventana
(`src-tauri`) y de la CLI (`crates/cli`), para que las dos caras usen el mismo código.

Cada códec entra por un crate que **compila su librería C dentro del binario** (`vendored`): ni una
dependencia del sistema en la máquina del usuario.

## Alternativas descartadas

- **Wails + Go, como Esfinge.** Coherencia máxima con la casa, pero cada códec iría por cgo —y la
  puerta de Esfinge compila con `CGO_ENABLED=0`, que aquí no se podría— y OxiPNG no tiene
  equivalente en Go: habría que llevarlo como binario aparte.
- **Electron.** Lleva Chromium entero en cada instalador; Tauri usa la vista web del sistema.

## Consecuencias

- Los instaladores pesan poco, pero la vista web es distinta en cada sistema (WebKit en macOS y
  Linux, WebView2 en Windows). La interfaz no puede depender de nada exótico.
- En Linux la aplicación necesita `libwebkit2gtk-4.1`. Debian 12 y Ubuntu 22.04 la traen; por debajo,
  no ([ADR 0010](0010-plataformas.md)).
- Compilar los códecs pide cmake y nasm en la máquina que compila, no en la que instala.

## Verificación

Elegida con el cliente el 2026-10-06. **Nada compilado todavía** en el momento de escribirla.
