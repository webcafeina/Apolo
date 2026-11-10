# ADR 0039 — Vectorizar en Apolo (fase 2)

**Fecha:** 2026-11-09 · **Estado:** en curso (hito 1 de 9 hecho)

## Contexto

El banco de la fase 1 (ADR 0038) dejó un motor propio a 1,2 puntos de Vectorizer.ai (81,9 frente a 83,1
de nota SSIMULACRA 2 en 18 imágenes), con la mitad de peso. El cliente aprobó llevarlo a Apolo
(2026-11-09): una **pantalla propia, «Vectorizar»**, con opciones sencillas y avanzadas; **modo por nota**;
**SVG, PDF, EPS y DXF** en la ventana, la CLI y el MCP; y **todo junto en la 1.1.0**. El plan, por hitos,
está en la sesión del 2026-11-09 (`docs/sesiones.md`).

## Decisión

- **Una operación aparte, no un `FormatoSalida` más.** El Estudio, `medir`, los lotes y los presets suponen
  una salida en mapa de bits del tamaño del original (unos 25 `match` en Rust y 7 mapas en TypeScript). La
  vista previa sí es un mapa de bits, el documento rasterizado con resvg, así que `medir`, el comparador y
  el modo por nota se reutilizan.
- **El motor, en `crates/vector` (`apolo-vector`)**, trasladado del laboratorio sin variables de entorno ni
  código de depuración: `vectorizar(&Rgba, &OpcionesVector) -> Documento`. Un **modelo propio**
  (`Documento`: fondo y capas con rectas, Bézier, círculos, elipses, y rellenos de color o degradado), del que
  saldrán los cuatro formatos.
- **`TipoImagen::Auto`** decide: pixel art (pocos colores exactos, en bloques o pequeño), **foto si hay más de
  0,05 colores distintos por píxel** (en el banco, las fotos dan de 0,13 a 0,25 y los dibujos, 0,007 como
  mucho), y si no, ilustración. Mirar si había zonas planas no servía: una foto de producto sobre fondo
  blanco las tiene y se trataba como ilustración (23 minutos y peor nota).
- **Pixel art con un trazado propio** sobre la cuadrícula, en vez de VTracer. **Sin Potrace ni VTracer en la
  aplicación**: solo el banco los usa. La elipse, con un 3 × 3 propio en vez de nalgebra.
- Dependencia nueva: **resvg** 0.49 (Apache-2.0 o MIT; sus transitivas, permisivas: revisadas en el banco).
- **Los cuatro formatos, con escritores propios** (hito 3, 2026-11-10; `exportar(&Documento, FormatoVector,
  &OpcionesExportar)`):
  - **PDF 1.4 y EPS 3 salen del mismo contenido** (`pintura.rs`): los operadores de PDF, que el EPS define con
    los mismos nombres en un diccionario propio (`ApoloDict`, para no pisar los del programa que lo
    incruste). El eje Y se da la vuelta con una transformación, así que las coordenadas son las del SVG. Los
    degradados, como sombreados nativos (tipos 2 y 3, con funciones cosidas para más de dos paradas). El PDF
    va comprimido con Deflate (`miniz_oxide`, la del núcleo). Un píxel, un punto.
  - **DXF R2007** con la estructura que pide AutoCAD desde R2000 (las nueve tablas, bloques de modelo y papel,
    diccionario raíz, grupos y diseños; identificadores y dueños). Una capa de DXF por color, con su color
    real (código 420) y el ACI más parecido. Sin rellenos ni degradados: contornos, y cada capa con el color
    medio. **Tres niveles**: solo rectas (polilíneas, curvas aplanadas a 0,05 px), rectas y arcos (polilíneas
    con abombamiento, curvas aproximadas por arcos a 0,05 px, y círculos) y rectas, arcos y splines
    (círculos, elipses y una spline de grado 3 por forma, cada Bézier entre nudos triples).
  - **Tres estilos**: rellenar; trazar formas (el contorno de cada forma, del color de su capa); y **trazar
    bordes**, para cortar: las capas están apiladas, así que donde dos colores se tocan pasan varios
    contornos casi por el mismo sitio, y se quita cada tramo cuyos puntos caen todos a menos de 0,75 px de
    otro ya dibujado (de arriba abajo). También los tramos del marco de la imagen (sería cortar el borde de
    la hoja). En negro por defecto: el color de la capa de encima puede ser el del fondo y no se vería.

- **Núcleo y servicio** (hito 4, 2026-11-10): `apolo_nucleo::vector` con `AjusteVector` (tipo, colores,
  paleta, detalle, objetivo, degradados, área mínima y las opciones de exportar; en JSON, para la ventana, la
  CLI y el MCP), `vectorizar(imagen, ajuste, progreso) -> Vectorizado` y `exportar`. Antes de vectorizar, la
  imagen **se endereza** según su EXIF, **se reduce si pasa de 2 Mpx** (como Vectorizer.ai; el dibujo sale
  del tamaño reducido) y se aplana sobre blanco: esa es la referencia de la nota. **Modo por nota**: el
  detalle más bajo (el dibujo más ligero) cuya vista previa llega a la nota, partiendo en dos de `detalle`
  (el mínimo) a 100; como mucho ocho pruebas. En el servicio, el último dibujo de cada imagen, con su
  generación para cancelar; exportar con otras opciones de dibujo (estilo, grosor, nivel de DXF) **no vuelve
  a vectorizar** (`AjusteVector::mismo_dibujo`). La vista previa, lo que recibió el motor y el mapa de
  diferencias, por `apolo://` (`vector`, `vector-referencia`, `vector-mapa`) y por el servidor de desarrollo.
  El «detalle» mueve las tolerancias y el área mínima, **no el número de colores** (el plan lo preveía): la
  paleta cambia el dibujo entero y se deja como opción aparte.

- **La CLI** (hito 5, 2026-11-10): `apolo vectorizar ENTRADAS… [-o SALIDA]`. Una imagen deja el dibujo al lado
  sin pisar (o en `-o`, y el formato sale de la extensión); varias o carpetas, a una carpeta con las
  subcarpetas, como los lotes («iconos-svg»). Cada opción, en `docs/cobertura-vectorizar.md`. Dos cosas que
  salieron al probarla con imágenes reales: **más detalle no siempre da más nota** (la búsqueda se queda con el
  más ligero que llega, o el de mejor nota; no con el detalle más bajo) y **con los colores limitados no hay
  degradados** (con dos colores, un cuadrado azul y un círculo rojo acababan en un degradado de azul a rojo).

## Alternativas descartadas

- **SVG como un formato más del Estudio**: el cliente prefirió una pantalla propia, y además tocaba todo el
  camino de los formatos de mapa de bits.
- **Copiar los motores libres** (VTracer, Potrace): 47,7 y 68,7 de nota frente a 81,9.

## Consecuencias

- El banco usa ya el crate (motor `apolo-crate`) y da **lo mismo que el laboratorio**: 81,9 en las 18, y
  por grupos 84,8 / 73,0 / 81,3 / 100 (logotipos, ilustraciones, blanco y negro, pixel art). Las fotos, de
  23 minutos a unos 7 s.
- ~~**Lo lento es pintar las capas** (hito 2)~~: en la bandera de EE. UU., unos 10 s por cada uno de los dos
  órdenes de capas, y casi todo era la síntesis (19,3 de 20 s). **Hecho el 2026-11-10**: la síntesis cuenta
  las submuestras por filas y guarda el error de cada píxel, y las capas, sus formas y los dos órdenes van en
  paralelo con rayon. La bandera, de 20 s a 1,1 s; la foto más lenta, 3,9 s (docs/rendimiento.md). El
  resultado es el mismo bit a bit.

## Verificación

- `cargo test -p apolo-vector`: arco en un tramo, elipse girada (y un cuadrado que no lo es), círculo
  exacto, cuadrado con sus cuatro esquinas y pixel art con hueco. Clippy sin avisos.
- El banco entero con `APOLO_MOTORES=apolo-crate` (31 s): mismas notas que el motor del laboratorio.
- `APOLO_TIPOS=1` en el banco: las 53 imágenes, con el tipo bien detectado.
- Hito 2 (2026-11-10): el corpus entero con `apolo-poli` y `apolo-crate` a la vez da las mismas notas en
  todo menos en dos fotos, que ya diferían antes (las fotos no pasan por la síntesis). El SVG de la bandera
  de EE. UU., byte a byte igual que antes del cambio. Prueba `barrido_igual_que_submuestras`: el barrido por
  filas cuenta lo mismo que la prueba de punto en polígono, píxel a píxel. Tiempos solo en el VPS (4
  núcleos), no en un Mac.
- Hito 3 (2026-11-10), `crates/vector/tests/formatos.rs`: cada formato rasterizado por su cuenta y comparado
  con el SVG de resvg, en los tres estilos. **PDF** con `pdftoppm`: diferencia media de 0,4 a 2,1 (de 255).
  **DXF**, los tres niveles, con un lector de la prueba que aplica el significado del estándar (abombamiento,
  de Boor, eje y razón de la elipse): 0,1 a 0,4. Identificadores únicos, dueños existentes y `$HANDSEED`
  mayor que todos; la tabla xref del PDF, a byte. En el banco (`APOLO_FORMATOS=1`), el PDF de las 53 imágenes
  frente al SVG: de 0,05 a 1,8, salvo el pixel art pequeño (unos 5: Poppler pinta gris donde dos cuadrados
  se tocan por la esquina, trampas.md). «Trazar bordes», mirado a ojo en tres imágenes.
- **El EPS, en la CI** (aquí no hay Ghostscript; `APOLO_FORMATOS_ESTRICTO=1`): frente al SVG de resvg daba de
  1,5 a 3,1 de media, por el suavizado de Ghostscript en los trazos finos, así que se compara con el **PDF
  pintado también por Ghostscript** (mismos operadores: menos de 0,3 de media) y frente al SVG con margen
  (menos de 4,5). Pasa desde `b847666`.
- **No verificado**: el **DXF en un programa de CAD o de corte de verdad** (sí con ezdxf 1.4.2, con
  permiso del cliente: lectura, auditoría y recuperación sin un error ni un arreglo en los 240 DXF del banco
  —tres niveles y dos estilos— y en la prueba `dxf_valido_para_ezdxf`, que la CI pasa con `uvx`; falta
  abrirlo en AutoCAD, LibreCAD o el programa de corte del cliente); los cuatro
  formatos en **Illustrator e Inkscape**.
- Hito 4: pruebas del núcleo (un círculo; por nota, que llega con nueve pruebas como mucho; una imagen de
  1.600 × 1.600 se reduce a menos de 2 Mpx; cancelar; colores; el ajuste en JSON) y del servicio (vectorizar,
  medir, exportar a los cuatro formatos, reexportar con otro estilo sin vectorizar, una generación vieja se
  cancela, nombres). La ruta `apolo://` de la ventana, compilada en la CI.
- Hito 5: `crates/cli/tests/vectorizar.rs` (al lado sin pisar; formato por extensión, los cuatro; carpetas
  con subcarpetas; colores, paleta, estilo, grosor, color de trazo, pixel art, detalle; por nota; errores de
  opciones con salida 2 y una imagen rota con salida 1; ayuda en inglés). El modo por nota, medido en
  imágenes del banco (docs/rendimiento.md): de 0,2 a 3 s, 6,9 s si el objetivo no se alcanza. Nada de los
  hitos 6 a 9 (MCP, pantalla, web).
