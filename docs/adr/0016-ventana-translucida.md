# ADR 0016 — Ventana translúcida en macOS, con `macOSPrivateApi`

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

La identidad «nativa con marca» ([ADR 0015](0015-identidad-nativa-con-marca.md)) pide la barra
lateral con el vidrio de macOS, como Esfinge (su ADR 0017). En Tauri 2 eso exige dos cosas:

- la ventana `transparent`, con el efecto `sidebar` (`NSVisualEffectMaterialSidebar`);
- en macOS, la transparencia solo se activa con **`macOSPrivateApi`** y la característica
  `macos-private-api` de `tauri`, porque usa API privada de Apple.

## Decisión

- **macOS**: `tauri.macos.conf.json` hace la ventana transparente, con `windowEffects: ["sidebar"]`,
  la barra de título superpuesta (`titleBarStyle: "Overlay"`, `hiddenTitle`) y los semáforos sobre la
  barra lateral. `app.macOSPrivateApi` va a `true` en la configuración común, y la característica en
  `src-tauri/Cargo.toml`. Así no hace falta repetirlo por sistema, y en los demás no hace nada.
- **Windows y Linux, opacos por ahora**. Tauri admite Mica en Windows 11, pero en Windows 10 una
  ventana transparente sin efecto deja ver el escritorio, y no hay un Windows a mano para probarlo.
  Linux sin compositor se queda opaco, igual que en Esfinge (su ADR 0020).
- Una orden **`plataforma`** devuelve el sistema y si hay vidrio. La interfaz lo pone en
  `<html data-sistema data-vidrio>` antes de pintar, y el CSS cuelga de ahí. En el navegador de
  desarrollo es `web` y sin vidrio.
- Con vidrio, **todo lo que es trabajo lleva fondo propio** (`--lienzo`). Solo la barra lateral deja
  ver el material.

## Alternativas descartadas

- **Sin vidrio.** Es justo lo que el cliente echó en falta frente a Esfinge.
- **El crate `window-vibrancy` a mano.** Hace lo mismo que `windowEffects` y también necesita la
  ventana transparente; queda como plan B si al probarlo el webview sale opaco (Esfinge tuvo que
  poner `setOpaque:NO` y el `underPageBackgroundColor` a mano).

## Consecuencias

- **`macOSPrivateApi` impide publicar en la Mac App Store.** Apolo se distribuye por GitHub
  (ADR 0014), así que hoy no importa. Si algún día va a la tienda, hay que quitar el vidrio o
  hacerlo de otra forma.
- Windows pierde, por ahora, el Mica que sí tiene Esfinge: en la deuda.

## Verificación

2026-10-06:

- En el navegador, con Playwright: sin vidrio, `data-vidrio="no"` y el `body` opaco.
- CI compila la ventana con la característica `macos-private-api`.
- **No verificado**: que en un Mac de verdad se vea el vidrio y no un fondo opaco o un agujero.
  Lo tiene que mirar el cliente con la v0.3.0.
