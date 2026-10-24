# ADR 0032 — Recortar y redimensionar, sobre la imagen y comunes a la imagen

**Fecha:** 2026-10-23 · **Estado:** aceptada · v0.18.0 · matiza la [0020](0020-mas-formatos-y-proceso.md) y la [0023](0023-mcp.md)

## Contexto

Hasta la v0.17.0, Recortar y Redimensionar estaban dentro del desplegable «Proceso» del panel
derecho del Estudio, junto a Enderezar y Reducir paleta, debajo de Formato, Partir de y Guardar.
Cada lado del comparador tenía los suyos. El cliente pidió, antes de la prueba guiada final, que
estén a la vista: son de lo más usado.

Al leer el código salieron tres cosas más:
- **Una redimensión que no hacía nada costaba la orden.** Con «nunca agranda» (el agente), pedir
  1920 en una foto de 1200 no cambiaba el tamaño, pero `salida::motivo` solo miraba
  `proceso.vacio()`: la imagen salía «Procesada por Apolo», sin orden equivalente, y JPEG perdía el
  camino de cjpeg y JPEG XL la recompresión sin pérdida.
- **`solo_reducir` no se veía en la ventana** (deuda.md).
- **El recorte sobrevivía al abrir otra imagen**, con las coordenadas de la anterior.

Planteado el 2026-10-23 en modo plan, con tres rondas de preguntas y un agente de diseño.

## Decisión

**Decidido con el cliente** (2026-10-23):

| Tema | Decisión |
|---|---|
| Dónde | Una barra bajo la cabecera del Estudio, encima del lienzo: [Recortar] [Redimensionar · 1920 px de lado mayor ▾] |
| Alcance | Comunes a los dos lados: se recorta y se redimensiona la imagen, y los dos lados comparan formatos sobre lo mismo |
| Redimensionar | Un botón que dice el tamaño y abre una ventanita: el lado mayor, el porcentaje o ancho × alto con candado; los tamaños habituales (3840, 2560, 1920, 1280, 800; 75, 50, 25 %); cómo quedará; «Permitir agrandar»; «Más» (el filtro y RGB lineal); Quitar y Listo |
| Agrandar | Nunca, por defecto (`solo_reducir`); la casilla lo permite |
| Presets | El lado mayor y el porcentaje se guardan como tales. El recorte no se guarda. Un preset sin tamaño conserva el de la barra |
| Lo demás | Enderezar y Reducir paleta se quedan en «Proceso»; `-resize` y `-crop` de cwebp, como estaban |
| Lotes | Un «Tamaño» para todas las salidas, encima del preset, como la nota objetivo. Un lote no recorta nunca, aunque lo traiga un preset de antes |
| CLI | `apolo lote --lado-mayor N` y `--porcentaje P`, encima del preset |
| Revisión | Galería de capturas de los cinco aspectos (https://claude.ai/artifact/74j6quMT3n3XCiUHpM3VrC), aprobada el 2026-10-24 |

**El modelo** (`crates/nucleo/src/proceso.rs`): `Redimension { medida, filtro, lineal,
solo_reducir }` con `Medida::{Medidas { ancho, alto }, LadoMayor(u32), Porcentaje(f64)}`. Se guarda
plana, con `modo` y todos los campos (`RedimensionJson`, con `try_from`/`into`). Un preset de antes,
sin `modo`, se deduce de lo que trae; sin `solo_reducir`, sigue a `false`, y hace lo que hacía. Se
rechazan un porcentaje ≤ 0 o NaN y un lado mayor de 0. El lado mayor es el ancho si la imagen es
apaisada o cuadrada, y el alto si es vertical; el porcentaje, a los dos lados con un mínimo de 1 px.

**El proceso efectivo**: `Proceso::efectivo(w, h)` quita el recorte que coge la imagen entera y la
redimensión que la deja igual; `salida::efectivo(img, a)` lo aplica con las medidas ya enderezadas.
De él salen los bytes (`codificar`, `referencia`), el motivo y la orden (`codificar_con` del
servicio y el agente).

**En el Estudio** (`Estudio.tsx`): un estado `comun { recorte, redimension }`. Los ajustes de cada
lado se guardan sin ellos y se componen al codificar (`useMemo`, o la vista previa codificaría en
cada dibujo). Al abrir otra imagen, el recorte se quita y el tamaño se queda. Componentes nuevos:
`BarraImagen.tsx`, `VentanaTamano.tsx` (`SelectorTamano`, también en Lotes), `Emergente.tsx` (la
ventanita genérica) y `tamano.ts` (la cuenta de Rust, con la misma tabla de pruebas).

**La ventanita, propia**: sin el atributo `popover` ni `<dialog>`, que el WKWebView de macOS 11 no
tiene (trampas.md). Se pinta junto a su botón con `position: absolute`, se abre hacia arriba si
abajo no cabe y arriba sí (mirando lo que la recorta, no solo la ventana), tiene `role="dialog"`, y
se cierra con Escape (el foco vuelve al botón), un clic o el foco fuera.

**En cada sistema**, su lenguaje: CommandBar y Flyout (Windows), barra secundaria de 38 y popover
de vidrio (macOS), barra de herramientas y GtkPopover (GNOME), ToolButtons y emergente de Breeze
(KDE), con sus iconos (Fluent, Yaru y Breeze, con sus licencias). En macOS y Linux, la barra tiene su
fila en la rejilla del Estudio.

**El agente** (`crates/mcp`): `lado_mayor` y `porcentaje` en optimizar, comparar y el lote, siempre
sin agrandar; errores `tamano_doble`, `porcentaje_fuera` y `lado_mayor_cero`; `guardar_preset`
rechaza `recorte` (`recorte_en_preset`) y quita el de un preset base; `empezar_lote` ya no tiene
`recorte`.

## Alternativas descartadas

- **Arriba del panel, siempre a la vista**, o **una pestaña «Imagen»** en el panel: menos visibles;
  el cliente eligió la barra sobre la imagen, como en un editor de fotos.
- **Cada lado con su tamaño**, como hasta ahora: permite comparar 1920 contra 1280, pero cuesta
  entender sobre qué actúa la barra, y el deslizador compara peor.
- **Los atajos como píxeles** (el preset guarda 1920 × 1440): en Lotes, una foto vertical saldría
  con el ancho de la horizontal.
- **Campos opcionales con un orden de preferencia** en el núcleo, en vez del enum: admiten
  combinaciones imposibles y la preferencia habría que repetirla en cada sitio.
- **Agrandar por defecto**, como Squoosh: agrandar empeora la imagen, y el agente ya no agrandaba.
- **El `popover` nativo o `<dialog>`**: no están en el WKWebView de macOS 11.

## Consecuencias

- Una Apolo anterior no entiende el lado mayor ni el porcentaje de un preset nuevo: no redimensiona,
  sin avisar.
- Un preset antiguo sin `solo_reducir` agranda, y su ventanita lo enseña con «Permitir agrandar»
  marcado.
- Cuando la redimensión no hacía nada, los bytes cambian respecto a la v0.17 (por ejemplo, JPEG
  conserva sus marcadores): es lo que debía pasar.
- Al cambiar el tamaño se recodifican los dos lados a la vez.

## Verificación

- `make comprobar` y e2e entero (147) en verde. Pruebas nuevas: en el núcleo (lado mayor apaisado,
  vertical y cuadrado; porcentajes; `solo_reducir`; el JSON de antes y un preset fijo de la v0.17;
  ida y vuelta; `efectivo`, y los mismos bytes con una redimensión que no hace nada); en el lote del
  núcleo (no recorta, sí redimensiona); en el agente (lado mayor, porcentaje, errores, preset sin
  recorte, orden equivalente sin agrandar); en `apolo lote`; `tamano.test.ts` con la tabla de Rust;
  e2e de la barra, la ventanita con el teclado, el candado, los dos lados, los presets, Lotes (leyendo
  el IHDR) y el alto de la barra y el radio de la ventanita en cada sistema.
- Galería de capturas de los cinco aspectos en claro y oscuro, revisada y aprobada por el cliente
  el 2026-10-24.
- **No verificado**: nada en una ventana de verdad todavía (va en la compilación de prueba); macOS 11
  con su WKWebView antiguo; la memoria al medir los dos lados a la vez con fotos de 24 MP; `make
  evaluar-mcp` con los dos escenarios nuevos (`o-lado-mayor`, `o-porcentaje`), sin correr porque gasta.
