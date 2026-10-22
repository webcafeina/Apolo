# ADR 0029 — Linux con el diseño de Ubuntu 26.10 (libadwaita y Yaru)

**Fecha:** 2026-10-22 · **Estado:** aceptada · v0.15.0 · matiza la [0015](0015-identidad-nativa-con-marca.md)

## Contexto

Con Windows ([0027](0027-windows-con-fluent.md)) y macOS ([0028](0028-macos-con-golden-gate.md))
ya con su lenguaje nativo, vistos y aprobados en las máquinas del cliente, faltaba Linux. Era la
interfaz base (la de la web y de Esfinge) con cuatro retoques en `sistema-linux.css`, sin paleta
propia, con la barra de título del sistema encima de la cabecera de 52 px y sin pruebas propias. El
cliente pidió hacerlo igual que con los otros dos: el lenguaje de **Ubuntu 26.10**, el que tiene en
su máquina virtual (VMPal, Ubuntu 26.10 ARM64).

Investigado el 2026-10-22 con tres agentes:
- Ubuntu 26.10 «Stonking Stingray» (2026-10-15): GNOME 51, **libadwaita 1.10.0**, GTK 4.24, Yaru
  26.10. Sin cambios de diseño propios: libadwaita con el tema Yaru, que **no cambia ninguna
  medida**, solo los colores y el acento (naranja por defecto).
- Las medidas, leídas en la hoja de estilos de libadwaita 1.10 (SCSS) y los colores en el binario
  de Yaru de 26.10.
- Tauri 2.12.1 en Linux: `decorations: false` funciona en GNOME (Wayland y X11, tao #1218), con
  arrastre y doble clic por `data-tauri-drag-region` y una franja de redimensión de 5 px por
  dentro; sin el menú del clic derecho; la transparencia, arriesgada por la GPU.

## Decisión

Todo cuelga de `:root[data-sistema="linux"]` (`frontend/src/sistema-linux.css`), y el marcado
cambia con `esLinux()` y `formularioNativo()` donde ya lo hacía para Windows y macOS.

**Decidido con el cliente** (2026-10-22):

| Tema | Decisión |
|---|---|
| Acento | El oro de Apolo con las formas de GNOME |
| Parte de arriba | **Barra propia, como las apps de GNOME** (elegida sobre capturas de las dos): sin la barra del sistema; la cabecera de la barra lateral y la del contenido a la misma altura (46), y los botones de la ventana redondos y grises (24 en una casilla de 34) en el lado y el orden del sistema (`gtk-decoration-layout`). Solo en GNOME; en otros escritorios, la barra del sistema |
| Iconos | Los simbólicos de **Yaru** (CC BY-SA 4.0, con atribución en `iconos-yaru.ts`) |
| Barra lateral | Como AdwNavigationSplitView: 220 con su cabecera («Apolo» en Heading), filas de 36 con radio 9 y la elegida en gris (el texto al 10 %), iconos de 16 en el color del texto. Se oculta con su botón y se recuerda; por debajo de 860 px va oculta y sale encima del contenido. Sin la marca ni la firma |
| Páginas | Ajustes, Lotes y MCP en **listas en tarjeta** (*boxed list*): filas de 50 con 12 a los lados, tarjetas de radio 12 con la sombra de libadwaita, centradas y hasta 600, con 24 entre grupos |
| Escala | La de Ubuntu: Ubuntu Sans 11 pt (14,67 px), controles de 34 |
| Paleta | `linux_claro` y `linux_oscuro` en `crates/tema`, con los colores de Yaru (ventana `#FAFAFA`/`#2c2c2c`, texto `#3d3d3d`/`#f7f7f7`, barra lateral `#ebebeb`/`#393939`) y el contraste medido |

**Medidas de libadwaita 1.10** (`src/stylesheet`): botón de 34 (24 + 5/10), radio 9, negrita, fondo
del texto al 10/15/30 %, el principal en el acento; campo de 34 con radio 9; casilla de 20 con radio
6; interruptor de 46 × 26 con botón de 20 y sombra `0 2px 4px`; deslizador con pista de 4 y botón
de 20; barra de cabecera de 46; fila de lista de 50; tarjeta con radio 12; barra lateral de 180 a
280 con filas de 36 y radio 9; preferencias con límite de 600; foco de 2 px; banner del gris
`#7d7d83` al 30 %.

**El interruptor, con su botón propio** (también para macOS, arreglo de la v0.15.0): el componente
`Interruptor` pone, junto a la casilla con el papel de `switch`, un `<span class="pomo">`
decorativo. WebKit no deja meter nada dentro de un `<input>`, así que hasta ahora el botón era su
imagen de fondo. Con el botón aparte, en Linux es el de libadwaita y en macOS la píldora que se
hincha en burbuja (con el vidrio del sistema, ADR 0028). En Windows y la web, oculto.

**La ventana** (`src-tauri/src/ventana_linux.rs`): `tauri.linux.conf.json` la deja en `"create":
false` y se crea al arrancar, sin decoraciones si `XDG_CURRENT_DESKTOP` es GNOME, leyendo de GTK la
disposición de los botones (sin `gio::Settings`, que aborta si falta el esquema). `plataforma` lo
cuenta (`barra_propia`, `botones`). Dependencia nueva solo en Linux: `gtk` 0.18 (la que ya trae
Tauri, MIT).

**Para mirarlo sin Linux**: `?sistema=linux&barra=propia` (o `sistema`, con una barra dibujada) y
`&botones=` en el navegador de desarrollo; `make capturas-linux`.

## Alternativas descartadas

- **El acento del sistema** (naranja de Ubuntu): el cliente quiso el oro, como en los otros dos.
- **La barra del sistema** (opción B): más robusta (menú del clic derecho, bordes), pero dos franjas
  arriba; el cliente eligió la integrada viendo las dos. Se queda para los escritorios que no son
  GNOME.
- **Una GtkHeaderBar de verdad** con el webview debajo: sus botones serían de GTK y no de la
  interfaz, y no se fundiría con la barra lateral.
- **Esquinas redondeadas y sombra propias** (ventana transparente): fallos de GPU conocidos en
  WebKitGTK; la ventana se queda con las esquinas del sistema.
- **Los iconos de Adwaita**: más «GNOME genérico»; los de Yaru son los que ve el usuario de Ubuntu.

## Consecuencias

- Cuatro hojas por sistema y una base para la web. Una pantalla nueva se mira con `?sistema=linux`,
  `macos` y `windows`.
- Sin la barra del sistema, en GNOME se pierde el menú del clic derecho sobre la barra y el borde de
  redimensión es fino (deuda.md). Con pantalla táctil hay un fallo conocido de GNOME 50 al
  redimensionar (tauri#16276); Ubuntu 26.10 trae GNOME 51.
- La letra es la del sistema (Ubuntu Sans): en otras distribuciones, la suya.

## Verificación

- `make comprobar`: contraste de las ocho paletas, tokens al día.
- e2e: lo de siempre (la web, Windows y macOS iguales, salvo el interruptor de macOS, con su prueba
  adaptada) y `linux.spec.ts` (escala de 11 pt y controles de 34, iconos de Yaru, barra lateral de
  220 sin marca y con filas de 36, cabecera de 46, botones de la ventana en el lado del sistema, la
  barra lateral que se oculta, se recuerda y sale encima con la ventana estrecha, Ajustes en lista en
  tarjeta con el interruptor de 46 × 26 que guarda). 107 en verde.
- Capturas de todas las pantallas en Linux con las dos partes de arriba (`make capturas-linux`),
  revisadas con el cliente el 2026-10-22: eligió la barra propia y dio el resto por bueno.
- **No verificado**: `ventana_linux.rs` no se compila en el VPS (no hay cabeceras de GTK); lo
  compila CI. Nada en un Ubuntu de verdad todavía: la prueba guiada en la máquina virtual va con
  una compilación de prueba. Otros escritorios, X11, la AppImage con el aspecto nuevo y las
  pantallas táctiles, sin ver.
