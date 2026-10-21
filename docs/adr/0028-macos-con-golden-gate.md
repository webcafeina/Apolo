# ADR 0028 — macOS con el diseño de macOS 27 Golden Gate

**Fecha:** 2026-10-21 · **Estado:** aceptada · v0.14.0 · matiza la [0015](0015-identidad-nativa-con-marca.md) y la [0016](0016-ventana-translucida.md)

## Contexto

Con Windows ya como una aplicación de WinUI ([ADR 0027](0027-windows-con-fluent.md)), al cliente le
gustó «casi más que la de Mac». En macOS, Apolo seguía con la anatomía de Esfinge de 2025: controles
de 28 con radio 9, barra lateral de 225 con la fila activa en oro, cabecera propia de 64 y un
material de barra lateral sutil. Pidió hacer en Mac lo mismo que en Windows: **las reglas de diseño
de macOS 27 Golden Gate**, sin cambiar lo que hace Apolo.

macOS 27 Golden Gate salió el 2026-09-14, solo para Apple silicon. Refina Liquid Glass (macOS 26
Tahoe): barras laterales de borde a borde (ya no flotan) con los iconos en el color de acento,
barras de herramientas uniformes, radio de ventana menor, más contraste. La HIG no publica medidas
de controles; están en el **kit oficial de Apple para Figma**, que el cliente duplicó en su cuenta y
se leyó con el conector de Figma (`get_metadata`, `get_variable_defs`, capturas).

## Decisión

Todo cuelga de `:root[data-sistema="macos"]` (`frontend/src/sistema-macos.css`), y el marcado cambia
con `esMac()` y `formularioNativo()` solo donde hace falta. La web de desarrollo, Linux y Windows no
lo ven.

**Decidido con el cliente** (2026-10-21):

| Tema | Decisión |
|---|---|
| Acento | El oro de Apolo con las formas de macOS 27 |
| Iconos de la barra lateral | En el **oro hecho línea** (`--oro-trazo`): el oro oscurecido lo justo para llegar a 3:1 sobre la barra (un objeto gráfico), como el acento de macOS 27 en los suyos. Matiza la regla de que el oro nunca es línea: solo aquí, y nunca para texto |
| Parte de arriba | **Barra de herramientas unificada** de 52: la cabecera de cada sección se hace barra (título en Headline 13 negrita y su subtítulo en 11), con las órdenes en cápsulas de vidrio de 36. En el Estudio cubre el lienzo y el inspector (`display: contents` en la columna del lienzo) |
| Barra lateral | De borde a borde, 240, filas de 32 («medium») con el icono de 16 a 18 px y el nombre a 8, la elegida en el relleno gris con radio 8. Sin la marca ni la firma. **Se oculta entera** con su botón (cápsula de 36), animada y recordada (`useLateral`); abre visible |
| Controles | Los del kit: 24 de alto, botón con relleno gris y radio 6 (no cápsula en el contenido), pop-up como botón, campo con su filete, casilla de 16, interruptor en cápsula de 36 × 16 con el botón más ancho que alto, deslizador con pista de 6 y botón en cápsula, segmentado en cápsula con el elegido en blanco |
| Vidrio en los controles | Las cápsulas de la barra con brillo en el filo, sombra interior y desenfoque, y un leve abombado al pulsar. Al pulsar, el botón del deslizador se hincha en una burbuja de vidrio que desborda la pista; el del interruptor se estira y se aclara (no puede salirse: es el fondo del control) |
| Páginas | Ajustes, Lotes (Calidad y Dónde) y MCP en **filas agrupadas** como Ajustes del Sistema: el mismo marcado que las SettingsCards de Windows (`TarjetaAjuste`), con filas de 42, 10 de relleno, el control a 16, grupos de radio 10 con filetes sangrados. Pestañas de Ajustes, centradas |
| Versión nueva | Un aviso que flota bajo la barra, en vidrio |
| Vidrio de la ventana | **Liquid Glass** (`NSGlassEffectView`, estilo `Regular`, el público) en macOS 26 y 27; en 11–15, el material de barra lateral de siempre |
| Paleta | `macos_claro` y `macos_oscuro` en `crates/tema`, con los neutros del kit (fondo de ventana, `Fills - Vibrant`, `Labels`) y el contraste medido |

**La ventana** (`src-tauri/src/ventana_macos.rs`). Sale de `tauri.macos.conf.json`, transparente y
con la barra superpuesta, con los semáforos en `trafficLightPosition` (x 19, y 24) para la barra de
52. En `setup`, `window_vibrancy::apply_liquid_glass` y, si no puede (antes de macOS 26),
`apply_vibrancy` con el material de barra lateral; guarda si hubo vidrio y `plataforma` lo cuenta. Se
quitó `windowEffects`, que se traga el error. No hace falta crear la ventana desde código, a
diferencia de Windows.

**Medidas leídas del kit** (Figma `uCn3dR3MxX8byi1d2oFmJD`; detalle en la sesión del 2026-10-21):
escala de controles 16 / 20 / 24 / 28 / 36; barra unificada 52 con márgenes 18 y 8, grupos en cápsula
de 36 con relleno 4 y botones de 28; semáforos de 14 con 23 entre centros, el primero a 26 del borde;
barra lateral 240, filas de 24 / 32 / 36, iconos de 16; botón de 24 con relleno 16 y radio 6;
interruptor 36 × 16 (formularios) a 80 × 36; pista del deslizador de 6; fila de formulario 42 con 10
de relleno y 16 hasta el control; menú 6 por debajo del botón. Estimadas (el límite de consultas del
asiento de Figma cortó antes): el radio del grupo de formulario (10), el del campo (6), el botón del
interruptor y del deslizador en reposo, y los colores de separador y de control.

## Alternativas descartadas

- **El acento del sistema (azul)**: Apolo dejaría de reconocerse; el cliente quiso el oro.
- **La cabecera de 64 de antes, retocada**: no es macOS 27; el cliente eligió la barra unificada.
- **El material de barra lateral en todas las versiones**: más sutil y menos de macOS 27.
- **SF Symbols**: su licencia limita el uso; incrustarlos en Apolo es zona gris. Se quedan los
  iconos de línea propios.
- **Imitar la refracción de Liquid Glass en CSS** (`backdrop-filter: url()` con un mapa de
  desplazamiento): WebKit no lo admite (bug 245510). Se usa el vidrio del sistema (matiz de abajo).
- **Los estilos de vidrio privados** de `window-vibrancy` (`Sidebar`, `Inspector`…): API privada.
- **Mover las órdenes de cada sección a la barra con un portal**: no hizo falta; la cabecera de cada
  sección ya está arriba y el CSS la convierte en barra.

## Consecuencias

- Hay cuatro hojas de estilo (`estilos.css` para la web y la base, y una por sistema). Una pantalla
  nueva se mira con `?sistema=macos` y `?sistema=windows`.
- El oro puede ser línea en un sitio (los iconos de la barra lateral de macOS), con su propio token
  medido a 3:1.
- La posición de los semáforos depende de cómo los coloca tao: hay que afinarla en un Mac.

## Verificación

- `make comprobar` en verde, con el contraste de las seis paletas (y `oro-trazo` sobre la barra y la
  fila elegida).
- e2e en Chromium: lo de siempre (la web y Windows, iguales) y `macos.spec.ts` (texto de 13 y
  controles de 24, barra lateral de 240 sin marca con filas de 32, la barra de herramientas de 52
  que en el Estudio cubre el inspector, cápsulas de 36, la barra lateral que se oculta, no se alcanza
  con el tabulador y se recuerda, Ajustes en filas que guardan, el vidrio simulado).
- Capturas de todas las pantallas en macOS, en claro y oscuro, con y sin vidrio, y de los controles
  pulsados (`make capturas-macos`), revisadas con el cliente en dos vueltas: «está perfecto como está
  ahora».
- `ventana_macos.rs` pasa clippy para `aarch64-apple-darwin` (proyecto mínimo con Tauri 2.12.1).
- **No verificado**: nada en un Mac todavía. Liquid Glass, la posición de los semáforos, la letra SF
  Pro y las animaciones, a la espera de la compilación de prueba en el Mac del cliente (macOS 27).
  macOS 26 y 11–15, y los Mac Intel, sin ver.

## Matiz del 2026-10-21: lo que salió de verlo en el Mac (v0.14.1)

El cliente actualizó su Mac (macOS 27) a la v0.14.0 y vio tres cosas:

- **La barra lateral, pequeña** para su ancho: pasa al tamaño «large» del kit (filas de 36, iconos
  de 20, letra de 15).
- **El interruptor**: su botón era una elipse dibujada con un degradado, que llenaba la pista y al
  pulsarlo la tapaba entera. Ahora es una píldora en SVG (26 × 16, 2 de margen y su sombra) en una
  pista de 44 × 20 («small»), que al pulsarla se estira 4 px.
- **El selector de modo daba un bote** al pulsarlo (se abombaba entero): ya no; la selección se
  desliza de una opción a otra (`segmentados.ts` mide la elegida y el CSS pinta su fondo).

Y preguntó por la refracción, que otras aplicaciones de Mac consiguen. **WebKit tiene una propiedad
propia, `-apple-visual-effect: -apple-system-glass-material`**, que pinta debajo de un elemento el
material de vidrio del sistema (el de SwiftUI), en macOS 26+. Está detrás de una preferencia privada
de `WKPreferences` (`_setUseSystemAppearance:`). `ventana_macos.rs` la enciende si existe (con
Liquid Glass), `plataforma` lo dice (`vidrio_controles`), y main.tsx inyecta entonces
`vidrio-macos.css`, que pone el material en las cápsulas de la barra, el aviso de versión nueva y el
botón del deslizador al pulsarlo. Se inyecta después porque WebKit descarta la propiedad si lee el
CSS antes de encenderla. Sin ella, la imitación de siempre. Es API privada: vale fuera de la Mac App
Store, que es como se distribuye Apolo. La técnica, de lennondotw/tauri-liquid-glass-demo (MIT).

**Verificación**: e2e de macOS (14, con el segmentado que se desliza, el interruptor de 44 × 20 y
las filas de 36); el minificado conserva la propiedad y solo en la hoja inyectada; clippy de
`ventana_macos.rs` para Apple silicon. **No verificado**: que el material se vea en las cápsulas y en
el botón del deslizador en el Mac (solo existe allí); el botón del interruptor no puede llevarlo (es
el fondo del `<input>`).
