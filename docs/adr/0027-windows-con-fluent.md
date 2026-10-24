# ADR 0027 — Windows con Fluent: la estructura y los controles de WinUI

**Fecha:** 2026-10-21 · **Estado:** aceptada · v0.13.0 · matiza la [0015](0015-identidad-nativa-con-marca.md) y la [0016](0016-ventana-translucida.md)

## Contexto

En Windows, Apolo se veía igual que en macOS: la anatomía copiada de Esfinge para Mac (texto de 13,
controles de 28, radios de 9, halos dorados, iconos de línea), la barra lateral con el logotipo y
la barra de título del sistema encima, gris. El cliente pidió, antes de la 1.0, que en Windows
pareciera una aplicación de Microsoft **sin cambiar lo que hace**. Una primera vuelta solo con los
controles de Fluent no bastó: «se sigue pareciendo a la que está… la estructura visual de la ventana
debe ser la nativa de Fluent/WinUI».

Las medidas se sacaron del código de WinUI (NavigationView, TitleBar, InfoBar, CommandBar,
AppBarButton, Expander), del Windows Community Toolkit (SettingsCard, SettingsExpander) y de la
Galería de WinUI.

## Decisión

Todo cuelga de `:root[data-sistema="windows"]` (`frontend/src/sistema-windows.css`), y lo que cambia
el marcado mira `esWindows()`. macOS y Linux quedan como estaban, salvo los interruptores.

**Decidido con el cliente** (2026-10-21, dos rondas):

| Tema | Decisión |
|---|---|
| Acento | El oro de Apolo, con las formas de Fluent (no el acento de Windows). El oro rellena; el foco, los indicadores y las pistas van en `--acento` |
| Paleta | Los neutros de WinUI, en `crates/tema` (`windows_claro`, `windows_oscuro`), con el contraste medido |
| Escala | Segoe UI Variable, texto 14/20, controles de 32, radios 4 y 8, líneas de 1 px |
| Iconos | Fluent System Icons (Microsoft, MIT), copiados en `iconos-fluent.ts`, solo en Windows |
| Interruptores | Con su forma en los tres sistemas (antes salían como casillas) |
| Barra de título | **Propia, de 48 px** (la alta de WinUI): ☰, icono y nombre a la izquierda; minimizar, maximizar y cerrar de 46 × 48 a la derecha. La ventana, sin marco |
| Panel de navegación | NavigationView: transparente sobre el marco, se anima al plegarse (200 ms), filas de 36 con el icono de 16 en 40, el indicador de 3 × 16. ☰ lo pliega a 48 o lo despliega a 240 (320 hasta la v0.18.0, el de WinUI; el cliente lo quiso más estrecho), y se recuerda; **abre plegado** la primera vez (para la imagen). Por debajo de 1008 px de ventana, plegado, y ☰ lo abre encima del contenido |
| Marca | Fuera del panel: el nombre y el icono los pone la barra de título; la versión, Ajustes › Acerca de |
| Páginas | La capa del contenido (esquina de arriba a la izquierda de 8 y su filete); el título en Title 28/36 con 36 px a los lados; Lotes, Presets y Ajustes, en una columna de 1000 **centrada** con su título (a la izquierda hasta la v0.18.0, como la Configuración; el cliente los quiso como en los demás sistemas); Ajustes, Lotes (Calidad y Dónde) y MCP con **SettingsCards**: icono, título, descripción y el control a la derecha, a 4 px una de otra y en grupos con encabezado. Lo que no es un ajuste (un preset, el resumen de un lote) sigue como tarjeta |
| Estudio | Su cabecera es una barra de órdenes: el nombre en BodyStrong, los datos en Caption, las órdenes sin relieve; si falta sitio, el modo de comparar se queda en sus iconos |
| Versión nueva | Una InfoBar dentro del contenido, con su icono |

**La ventana, desde código** (`src-tauri/src/ventana_windows.rs`). `tauri.windows.conf.json` la deja
en `"create": false, "decorations": false`, y se crea al arrancar con `transparent` solo en Windows
11 (build ≥ 22000) y `window_vibrancy::apply_mica`, que devuelve si se pudo; `plataforma` lo cuenta
como vidrio. En Windows 10, o si Mica falla, la interfaz pinta el marco opaco (`--barra`). Las
esquinas se piden redondeadas (`DWMWCP_ROUND`). Tauri ya redimensiona una ventana sin marco desde
los bordes; arrastrar es `data-tauri-drag-region` y el doble clic maximiza.

**Snap.** Windows enseña el menú de Snap cuando la ventana contesta `HTMAXBUTTON` a `WM_NCHITTEST`,
pero sobre la barra contesta la ventana hija de WebView2. Se pone otra ventana hija, invisible
(`WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS`, pincel nulo), encima del botón de maximizar, que contesta
`HTMAXBUTTON`, maximiza ella al hacer clic y avisa a la interfaz con el evento `ventana:maximizar`
(«encima», «pulsado», «fuera») para que el botón se ilumine. Se recoloca con `WM_SIZE` y
`WM_DPICHANGED`. Es la técnica de todos los complementos de Tauri que lo resuelven, escrita aquí
(unas 200 líneas), sin depender de ellos. Las medidas de la barra están en el CSS **y** en
`ventana_windows.rs` (`ALTO_BARRA`, `ANCHO_BOTON`).

**Dependencias nuevas**, solo en Windows y en las versiones que Tauri ya trae: `window-vibrancy`,
`windows-version` y `windows-sys` (MIT o Apache-2.0). Permisos nuevos: minimizar, maximizar y
cerrar la ventana.

**Para mirarlo sin Windows**: `?sistema=windows` (y `&vidrio=si`, con un degradado que imita Mica)
en el navegador de desarrollo, y `make capturas-windows`.

## Alternativas descartadas

- **El acento de Windows** en vez del oro: Apolo dejaría de reconocerse; el cliente quiso el oro.
- **Solo los controles de Fluent**, con la estructura de Mac: la primera vuelta, y no bastó.
- **La barra de título del sistema sobre Mica**: la opción de menos riesgo (Snap nativo, sin código
  Win32), recomendada; el cliente prefirió la barra alta propia, con ☰ dentro, como la Galería de
  WinUI.
- **`tauri-plugin-decorum`**: consigue Snap simulando pulsaciones de teclado (Win+Z), que no da el
  mismo menú y roba el foco. **`tauri-plugin-frame` y similares**: hacen lo mismo que lo de aquí,
  pero son jóvenes y de una sola persona; son 200 líneas.
- **Window Controls Overlay de WebView2**: experimental. **`app-region: drag`**: solo arrastra.
- **`windowEffects` de Tauri** para Mica: se traga el error, y no se sabría si pintar el marco
  opaco. **Acrylic** de fondo: es para lo que flota, no para la ventana.
- **Dos interfaces** (una de Windows aparte): el marcado cambia solo donde hace falta (Ajustes,
  Lotes, MCP, la barra y el panel), con `esWindows()`.

## Consecuencias

- Hay tres hojas de estilo por sistema (`estilos.css`, `sistema-windows.css`, `sistema-linux.css`)
  y algo de marcado con dos ramas. Una pantalla nueva tiene que mirarse con `?sistema=windows`.
- La barra de título es de Apolo: lo que hace Windows con la suya (el menú de sistema con clic
  derecho o Alt+Espacio, el atenuado al perder el foco) hay que hacerlo aquí. El atenuado está; el
  menú de sistema, no.
- El ancho mínimo de la ventana sigue en 900: Microsoft pide 500 o menos para que quepa en todas
  las zonas de Snap. En pantallas pequeñas, las zonas estrechas no llegan (deuda.md).
- En el navegador la letra no es Segoe UI y Mica es un degradado: las capturas orientan, pero lo
  que vale es la ventana.

## Verificación

- `make comprobar` en verde: contraste de las cuatro paletas, tokens al día, la interfaz.
- e2e en Chromium: lo de siempre (macOS y la web, iguales salvo los interruptores) y
  `windows.spec.ts` (escala de 14 y controles de 32, iconos de Fluent, interruptores que se marcan,
  la barra de progreso sin transición, el vidrio simulado, la barra de título de 48 con los botones
  fuera del tabulador, el panel sin marca y con filas de 36, que abre plegado y se recuerda, la
  ventana estrecha con el panel encima, Ajustes con tarjetas que guardan).
- Capturas de todas las pantallas en Windows, en claro y oscuro, con y sin vidrio, revisadas con el
  cliente (dos rondas): «ahora está perfecto».
- `ventana_windows.rs` compila para `x86_64-pc-windows-msvc` y pasa clippy sin avisos (comprobado
  aparte, con un proyecto mínimo con Tauri 2.12.1).
- En Windows x64, `humo.yml` instala la compilación de prueba y la ventana arranca y sigue abierta.
- **En la máquina virtual Windows 11 ARM del cliente** (2026-10-21, compilación de prueba
  39590749049): Mica, la barra de título (arrastrar, doble clic, los tres botones), el menú de Snap
  y el encaje en sus zonas, redimensionar, ☰ y todas las pantallas, bien. Pidió que el panel se
  animara al plegarse, como el nativo: 200 ms con la curva de deceleración de Fluent. Mica, con su
  tinte normal (no Mica Alt).
- **No verificado**: el tema oscuro en Windows (su copia no está activada y no deja cambiarlo),
  Windows 10 y Mica en x64 a ojo.
