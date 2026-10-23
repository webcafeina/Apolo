# ADR 0031 — Linux con KDE Plasma: el lenguaje Breeze

**Fecha:** 2026-10-22 · **Estado:** aceptada · v0.17.0 · matiza la [0029](0029-linux-con-ubuntu.md)

## Contexto

Desde la v0.15.0 (ADR 0029), Apolo en Linux tiene el aspecto de Ubuntu 26.10 (libadwaita con Yaru)
en cualquier escritorio. En KDE salía con la barra de KWin encima y, dentro, una aplicación de GNOME.
El cliente pidió adaptarla a KDE, el otro gran escritorio de Linux (más usado que GNOME en Arch y en
Fedora KDE), al plantear la v0.16.0 (ADR 0030).

Investigado el 2026-10-22 con un agente, leyendo el código de KDE de octubre de 2026 (`breeze`:
kstyle, colores y decoración; `kirigami`; `qqc2-desktop-style`; `plasma-integration`;
`systemsettings`; `breeze-icons`), clonado en `~/.cache/kde-ref/`.

## Decisión

Un segundo aspecto de Linux, solo con KDE: `ventana_linux.rs::escritorio()` lee
`XDG_CURRENT_DESKTOP` (una lista) y da `gnome`, `kde` u `otro`; `plataforma` lo devuelve y la
interfaz lo pone en `<html data-escritorio>`. Todo lo de KDE cuelga de
`:root[data-sistema="linux"][data-escritorio="kde"]` (`frontend/src/sistema-kde.css`), que pisa a
`sistema-linux.css` solo en lo que cambia. `otro` (Xfce, Cinnamon, COSMIC…) se queda con el de GNOME.

**Decidido con el cliente** (2026-10-22, sobre la galería de capturas):

| Tema | Decisión |
|---|---|
| Parte de arriba | **La barra de Apolo, sin la de KDE** (opción A), como en GNOME, con los botones de la ventana de Breeze y el marco de Breeze (radio 5, su sombra) |
| Botón principal | **Relleno de oro**, como en los otros sistemas (no el tinte suave del botón por defecto de KDE) |
| Acento | El oro donde Breeze pone su azul `#3daee9` |

**Medidas de Breeze** (con la ruta de cada una en el informe del agente; sesiones.md):
- Letra Noto Sans de 10 pt (13,33 px), pequeña de 8; radio 5 (`Frame_FrameRadius`).
- Botón: 34 de widget y 32 visibles, fondo `Button`, borde de 1 del texto al 20 %, sombra de 1 abajo;
  al pasar, el borde en el acento; al pulsar, el acento al 30 %.
- Campo de 30, en el fondo de las vistas con el borde de los marcos; desplegable como un botón.
- Casilla de 16 con radio 4; interruptor de 36 con pista de 12 y botón de 18; deslizador con pista
  de 6 y botón de 18; foco con el borde en el acento y un anillo de 2.
- Cabecera de 46 en el color Header con su filete; barra lateral en el color de las vistas, filas de
  30 cada 36 con radio 5, la elegida rellena del acento (Ajustes del sistema; 240 de ancho, no 270,
  para que quepa el Estudio).
- Los ajustes, como un Kirigami.FormLayout: la etiqueta a la derecha en una columna, el control y
  su explicación en la otra; las secciones, en seminegrita.
- Bordes y separadores: el texto al 20 % sobre el fondo, como `frameContrast` de Breeze.
- La versión nueva, como un Kirigami.InlineMessage: el acento al 20 % con su borde.

**Paletas** `kde_claro` y `kde_oscuro` en `crates/tema` (diez en total), con los colores de
`BreezeLight.colors` y `BreezeDark.colors` y el contraste medido.

**Iconos** de Breeze (`frontend/src/iconos-breeze.ts`, 29, los monocromos de 16 de
`breeze-icons`), con su licencia: **LGPL-3.0 o posterior**, compatible con la GPLv3.

**La ventana**: `barra_de_apolo()` es GNOME o KDE; en KDE el marco de GTK lleva `MARCO_KDE` (radio
5, la sombra de Breeze) y la interfaz recorta sus esquinas a 5.

**Para mirarlo sin KDE**: `?sistema=linux&escritorio=kde&barra=propia` y `make capturas-kde`.

## Alternativas descartadas

- **La barra de KDE** (opción B): es lo propio de KDE (Dolphin, Kate) y conserva el menú del clic
  derecho y las reglas de ventana; el cliente prefirió la integrada, como en GNOME.
- **El botón principal de KDE** (un tinte suave del acento): el cliente prefirió el oro relleno.
- **Un aspecto para cada escritorio** (COSMIC, Cinnamon, elementary): poco uso, y COSMIC y
  elementary ya leen el acento y el modo oscuro del portal. Se quedan con el de GNOME.
- **Las tarjetas de libadwaita en KDE**: las páginas de ajustes de KDE son formularios.
- **Los iconos de Yaru en KDE** o el azul de Breeze: ni uno ni otro son de Apolo en KDE.

## Consecuencias

- Tres aspectos en Linux (GNOME, KDE y el resto como GNOME) y cinco hojas por sistema. Una pantalla
  nueva se mira también con `?sistema=linux&escritorio=kde`.
- Sin la barra de KDE se pierden el menú del clic derecho sobre la barra y las reglas de ventana de
  KWin (deuda.md, la misma de GNOME).
- La letra es la del sistema: Noto Sans en Plasma.

## Verificación

- `make comprobar` (contraste de las diez paletas, tokens al día) y e2e con `kde.spec.ts` (escala de
  10 pt, botones de 32, barra lateral de 240 con filas de 30 y la elegida en el acento, iconos de
  Breeze, botones de la ventana de 20, Ajustes en formulario, el interruptor de Breeze, el radio 5,
  y GNOME sin cambios).
- Capturas con las dos partes de arriba (`make capturas-kde`), revisadas con el cliente el
  2026-10-22 (galería https://claude.ai/artifact/AGwN7QZdye1UBGJa8bKkDR): eligió la barra propia y
  el oro relleno.
- **No verificado**: nada en un KDE de verdad todavía. `ventana_linux.rs` lo compila CI. El marco de
  GTK con la ventana transparente bajo KWin (la sombra, el redimensionado), la disposición de los
  botones que KDE pasa a GTK y la letra Noto Sans, con una compilación de prueba en Kubuntu ARM
  (`kubuntu-desktop` sobre Ubuntu 26.10) y después en Fedora KDE.

## Matiz del 2026-10-23: lo que salió de verlo en Kubuntu

El cliente lo vio en Kubuntu ARM (compilación de prueba 39847459084): «todo correcto», con tres
cosas. **El icono de la barra lateral** salía con la flecha fuera de sitio (el fallo de la «m» de
trampas.md, que también rompía tres iconos de Yaru en Ubuntu desde la v0.15.0) y no cambiaba al
plegarla: ahora es `sidebar-collapse-left` con la barra abierta y `sidebar-expand-left` con ella
plegada, como en Breeze (los iconos de Breeze admiten la transformación `t`). **Diferencias** pasa
de `vcs-diff` (diferencias entre versiones de código) a `path-difference`. Y **el grupo de modos del
Estudio** (Deslizador, Lado a lado, Diferencias) se confundía con «Abrir otra…»: ahora va en una
bandeja (el texto al 7 %, radio 6) con el elegido levantado como un botón de Breeze.

Con la compilación de prueba 39919081725 el cliente lo vio «perfecto» en Kubuntu ARM (2026-10-23).

## Matiz del 2026-10-23: lo que salió de verlo en Fedora KDE

Con la misma compilación, en Fedora KDE ARM (Plasma 6, sesión Plasma), el cliente lo vio «todo
perfecto»: el marco, la ventana y las pantallas. Apuntó dos cosas. **Agentes de IA** seguía el
formulario de Ajustes: la etiqueta a la izquierda y el botón de la extensión y la orden de Claude
Code apretados en la columna derecha, entre dos tarjetas a todo el ancho. Eso no son ajustes, así
que en KDE va en tarjetas (Kirigami.Card), como lo de arriba y lo de abajo. Y **un nombre de archivo
largo** en el Estudio se metía debajo de los modos del comparador. Era de todo Linux, no solo de KDE
(trampas.md), y se arregla en `sistema-linux.css`.
