# ADR 0022 — Medir la pérdida con SSIMULACRA 2 de libjxl, el mapa de diferencias y la calidad por nota

**Fecha:** 2026-10-08 · **Estado:** aceptada · desarrolla la [0005](0005-extras.md)

## Contexto

La [0005](0005-extras.md) prometió el mapa de diferencias con PSNR y SSIM, con el SSIM escrito aquí
porque `dssim` es AGPL, y dejó Butteraugli en prioridad baja porque no había una biblioteca suelta
con licencia cómoda.

Desde la [0021](0021-avif-y-jpeg-xl.md), libjxl va dentro de Apolo, y sus herramientas traen
**SSIMULACRA 2** y **Butteraugli**, con licencia BSD. SSIMULACRA 2 es la medida que más se parece a
lo que ve el ojo, y la que usa el proyecto JPEG XL para comparar codificadores. Su autor la explica
en una escala de 0 a 100: 90 no se distingue ni alternando las dos imágenes, 80 no se distingue lado
a lado, 70 es alta calidad, 50 media y 30 baja.

## Decisión

El cliente eligió cuatro respuestas, tres de ellas las recomendadas.

### Las medidas: SSIMULACRA 2, con PSNR y SSIM al lado

- **SSIMULACRA 2** es la nota principal. Es la de libjxl: Apolo compila su `tools/ssimulacra2.cc`
  dentro (`crates/avifjxl/c/medir.cc`) y la llama con los píxeles en memoria.
  - Da **la misma nota que la herramienta `ssimulacra2` oficial**.
  - Con transparencia hace lo mismo que la herramienta: mide sobre fondo oscuro (0,1) y claro (0,9),
    y vale la peor de las dos.
- **PSNR y SSIM**, escritos aquí (`crates/nucleo/src/medir.rs`), como cifras técnicas:
  - el PSNR, de los tres canales juntos;
  - el SSIM, el de Wang y otros (2004), sobre la luminancia BT.601 con ventana gaussiana de 11 y
    σ = 1,5;
  - con transparencia, sobre los mismos dos fondos, y la peor.
- **Butteraugli no**: es mucho más lenta, y SSIMULACRA 2 ya dice lo que el ojo ve.

Se mide contra **lo que recibió el codificador**, la referencia (`salida::referencia`): la imagen
enderezada y procesada. Si el resultado tiene otro tamaño, como con `-resize` de cwebp, no se mide, y
se dice.

### Un modo «Diferencias» en el comparador

Junto a Deslizador y Lado a lado, un tercer modo pinta el mapa de calor del lado que se edita. Lleva
la referencia apagada en gris debajo, y encima el calor, de rojo oscuro a amarillo y blanco. Hay dos
mapas:
- **Píxeles que cambian**: la mayor diferencia de los cuatro canales, por 4.
- **Detalle que se pierde**: 1 − SSIM local, por 2. Con 4 saturaba: una nota de 80 parecía mala por
  todas partes.

### En Lotes, opcional

«Medir la calidad» está apagado por defecto. Encendido, cada fila lleva su nota, y el resumen da la
nota media y las cinco de peor nota. Solo se mide lo que se escribe: con «el más ligero», el que gana.

### La calidad por nota, en esta entrega

El cliente eligió hacerlo ya, y no en una entrega aparte, que era lo recomendado.

`Ajuste.objetivo` es una nota SSIMULACRA 2. En los formatos con pérdida, Apolo busca **la calidad más
baja que llega a esa nota**:
- parte en dos el intervalo de 0 a 100: siete pruebas, y una más si ni la 99 llega;
- la 100 solo se prueba si hace falta, porque es la más lenta, y en avifenc es sin pérdida.

**La orden lleva la calidad encontrada**, así que sigue dando el mismo fichero que la herramienta
oficial. La promesa de las ADR 0002, 0020 y 0021 no se rompe.

Qué formatos la admiten:

| Formato | Calidad que se busca | Cuándo no |
|---|---|---|
| WebP | `-q` | Sin pérdida |
| JPEG | `-quality` | — |
| AVIF | `-q` | `-l` |
| JPEG XL | `-q` | Con un JPEG que cjxl recomprime sin pérdida |
| PNG y QOI | — | Siempre: no tienen calidad |

Dónde se usa:
- **En el Estudio**: «Calidad por nota» en Básico, con 80 por defecto. Mientras está encendida, la
  calidad del formato se apaga («La busca Apolo»).
- **En Lotes**: una nota para todas las salidas, encima de su preset. Cada imagen tiene su calidad.
- **En la CLI**: `-apolo_objetivo NOTA` en `apolo webp`, `jpeg`, `avif` y `jxl`, y `apolo lote --objetivo NOTA`.

### Lo que decidí yo

- **Una medida SSIMULACRA 2 a la vez en todo Apolo**, con un candado. Usa mucha memoria: 1,75 GB con
  una foto de 12 MP, y la herramienta oficial lo mismo. Ocho a la vez, en un lote, serían 14 GB.
- **En el Estudio, la medida va después de la vista previa**, en una petición aparte (`medir`).
  - Con una foto grande tarda segundos, y así no retrasa ver el resultado.
  - Lleva la generación de la vista: si entretanto llega otra, se cancela antes de lo caro.
- **La CLI gana `apolo medir original resultado`**: las tres medidas de dos ficheros.
- **La búsqueda supone que más calidad da más nota.** Es lo normal. Si en algún tramo no lo fuera, la
  calidad que sale llega a la nota igual, aunque quizá no sea la más baja posible.

## Alternativas descartadas

- **Solo PSNR y SSIM**, lo planeado en la 0005. Se parecen menos a lo que se ve.
- **Butteraugli**: muy lenta para una vista previa en vivo.
- **Medir dentro de la vista previa**: retrasaría verla segundos con una foto grande.
- **Medir en paralelo sin límite**: la memoria se dispara en los lotes.
- **Buscar la nota midiendo una versión reducida**, que es más rápido: daría otra nota que la de la
  herramienta, y la calidad encontrada no sería la que se promete.
- **`dssim`**: es AGPL (0005).

## Consecuencias

- **La búsqueda cuesta varias codificaciones y medidas por imagen.** En el VPS, unos 22 s para una
  foto de 2400 × 1600 en AVIF. En un lote con nota objetivo eso se multiplica; en el Mac del cliente,
  bastante menos.
- **Medir es caro en memoria con fotos grandes** (deuda). Si en un Mac de 8 GB molesta, la salida es
  medir por franjas, o reducir solo para la vista previa y avisarlo.
- La CLI, el Estudio y Lotes dan la misma nota: es la misma función del núcleo.
- La referencia se calcula al pedir la primera medida o el primer mapa de cada vista, y se guarda
  hasta la siguiente.

## Verificación

2026-10-08, en el VPS:

- **SSIMULACRA 2 igual que la herramienta oficial** (`ssimulacra2` de libjxl 0.12.0):
  - **56 de 56** con `make equivalencia`: cada imagen del corpus frente a su JPEG con calidad 30 y 75;
  - a mano, 18 de 18 a 8 decimales, con foto, logo con transparencia y captura, a tres calidades, en
    JPEG y WebP.
- **La búsqueda**, probada en los cuatro formatos con pérdida:
  - llega a la nota;
  - con una calidad menos ya no llegaría;
  - la calidad encontrada, codificada aparte, da el mismo fichero;
  - PNG no la admite.
- **La CLI**:
  - `apolo avif -apolo_objetivo 80` encuentra la calidad 83, con nota 80,2 en 7 pruebas, y
    `apolo medir` da 80,15 sobre el fichero escrito;
  - `apolo lote --objetivo 85 --mas-ligero` da una nota media de 85,5.
- **Pruebas:**
  - **núcleo:** medidas idénticas, el orden de las notas con más ruido, la transparencia, los mapas y
    la búsqueda;
  - **e2e:** la nota y la búsqueda en el Estudio (la orden lleva la calidad encontrada), el modo
    Diferencias con sus dos mapas, y un lote que mide y busca la nota. Son 53 en total (eran 49).
- **Capturas**, en claro y en oscuro: medidas, Diferencias y Detalle.

**No verificado:**
- en la ventana de verdad, en el Mac del cliente;
- la memoria y el tiempo con fotos de 24 MP en un Mac;
- que la búsqueda dé la calidad más baja en todos los casos: se supone que la nota crece con la
  calidad.
