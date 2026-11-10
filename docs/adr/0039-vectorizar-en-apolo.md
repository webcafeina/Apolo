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
- **No verificado aún**: nada de los hitos 3 a 9 (formatos, núcleo, CLI, MCP, pantalla, web).
