# ADR 0038 — Vectorizar: banco de pruebas y motor propio

**Fecha:** 2026-11-09 · **Estado:** en curso (fase 1, investigación)

## Contexto

El cliente quiere que Apolo vectorice imágenes automáticamente, como Vectorizer.ai (Cedar Lake Ventures),
y plantearlo bien antes de construir. Decidido con él (2026-11-09): todos los tipos de imagen (logotipos e
iconos, ilustraciones, blanco y negro para corte, fotos y pixel art); SVG, PDF, EPS y DXF; **banco de
pruebas primero**; la IA, se decide después; un mes de suscripción a Vectorizer.ai **solo para comparar**
(sus condiciones prohíben entrenar modelos con sus salidas, y se respeta); corpus solo con licencia libre.

Lo que se sabe de su motor sale de su web, su API y sus opciones de salida, y de mirar sus SVG: redes
neuronales y geometría clásica; bordes por debajo del píxel según el suavizado; figuras de verdad
(`<circle>`, `<ellipse>`, `<rect>`); arcos exactos en los huecos; muchas Bézier cuadráticas; estrellas como
polígonos de esquinas exactas; y un trazo de 2 px bajo cada borde, del color mezcla, para tapar rendijas
(«gap filler»).

## Decisión

Un **laboratorio fuera de la aplicación**, `laboratorio/vectorizar/` (proyecto Cargo propio, excluido del
workspace: ni la CI ni `make comprobar` lo compilan). `cargo run --release` pasa 53 imágenes libres
(`corpus/LICENCIAS.md`; las fotos las baja `corpus/descargar.sh`) por cada motor, rasteriza cada SVG con
**resvg** y lo mide con **`apolo medir`** (la SSIMULACRA 2 de libjxl, la de Apolo). Escribe
`salida/resultados.csv` y una galería. Los SVG de Vectorizer.ai, si están en
`capturas/vectorizar/vectorizer-ai/<caso>.svg` (fuera de git), entran como un motor más.

Motores: VTracer 1.0 (MIT o Apache-2.0) con sus presets; Potrace 1.16 (GPL-2.0-or-later, compilado dentro,
`vendor/potrace`) en blanco y negro y por capas; y los nuestros. **El que vale es `apolo-poli`**:

1. **Paleta automática** con los colores de los píxeles planos (los del borde son mezcla del suavizado).
2. **Contorno por debajo del píxel**: cada píxel de borde es mezcla de dos colores de la paleta, y el
   contorno de cada capa es la línea de nivel 0,5 de su cobertura («marching squares»). Es lo que
   Vectorizer.ai describe como colocar los bordes según el suavizado.
3. **Polígono primero** (al estilo de PolyFit, Dominici y otros, 2020): el contorno se afina a polígono;
   los lados largos con una esquina en un extremo son rectas, y la esquina va al cruce de las rectas; las
   rachas de lados cortos son Bézier si ahorran nodos; las puntas romas entre dos rectas se afilan.
4. **Círculos y elipses de verdad** cuando el contorno lo es (elipse: Halíř y Flusser, 1998, con una
   comprobación geométrica de 0,35 px); formas de menos de 10 px, como polígono.
5. **Pixel art** (pocos colores, sin suavizado y en bloques) se traza a píxeles.
6. **Puntas finas**: una punta roma entre dos rectas se prolonga hasta su cruce si está cerca (3 px) o,
   hasta 12 px, si los píxeles del tramo que se añade están a medio cubrir (hay un rasgo de menos de un
   píxel que el contorno al 50 % se comió).

## Alternativas descartadas

- **VTracer tal cual**: 47,7 de nota media; mezcla colores y redondea.
- **Asignar cada píxel de borde a un color entero**: o deja halos (al color global más parecido), o se
  come las puntas finas (al color de alrededor). Por eso la mezcla.
- **Detectar esquinas en el contorno denso y ajustar Bézier entre ellas** (motor `apolo`): se le escapan
  esquinas suaves y las redondea; 77,4 frente a 80,6 del polígono primero.
- **Afinar tolerancias**: entre 0,2 y 0,3 px la nota apenas cambia; no es lo que falta.
- **Entrenar con salidas de Vectorizer.ai**: prohibido por sus condiciones.

## Consecuencias

Con las 18 imágenes que se le pasaron a Vectorizer.ai (sin fotos), al cierre del 2026-11-09:

| | Nota media | Nodos | Peso medio |
|---|---|---|---|
| SVG original | 97,5 | 57 | — |
| Vectorizer.ai | 83,1 | 555 | 18,6 KB |
| **apolo-poli** | **80,7** | 520 | **8,8 KB** |
| VTracer | 47,7 | 255 | 7,0 KB |

- **Ganamos o empatamos en 6 de 18**: Japón (93,7 con 5 nodos), los dos pixel art (100), la Unión Jack sin
  suavizar, la cara de OpenMoji y la paleta de pintor. En pixel art, Vectorizer.ai suaviza los píxeles.
- **Perdemos sobre todo en**: las estrellas pequeñas (Estados Unidos, 61 frente a 77), Brasil y Corea, las
  ilustraciones de Noto con degradados y la pluma.
- **Fotos**: nadie las vectoriza bien (notas negativas, también Vectorizer.ai). Avisar al usuario.
- **Vectorizer.ai tampoco pasa de 83 de media**: el techo realista con esta medida está entre 85 y 90.

- **Relleno de huecos** («gap filler» de Vectorizer.ai): no hace falta. Nuestras capas se apilan y cada
  una cubre también las de encima, así que entre dos colores no queda rendija.
- **Las estrellas pequeñas** siguen siendo la pérdida mayor. Prolongar las puntas validándolas con el
  suavizado ayudó poco (61,5 → 64,0 en Estados Unidos); ajustar las rectas solo con puntos «fiables»,
  nada (se deshizo). Lo que falta es mirar la forma entera: el borde de dos brazos y la cabeza de una
  estrella es una sola recta, y las cinco puntas son simétricas. Es lo que Vectorizer.ai llama detectar
  simetrías y optimizar el «grafo vectorial»: la siguiente pieza grande, si se sigue.
- **Rectángulos**: un rectángulo ya sale como 4 rectas con esquinas exactas; escribirlo como `<rect>` no
  cambia la imagen. Se deja para la salida definitiva (fase 2), por si se edita.

Lo siguiente: decidir con el cliente la fase 2 (núcleo, CLI, MCP e interfaz, y PDF, EPS y DXF), si se
sigue mejorando el motor (simetrías, degradados) y si hace falta IA.

## Verificación

- La referencia (el SVG original rasterizado) da 100 frente a su propio PNG en todos los casos
  suavizados: la cadena resvg → `apolo medir` es fiel. (La bandera sin suavizar da 55: el original es
  dentado y el SVG, no.)
- Pruebas unitarias del laboratorio: el ajuste de Bézier sobre un arco perfecto se desvía 0,14 px
  (tolerancia 0,3); el polígono de un cuadrado sale con sus 4 esquinas.
- **Comprobado a mano**, mirando los píxeles: las puntas de las estrellas, los halos, la cara de OpenMoji
  y la Unión Jack.
- **En un navegador** (Chromium, con Playwright), seis casos: los SVG se ven bien (círculos, elipses y
  huecos). La nota cambia hasta 3 puntos según el visor (Chromium y resvg suavizan distinto: entre ellos
  dan de 81 a 89); como los SVG de Vectorizer.ai se miden igual, la comparación es justa.
- **No verificado**: que los SVG se vean igual en Illustrator o Inkscape;
  la velocidad en imágenes grandes (las de 512 px tardan menos de 1 s); el comportamiento con
  transparencias (todo se aplana sobre blanco).
