# ADR 0002 — libwebp enlazada, y la misma salida que cwebp

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

El encargo es «todo lo que haga `cwebp`, de manera rica y visual». `cwebp` es una herramienta de
terminal sobre libwebp; Squoosh usa la misma libwebp compilada a WebAssembly. Para que la vista
previa sea en vivo —cambiar la calidad y ver el resultado al momento, con progreso y pudiendo
cancelar— hay que hablar con la librería, no con un proceso.

## Decisión

**libwebp se enlaza dentro de Apolo** (`libwebp-sys` con la librería compilada dentro), con su
versión **fijada**. Toda opción de `cwebp` se traduce a los campos de `WebPConfig` y `WebPPicture`
que toca, y queda registrada en [cobertura-cwebp.md](../cobertura-cwebp.md).

Y la promesa se comprueba: con las mismas opciones, **`apolo webp` tiene que dar el mismo fichero,
byte a byte, que el `cwebp` oficial de la misma versión**. Esa prueba corre en CI con el binario de
Google descargado y su sha256 verificado.

## Alternativas descartadas

- **Llevar el binario `cwebp` oficial** y llamarlo. Más simple al principio, pero cada vista previa
  sería un proceso nuevo con ficheros temporales, sin progreso fino, y Apolo dependería de lo que
  `cwebp` saque por pantalla.

## Consecuencias

- Subir de versión libwebp es una decisión: cambia los bytes y obliga a subir a la vez el `cwebp` de
  referencia de la prueba.
- Donde la salida no pueda ser idéntica —se sospecha de `-mt`—, se compara por PSNR y se apunta aquí.

## Verificación

- 2026-10-06: `libwebp-sys 0.14.4` compila y enlaza **libwebp 1.6.0** (`apolo --version`, y la
  prueba `libwebp_esta_enlazada`). El `cwebp` de referencia tiene que ser, por tanto, el 1.6.0.
- **Pendiente** la equivalencia byte a byte: es la prueba principal de la entrega 1.
