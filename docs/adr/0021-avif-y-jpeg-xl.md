# ADR 0021 — AVIF y JPEG XL con avifenc y cjxl compilados dentro, y libjxl con clang

**Fecha:** 2026-10-07 · **Estado:** aceptada · desarrolla la [0020](0020-mas-formatos-y-proceso.md)

## Contexto

La [0020](0020-mas-formatos-y-proceso.md) dejó AVIF y JPEG XL para la v0.6, con la misma promesa
que el resto: **el mismo fichero que la herramienta oficial, byte a byte**. Las herramientas son:
- **avifenc** de libavif 1.4.2, que codifica con aom 3.14.1;
- **cjxl** de libjxl 0.12.0.

Las dos son grandes y en C/C++. Sus opciones son muchas (avifenc admite cualquier opción de aom
con `-a`), y trasladarlas a Rust como se hizo con cjpeg sería mucho trabajo y mucho riesgo.

Antes de decidir se midió, con prototipos en el VPS:
- **avifenc compilado con gcc** da los mismos bytes que el oficial: 40 de 40.
- **cjxl compilado con gcc no**, ni siquiera con `-ffp-contract=off`. Compilado con **clang 18**,
  el compilador con el que sale el oficial, sí: 27 de 27. libjxl calcula en coma flotante, y cada
  compilador redondea a su manera.

## Decisión

### Enlazadas dentro de Apolo

El cliente eligió la opción recomendada: **las bibliotecas van compiladas dentro**, como libwebp o
libheif, y no como programas aparte.

El crate `crates/avifjxl` las compila desde submódulos, a las versiones de los binarios oficiales:
- libavif 1.4.2, con aom 3.14.1 y libyuv 644251f;
- libjxl 0.12.0, con highway, brotli, skcms, sjpeg, libpng y zlib a sus commits.

Todo va en un solo proyecto de CMake (`crates/avifjxl/cmake/CMakeLists.txt`), para que nada vaya
dos veces:
- **zlib y libpng** van una sola vez, las de libjxl, y libavif las comparte.
- **El JPEG** es el de mozjpeg-sys, que ya está en Apolo; de él solo se toman las cabeceras.
- **sharpyuv** es el de libwebp-sys, el mismo libwebp 1.6.0 que usa avifenc. Sus dos cabeceras
  van copiadas en `crates/avifjxl/cabeceras/`.

### Los `main` de las herramientas, con otro nombre

Apolo no reimplementa avifenc ni cjxl: **compila sus `main`** (`avifenc.c`, `avifdec.c`,
`cjxl_main.cc` y `djxl_main.cc`) con otro nombre, y los llama con los mismos argumentos que la orden
que enseña. Los ficheros de entrada y de salida van por una carpeta temporal. Así la orden y el
fichero son los mismos por construcción, también con opciones que el panel no tiene.

Para que funcione dentro de una aplicación:
- **cjxl y djxl terminan con `exit()`** ante una opción mala, y eso cerraría Apolo. `build.rs` copia
  libjxl a la carpeta de compilación y, en la copia, cambia esos `exit()` por `apolo_salir()`. Esa
  función vuelve al puente (`c/puente.c`) con `longjmp`. Si `cjxl_main.cc` cambia en otra versión,
  `build.rs` falla y lo dice.
- **avifenc y avifdec escriben mucho por la salida estándar**, y no tienen `--quiet`. Cuando los usa
  Apolo por dentro, el puente tira esa salida mientras dura la llamada. cjxl y djxl se callan con
  `--quiet`.
- **Una herramienta de cada familia a la vez**: un candado para avifenc y avifdec, y otro para cjxl
  y djxl. Sus `main` no están pensados para ir dos a la vez en el mismo proceso, y cada uno ya
  reparte su trabajo en hilos.

### libjxl, con clang

En Linux, `build.rs` busca clang (`APOLO_CLANG`, `clang-18` o `clang`) y no compila sin él. En macOS
es el clang de Apple, y en Windows, clang-cl (`-T ClangCL`). Todo el proyecto de CMake va con clang,
aom y libavif incluidos: así sale igual que el oficial también.

### aom también decodifica

avifenc oficial decodifica con dav1d; Apolo usa el decodificador de aom. Decodificar AV1 es
normativo: los dos dan los mismos píxeles, y así no hace falta una biblioteca más.

### Las opciones: las del panel y las demás tal cual

`OpcionesAvif` y `OpcionesJxl` guardan las opciones que el panel enseña. **Las demás que la
herramienta admite van en `otras`, tal cual y en su orden**: Apolo conoce la tabla de opciones de
cada herramienta (cuáles llevan valor) para separar opciones de ficheros, pero no las interpreta.

- **avifenc:** en el panel van la calidad, la velocidad, sin pérdida, la calidad del alfa, el
  submuestreo, sharp YUV, el afinado, la nitidez, progresivo, los bits, el rango, premultiplicar y
  quitar EXIF, XMP o el perfil.
- **cjxl:** en el panel van la calidad (o la distancia), el esfuerzo, recomprimir el JPEG sin
  pérdida, la distancia del alfa, progresivo, el modo, el grano, el filtro de bordes, Gaborish y la
  decodificación rápida.

Con un JPEG de entrada, **cjxl lo recomprime sin pérdida** salvo que se le diga
`--lossless_jpeg=0`, y entonces no admite calidad. El panel apaga la calidad en ese caso, y Apolo
avisa con un error claro en vez del «código 1» de cjxl.

### Lo que lee cada una

Apolo le pasa a la herramienta **el PNG o el JPEG original tal cual**. Con otro formato (WebP,
HEIC, TIFF…), o con proceso, le pasa un PNG con los píxeles y el perfil, y la orden lo avisa como con
los demás formatos.

AVIF y JPEG XL **también se abren**: avifdec y djxl los pasan a PNG, con su perfil y sus metadatos.

### La CLI: son las herramientas

`apolo avif` y `apolo jxl` **le pasan los argumentos a avifenc y cjxl sin tocarlos**: ayuda, errores
y todas las opciones son los suyos. Apolo solo se pone en medio con `-apolo_preset`,
`-apolo_enderezar`, o con una entrada que la herramienta no abre.

## Alternativas descartadas

- **Los binarios oficiales al lado de Apolo.** Era la otra opción que se le dio al cliente. Daban
  la igualdad sin compilar nada, pero había que llevar seis binarios por sistema, sin firmar, y
  avifenc oficial no existe para Linux ARM ni Windows ARM.
- **Trasladar las opciones a Rust y usar las bibliotecas por su API**, como cjpeg. Hay demasiadas
  opciones, y cada versión puede cambiarlas.
- **ravif/rav1e o jpegxl-rs.** Dan ficheros válidos, pero no los de avifenc y cjxl.
- **libjxl con gcc.** Comprobado: otros bytes.
- **dav1d para decodificar.** Una biblioteca más, con ensamblador propio, para los mismos píxeles.

## Consecuencias

- **Compilar Apolo necesita clang en Linux**, y en el VPS es un LLVM 18 sin instalar (docs/trampas.md).
  CI lo instala con `llvm.sh 18`.
- La primera compilación de `apolo-avifjxl` tarda unos 4 minutos. `build.rs` copia libjxl sin
  borrar lo anterior, para que las siguientes no recompilen todo.
- **Subir libavif, aom o libjxl exige subir la referencia a la vez**, como con las demás.
- Lo que la herramienta tuviera reservado cuando sale con `exit()` se pierde. Solo pasa con órdenes
  que la herramienta rechaza, y es poco.
- Callar avifenc tira, durante la llamada, lo que **cualquier** hilo de Apolo escriba por la salida
  estándar. Apolo escribe por la de errores.
- `make referencias` descarga avifenc (zip) y cjxl (`.tar.lz`). Para no depender de lzip, un Python
  de la biblioteca estándar lo descomprime (`pruebas/referencias/deslzip.py`).
- djxl escribe los trozos sRGB y gAMA del PNG en otro orden que el djxl oficial. Los píxeles son
  los mismos, y es lo que usa Apolo.

## Verificación

2026-10-07, en el VPS, con `make equivalencia`. Cada combinación pasa por todo el camino de Apolo:
leer la orden, volver a escribirla y llamar a la herramienta compilada dentro.

| Herramienta | Iguales |
|---|---|
| avifenc 1.4.2 | **376 de 376**: los PNG del corpus (color, gris, alfa, 16 bits, paleta, gamma, sRGB, perfiles válidos y rotos, metadatos) y sus JPEG, con 21 combinaciones de opciones. Entre ellas, sin pérdida, los cuatro submuestreos, sharp YUV, 10 y 12 bits, rango limitado, afinado, nitidez, `-a` de aom, progresivo, mosaico y `--min/--max` |
| cjxl 0.12.0 | **280 de 280**: los mismos ficheros con 20 combinaciones. Entre ellas, la recompresión sin pérdida de los JPEG, `--lossless_jpeg=0`, esfuerzos de 1 a 9, modular, grano, filtros, remuestreo y contenedor |

Las demás siguen iguales: cwebp, cjpeg, oxipng y qoiconv.

**A mano:**
- `apolo avif` y `apolo jxl` dan los mismos bytes que avifenc y cjxl: 9 de 9 y 6 de 6.
- Una opción mala de cjxl (`--center_x` sin `--group_order=1`) devuelve el código 1 y Apolo
  sigue vivo.
- avifdec da el mismo PNG que el oficial.

**Pruebas:**
- **Núcleo:** ida y vuelta de las dos órdenes, lo que se rechaza, y los seis formatos salen y se
  vuelven a leer.
- **e2e:** la orden y los controles de los dos, exportar y volver a abrir un AVIF y un JPEG XL, y
  pegar una orden de cjxl. Son 49 en total (eran 43).
- **Licencias**, todas compatibles con la GPLv3 (mirado el LICENSE de cada submódulo):
  - libavif, aom, libyuv, libjxl y skcms son BSD; aom, con su licencia de patentes de AOMedia;
  - highway y sjpeg, Apache-2.0;
  - brotli, MIT;
  - libpng y zlib, las suyas.

**Después, en el Mac del cliente** (2026-10-08, Apple Silicon, v0.6.1):
- la prueba guiada entera en la ventana: AVIF y sus controles, JPEG XL con un JPEG (sin pérdida y
  con calidad), comparar AVIF con WebP, exportar y volver a abrir con transparencia, y un lote con
  WebP y AVIF y «el más ligero» (24 convertidas, 18 WebP y 6 AVIF, como en el VPS);
- `apolo avif` frente al avifenc 1.4.2 oficial de Mac: **16 de 16 iguales**.
- La prueba sacó un error: «Sin pérdida» con la calidad tocada antes. Arreglado en la v0.6.1 (un
  control apagado no cuenta en la orden).

**No verificado:**
- **La equivalencia de JPEG XL en macOS, y la de los dos en Windows.** Allí libjxl sale del clang de Apple y de clang-cl, que no
  son el clang 18 del cjxl oficial de Linux. Hay binarios oficiales de avifenc para macOS y Windows,
  y de cjxl para Windows, con los que se podría comprobar en CI.
- **Rutas con caracteres no ASCII en Windows.** Los `main` abren los ficheros temporales con
  `fopen`, que en Windows no entiende UTF-8 sin un manifiesto. Si la carpeta temporal del usuario
  lleva tildes, podría fallar (deuda).
- Los mapas de ganancia de los JPEG, porque falta libxml2 (deuda).
- Lotes grandes en AVIF: tiempo y memoria.
