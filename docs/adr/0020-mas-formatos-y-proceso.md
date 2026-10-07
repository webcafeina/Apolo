# ADR 0020 — JPEG, PNG y QOI con el mismo fichero que sus herramientas; el proceso; un formato por lado

**Fecha:** 2026-10-07 · **Estado:** aceptada · desarrolla la [0003](0003-alcance-squoosh.md) y la [0005](0005-extras.md)

## Contexto

La entrega 5 trae el resto de formatos de Squoosh (MozJPEG, OxiPNG, AVIF, JPEG XL y QOI) y el
proceso de la imagen: redimensionar, recortar y reducir la paleta. Con WebP, Apolo da **el mismo
fichero que cwebp, byte a byte**, y enseña la orden (ADR 0002). Había que decidir:
- qué se promete con los demás formatos;
- cómo se comparan entre sí;
- cómo encajan en Lotes.

## Decisión

El cliente eligió las cuatro respuestas recomendadas.

### En dos versiones

- **v0.5:** MozJPEG, OxiPNG y QOI, el proceso, el comparador con un formato por lado y Lotes con
  varios formatos.
- **v0.6:** AVIF y JPEG XL, que traen librerías en C grandes (aom, libjxl) y más riesgo al compilar
  en los seis sistemas.

### El mismo fichero que la herramienta oficial de cada formato

Como con cwebp: misma orden, mismo fichero, y una prueba de equivalencia por formato.

**JPEG: el `cjpeg` de MozJPEG 4.1.5.** Apolo traslada a Rust su `main` y su `parse_switches`
(`crates/nucleo/src/jpeg/cjpeg.rs`), con:
- las dos pasadas por las opciones;
- su propio `jpeg_default_qtables` (con el ABI 62 vive en cjpeg, no en la librería);
- el submuestreo que `-quality` quita desde 80 y desde 90;
- el factor de escala truncado a entero.

La imagen se lee **como la lee cjpeg**, que no es como la lee cwebp:
- el gris se queda en gris;
- la transparencia se descarta sin componerla;
- la gamma de un PNG no se corrige;
- se copian sus marcadores: el perfil de un PNG, o el perfil sRGB mínimo de cjpeg si el PNG trae
  `sRGB`; todos los APPn y COM de un JPEG.

Sin `-arithmetic`, como el oficial, que viene compilado sin ella.

**PNG: `oxipng` 10.2.1.** OxiPNG optimiza un PNG, no codifica píxeles:
- con un PNG de entrada, Apolo se lo pasa entero;
- con otro formato, hace antes un PNG con sus píxeles y su perfil.

Las opciones son las de su línea de órdenes, convertidas como su `parse_opts_into_struct`.
libdeflater va fijada a la 1.26.0 del `Cargo.lock` del oxipng publicado.

**QOI: `qoiconv`**, la herramienta del autor del formato:
- lee el PNG como stb_image: sin gamma, 16 bits al byte alto, y 3 canales solo si el PNG es RGB o
  de paleta sin transparencia;
- Apolo traslada `qoi_encode` de qoi.h. El crate `qoi` da ficheros válidos, pero otros.

Las órdenes que se enseñan son las de cada herramienta:
- `cjpeg … -outfile salida.jpg entrada`;
- `oxipng … --out salida.png entrada`;
- `qoiconv entrada salida.qoi`.

La CLI gana `apolo jpeg`, `apolo png` y `apolo qoi`, que aceptan esas opciones tal cual.

### El comparador, con un formato por lado

- Cada lado del Estudio elige su formato y sus ajustes, como Squoosh.
- El izquierdo empieza siendo el original; «Comparar con otro formato» le pone uno.
- La orden, los pesos y exportar son del lado que se edita.
- El servicio codifica por imagen **y lado**, cada uno con su contador de generación (el de la
  0013, ahora por lado).

### En Lotes, varios formatos, con la opción de guardar solo el más ligero

- Una salida por formato pedido, cada una con su preset.
- **Sin la opción**, un fichero por formato; dos salidas del mismo formato no se pisan (`-2`).
- **Con «solo el más ligero»**, se codifican todas en memoria y se escribe la que menos pese.
- El resumen desglosa por formato.
- La carpeta propuesta lleva el sufijo de lo pedido: `-webp`, `-jpg`… o `-apolo` si son varios.

Y lo que decidí yo:

- **El proceso** (`crates/nucleo/src/proceso.rs`) va antes de codificar, para cualquier formato, en
  el orden de Squoosh:
  - **enderezar**: pasa aquí desde las opciones de WebP; ahí se queda solo para `-apolo_enderezar`;
  - **recortar**;
  - **redimensionar**: con `fast_image_resize`, Lanczos3 por defecto, transparencia premultiplicada
    y mezcla en RGB lineal;
  - **reducir la paleta**: con libimagequant, la de Squoosh, por defecto a 256 colores y con todo el
    tramado.
- **Con proceso, la orden de la herramienta ya no da ese fichero**: ninguna redimensiona como Apolo.
  El aviso lo dice («Procesada por Apolo»), igual que «Enderezada» o «cjpeg no abre WebP».
- **`salida::Ajuste`** guarda las opciones de **todos** los formatos, no solo las del elegido:
  cambiar de WebP a JPEG y volver no pierde nada. Un preset es un nombre y un `Ajuste`, aplanado en
  el JSON. Los presets de antes (`{nombre, formato, webp}`) se siguen leyendo.
- El `-resize` y el `-crop` de cwebp siguen, en Experto, como «Redimensionar como cwebp»: sirven
  para repetir una orden de cwebp.

## Alternativas descartadas

- **Solo las opciones de Squoosh, sin prometer igualdad.** Más rápido, pero la marca de Apolo es
  enseñar la orden y que dé el mismo fichero.
- **El crate `mozjpeg` con su API de alto nivel.** No expone todo lo de cjpeg, y su orden de
  llamadas no es el de cjpeg: daría ficheros parecidos, no iguales.
- **El crate `qoi`.** Da QOI válidos pero no los de qoi.h. Trasladar el codificador son 60 líneas.
- **Una sola v0.5 con todo.** Si AVIF o JPEG XL se atascan en un sistema, retrasan lo demás.
- **Comparar siempre con el original.** No deja decidir entre WebP y AVIF viéndolos.
- **El proceso como opciones de cada formato.** Habría cuatro «redimensionar» distintos, y cada
  herramienta lo hace a su manera.

## Consecuencias

- Tres herramientas de referencia más que preparar: `make referencias`.
  - **cjpeg** se compila desde la fuente de MozJPEG 4.1.5, con una libpng también compilada, porque
    Mozilla no publica binarios.
  - **oxipng** se instala con `cargo install --locked`.
  - **qoiconv** se compila de commits fijados.
- Subir MozJPEG, OxiPNG o libdeflater exige subir la referencia a la vez, o la prueba falla.
- Lo que cjpeg lee pero no está comprobado que Apolo lea igual (BMP, GIF, Targa) cuenta como «cjpeg
  no abre…». Va a la deuda.
- Con proceso, el perfil ICC del original se conserva al reconstruir los píxeles (en JPEG y PNG).

## Verificación

2026-10-07, en el VPS, con `make equivalencia`:

| Herramienta | Iguales |
|---|---|
| cwebp 1.6.0 | 1015 de 1015, sin cambios |
| cjpeg 4.1.5 | 816 de 816: PNG en color, gris, con transparencia, de 16 bits, de paleta, con gamma, con sRGB, con perfiles válidos y rotos; JPEG con y sin metadatos y en gris; PNM. Cruzados con 41 combinaciones de opciones, las de orden delicado incluidas |
| oxipng 10.2.1 | 351 de 351: los PNG del corpus con 27 combinaciones |
| qoiconv | 13 de 13 |

En el camino, la prueba cazó tres errores míos (trampas.md):
- el perfil sRGB copiado a mano, con 4 bytes de menos;
- la codificación aritmética, que el oficial no trae;
- los límites de QOI mal traducidos.

**A mano**, `apolo jpeg`, `apolo png` y `apolo qoi` dan los mismos bytes que las tres herramientas
con imágenes de la carpeta de prueba.

**Pruebas:**
- **Núcleo:** el proceso, la salida (los cuatro formatos se leen, el camino de JPEG es el de cjpeg,
  y el motivo), los presets de antes, y Lotes con varios formatos y con el más ligero.
- **Servicio:** las órdenes de las cuatro herramientas, los dos lados y el motivo.
- **e2e:** cada formato con su orden, pegar una orden de cjpeg, comparar dos formatos, el proceso, y
  Lotes con dos formatos (todos y el más ligero). Son 43 en total.
- **Licencias:** libimagequant GPLv3 (ADR 0006); lo demás MIT, Apache o IJG.

- **CI**, el mismo día: `comprobar` (con la ventana de Tauri compilada), `equivalencia` (cjpeg
  compilado desde la fuente en el runner, oxipng instalado, qoiconv compilado: las cuatro
  herramientas iguales) y `e2e`, en verde.

**No verificado:**
- la ventana abierta: diálogos, exportar a cada formato, el comparador con dos formatos. Lo verá el
  cliente;
- BMP, GIF y Targa en cjpeg;
- lotes grandes con varios formatos: memoria y tiempo;
- que la mezcla en RGB lineal de `fast_image_resize` trate la transparencia como Squoosh.
