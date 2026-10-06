# ADR 0011 — Leer las imágenes como las lee cwebp

**Fecha:** 2026-10-06 · **Estado:** aceptada

## Contexto

La ADR 0002 promete el mismo fichero que cwebp. Con libwebp enlazada, la codificación es la misma;
lo que puede cambiar son **los píxeles de partida**. cwebp los lee con libpng, libjpeg-turbo,
libtiff y su propio código de `imageio/`, y cada uno tiene sus manías. Apolo no lleva esas
librerías: lee con crates de Rust (`png`, `tiff`, `image`) y con mozjpeg.

## Decisión

**Cada lector de Apolo repite lo que hace el de cwebp**, y donde el crate hace otra cosa, se corrige
encima. Lo que hubo que copiar, porque sin ello el fichero salía distinto:

- **JPEG**: se decodifica con **mozjpeg**, que usa el mismo decodificador que libjpeg-turbo (DCT
  entera lenta y `do_fancy_upsampling`, sus valores por defecto). Con otro decodificador, los
  píxeles difieren en la última cifra y la salida también.
- **PNG, gamma**: libpng con `png_set_gamma(2.2, gamma_del_fichero)` cuando hay `sRGB` o `gAMA`.
  Con `sRGB` no cambia nada; con un `gAMA` que no sea 1/2,2 cambian los píxeles, con la aritmética
  de libpng (punto fijo de 1e-5, umbral del 5 %, tabla de 8 bits).
- **PNG, ICC**: libpng **descarta los perfiles ICC que no pasan sus comprobaciones** (longitud,
  `acsp`, espacio de color acorde con el PNG, clase, PCS, tabla de etiquetas) con solo un aviso, y
  cwebp ni se entera. Apolo hace las mismas comprobaciones y descarta los mismos.
- **TIFF, alfa**: libtiff, al leer en RGBA, **premultiplica el alfa normal** (no asociado), y cwebp
  solo deshace la premultiplicación cuando el fichero dice que el alfa ya venía asociado. O sea,
  cwebp codifica los TIFF con transparencia con el color oscurecido. Es una rareza de cwebp, y se
  repite, con la tabla `UaToAa` de libtiff.
- **WebP de entrada**: no se decodifica a RGBA; se decodifica directamente al espacio del
  codificador (YUV o ARGB), como `webpdec.c`.
- **Metadatos**: solo se leen si se van a copiar, como en cwebp. Un fallo al leerlos hace fallar la
  lectura entera. En la interfaz se querrá leerlos siempre; eso será lectura con reintento, no un
  cambio aquí.

Y lo que Apolo hace **de más**, sin romper la promesa:

- Lee **GIF, BMP y QOI**, que cwebp no lee. Con esos no hay con qué compararse.
- Lee TIFF a los que les falta la etiqueta `ExtraSamples`, que cwebp rechaza.

Y lo que **no** hace por defecto: **no aplica la orientación EXIF.** cwebp tampoco. (El cliente pidió
después tenerlo como opción: [ADR 0012](0012-enderezar-como-opcion.md).) Una foto de móvil
girada sale girada, igual que con cwebp. Si la interfaz quiere enderezarlas, será una opción propia
de Apolo, marcada como tal y fuera de la orden cwebp equivalente.

## Alternativas descartadas

- **Enlazar libpng, libjpeg-turbo y libtiff** como cwebp. Sería la fidelidad por construcción, pero
  son tres librerías C más que compilar en seis objetivos, y libtiff arrastra zlib, libjpeg y
  liblzma. Copiar las manías concretas cuesta menos, y la prueba de equivalencia vigila que no se
  escape ninguna.

## Consecuencias

- Cada manía copiada está comentada en su lector, con el nombre de la función de C que imita.
- Lo que el corpus no cubre puede estar mal sin que nadie lo sepa: está en [deuda.md](../deuda.md).

## Verificación

2026-10-06, en Linux x86-64: **1015 comparaciones idénticas byte a byte** contra el `cwebp` 1.6.0
oficial de Google (29 imágenes × 35 combinaciones de opciones), con `make equivalencia`.

- El corpus cubre PNG RGB, RGBA, gris, gris con alfa, 16 bits, paleta con tRNS, `gAMA` 1,0 y 0,45,
  `sRGB`, metadatos (iCCP válido y roto, eXIf, iTXt XMP, «raw profile» en tEXt); JPEG normal,
  4:4:4, progresivo, gris y con EXIF, XMP e ICC troceado y desordenado; TIFF RGB y RGBA con alfa
  normal y asociado; PNM P5, P6 y P7; WebP con pérdida, con alfa y sin pérdida; YUV crudo de tamaño
  impar.
- Hasta llegar ahí fallaron tres cosas, que son las de arriba: el ICC roto, el alfa de TIFF y un
  TIFF mal formado del propio corpus.
- **No comprobado**: el resto de plataformas, sobre todo ARM con NEON, donde libwebp usa otro
  código SIMD. Google publica cwebp también para macOS, Windows y Linux arm64, así que se puede
  comparar en las máquinas de CI; todavía no se ha montado. Ver [deuda.md](../deuda.md).
