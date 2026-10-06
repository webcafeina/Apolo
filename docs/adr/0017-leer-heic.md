# ADR 0017 — Leer HEIC, con libheif y libde265 compiladas dentro

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

En la prueba guiada, el cliente hizo una foto con el iPhone para probar enderezar, y le salió en
HEIC, el formato por defecto del iPhone. Apolo no lo leía y hubo que convertirla con `sips`. HEIC
estaba en prioridad baja; el cliente pidió meterlo en la v0.3.2.

## Decisión

- **libheif + libde265** (el decodificador HEVC), **compiladas desde su código fuente dentro de
  Apolo**, estáticas. Van como **submódulos de git** fijados a versiones publicadas
  (`crates/heic/vendor/`: libde265 v1.1.3, libheif v1.23.6), y las compila el `build.rs` de
  `crates/heic` con CMake, con todo lo demás apagado: ningún otro códec, ni plugins, ni herramientas.
- **Enlaces escritos a mano** (`crates/heic/src/lib.rs`), solo para las ~20 funciones que se usan:
  abrir, la imagen principal, decodificarla a RGB(A) de 8 bits, y los metadatos EXIF, ICC y XMP.
- **La orientación**: libheif aplica al decodificar los giros que guarda el HEIF (`irot`, `imir`),
  así que los píxeles salen **ya derechos**. El EXIF del iPhone dice además «orientación 6», y se deja
  en 1 al leer: si no, enderezar (ADR 0012) giraría la foto otra vez. Una foto de iPhone en HEIC se
  abre derecha y sin aviso, como en Fotos o en Vista Previa.
- **cwebp no lee HEIC**: con una foto así, la orden cwebp sale como «Distinto de cwebp», con la
  explicación.

## Alternativas descartadas

- **El modo «embebido» del crate `libheif-sys`.** Compila libheif **sin decodificador HEVC**: lo
  espera instalado en el sistema. Sin él, ninguna foto de iPhone se abre (docs/trampas.md).
- **El decodificador de cada sistema** (ImageIO en macOS, WIC en Windows). En Windows, leer HEIC
  exige que la persona haya instalado las extensiones HEIF y HEVC de la tienda, que no siempre están;
  y serían tres caminos con tres resultados distintos.
- **Copiar el código fuente al repositorio** en vez de submódulos: unos 13 MB más en cada clon.

## Consecuencias

- **Compilar necesita CMake y un compilador de C++**, y clonar con submódulos
  (`git submodule update --init`). En el VPS, CMake está en `~/.local/bin`, instalado con pip, sin
  sudo. La primera compilación de `crates/heic` tarda unos dos minutos.
- **Licencias**: libheif y libde265 son LGPL-3.0, compatibles con la GPLv3 de Apolo, enlazadas
  estáticamente.
- **Patentes de HEVC**: decodificar HEVC está cubierto por patentes en algunos países. Muchos
  programas libres llevan libde265 (GIMP, ImageMagick, darktable). Para una herramienta gratuita y
  libre, el riesgo es el mismo que el suyo. **Si Apolo se vende algún día, hay que revisarlo.**

## Verificación

2026-10-06:

- La foto de ejemplo de libheif (`examples/example.heic`, 1280 × 854) se decodifica con sus colores.
  La CLI la pasa a WebP (de 718 KB a 268 KB a calidad 80) y se ve bien.
- Prueba de Playwright: el Estudio abre el HEIC y avisa de que cwebp no lo lee.
- Compila en Linux x86-64.
- **No verificado**:
  - las otras cinco plataformas; lo dirá CI, y lo más dudoso es Windows ARM64 con MSVC;
  - **una foto real de iPhone** (orientación, HDR de 10 bits, imágenes en mosaico): la tiene que
    abrir el cliente.
