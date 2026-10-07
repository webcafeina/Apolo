# ADR 0003 — WebP entero, y el resto de Squoosh

**Fecha:** 2026-10-06 · **Estado:** aceptada · MozJPEG, OxiPNG y QOI hechos en la [0020](0020-mas-formatos-y-proceso.md)

## Contexto

Se le dieron al cliente tres alcances: solo `cwebp`; la suite libwebp entera (`dwebp`, `gif2webp`,
`img2webp`, `webpmux`); o WebP más el resto de los códecs de Squoosh.

## Decisión

**WebP con todo lo de `cwebp`, más MozJPEG, OxiPNG, AVIF, JPEG XL y QOI**, que son los formatos de
salida de Squoosh (menos WebP2, que es experimental y Google lo dejó). Las entradas son las que
cualquiera trae: JPEG, PNG, GIF, WebP, AVIF, JXL, TIFF, BMP y QOI.

## Alternativas descartadas

- **Solo `cwebp`.** Lo mínimo, pero no sustituye a Squoosh: el día que haga falta un JPEG o un AVIF
  se vuelve a la web.
- **La suite libwebp entera como alcance inicial.** El WebP animado queda en
  [siguiente.md](../siguiente.md) con prioridad baja: no es lo que se hace a diario.

## Consecuencias

- La interfaz tiene que poder pintar las opciones de seis códecs distintos sin que cada uno sea una
  pantalla nueva: los controles se describen como datos.
- AVIF queda por decidir entre libavif + aom (lo de Squoosh) y ravif/rav1e: se decide midiendo, en
  su ADR.

## Verificación

Elegida con el cliente el 2026-10-06.
